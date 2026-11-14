//! Dynamics ("effector") blobs: big-endian, a 44-byte header and curve blocks (sut.md,
//! "Dynamics"; S-01, S-37..S-43).

use brushye_command::core::brush::Curve;

use crate::Note;

/// Input bits of header word 2.
pub const PRESSURE: u32 = 0x10;
pub const TILT: u32 = 0x20;
pub const VELOCITY: u32 = 0x40;
pub const RANDOM: u32 = 0x80;

/// A parsed effector blob.
#[derive(Clone, Debug, PartialEq)]
pub struct Effector {
    /// Inputs that drive the setting (bits above).
    pub mask: u32,
    /// Pen pressure minimum, percent.
    pub pressure_min: u32,
    /// Pen pressure curve points, endpoints included; empty when absent.
    pub pressure_curve: Vec<[f64; 2]>,
}

fn u32_at(b: &[u8], i: usize) -> Result<u32, String> {
    b.get(i..i + 4)
        .map(|w| u32::from_be_bytes([w[0], w[1], w[2], w[3]]))
        .ok_or_else(|| format!("dynamics data ends at byte {}", b.len()))
}

fn f64_at(b: &[u8], i: usize) -> Result<f64, String> {
    b.get(i..i + 8)
        .map(|w| f64::from_be_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]))
        .ok_or_else(|| format!("dynamics data ends at byte {}", b.len()))
}

/// A curve block at `at`: u32 12, u32 count, u32 16, then (x, y) f64 pairs.
fn curve(b: &[u8], at: usize, len: usize) -> Result<Vec<[f64; 2]>, String> {
    if u32_at(b, at)? != 12 || u32_at(b, at + 8)? != 16 {
        return Err("unrecognised curve block".into());
    }
    let n = u32_at(b, at + 4)? as usize;
    if n > 64 || 12 + n * 16 != len {
        return Err(format!(
            "curve block of {len} bytes doesn't hold {n} points"
        ));
    }
    (0..n)
        .map(|i| {
            let p = at + 12 + i * 16;
            Ok([f64_at(b, p)?, f64_at(b, p + 8)?])
        })
        .collect()
}

impl Effector {
    pub fn parse(b: &[u8]) -> Result<Self, String> {
        if u32_at(b, 0)? != 44 {
            return Err("unrecognised dynamics header".into());
        }
        let mask = u32_at(b, 8)?;
        let pressure_min = u32_at(b, 12)?;
        let pressure_len = u32_at(b, 32)? as usize;
        let pressure_curve = if pressure_len == 0 {
            Vec::new()
        } else {
            curve(b, 44, pressure_len)?
        };
        Ok(Self {
            mask,
            pressure_min,
            pressure_curve,
        })
    }

    /// Brushye's pen-pressure minimum and curve for this setting, with notes for what's lost.
    pub fn to_brushye(&self, label: &str) -> (Option<u8>, Option<Curve>, Vec<Note>) {
        let mut notes = Vec::new();
        for (bit, input) in [(TILT, "tilt"), (VELOCITY, "velocity"), (RANDOM, "random")] {
            if self.mask & bit != 0 {
                notes.push(Note::NotImported {
                    feature: format!("{label} dynamics"),
                    detail: format!("{input} input"),
                });
            }
        }
        if self.mask & PRESSURE == 0 {
            return (None, None, notes);
        }
        let min = self.pressure_min.min(100) as u8;
        let pts = &self.pressure_curve;
        let curve = match pts.as_slice() {
            [] | [[0.0, 0.0], [1.0, 1.0]] => None,
            [first, inner @ .., last] if *first == [0.0, 0.0] && *last == [1.0, 1.0] => {
                match Curve::new(inner.to_vec()) {
                    Ok(c) => {
                        if inner.len() > 1 {
                            notes.push(Note::Approximated {
                                setting: format!("{label} pressure curve"),
                                detail: format!(
                                    "{} interior points; curves with more than one aren't observed yet",
                                    inner.len()
                                ),
                            });
                        }
                        Some(c)
                    }
                    Err(e) => {
                        notes.push(Note::Approximated {
                            setting: format!("{label} pressure curve"),
                            detail: format!("{e}; a straight line is used"),
                        });
                        None
                    }
                }
            }
            _ => {
                notes.push(Note::Approximated {
                    setting: format!("{label} pressure curve"),
                    detail: "its ends aren't at (0,0) and (1,1); a straight line is used".into(),
                });
                None
            }
        };
        (Some(min), curve, notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(mask: u32, min: u32, curve_len: u32) -> Vec<u8> {
        [44, 0xF0, mask, min, 100, 0, 0, 0, curve_len, 0, 500]
            .iter()
            .flat_map(|w: &u32| w.to_be_bytes())
            .collect()
    }

    fn block(points: &[[f64; 2]]) -> Vec<u8> {
        let mut b: Vec<u8> = [12u32, points.len() as u32, 16]
            .iter()
            .flat_map(|w| w.to_be_bytes())
            .collect();
        for p in points {
            b.extend(p[0].to_be_bytes());
            b.extend(p[1].to_be_bytes());
        }
        b
    }

    #[test]
    fn parses_the_sample_layouts() {
        // S-01 opacity: no inputs, no curve (44 bytes).
        let e = Effector::parse(&header(0, 0, 0)).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(e.to_brushye("Opacity").0, None);
        // S-38: pressure, minimum 20, an S-curve with 4 points.
        let pts = [[0.0, 0.0], [0.2566, 0.1043], [0.75, 0.8957], [1.0, 1.0]];
        let mut b = header(PRESSURE, 20, 76);
        b.extend(block(&pts));
        let e = Effector::parse(&b).unwrap_or_else(|e| unreachable!("{e}"));
        let (min, curve, notes) = e.to_brushye("Brush size");
        assert_eq!(min, Some(20));
        assert_eq!(
            curve.map(|c| c.points().to_vec()),
            Some(vec![[0.2566, 0.1043], [0.75, 0.8957]])
        );
        assert_eq!(notes.len(), 1);
        // S-41: pressure + velocity; velocity is reported.
        let mut b = header(PRESSURE | VELOCITY, 0, 44);
        b.extend(block(&[[0.0, 0.0], [1.0, 1.0]]));
        let (min, curve, notes) = Effector::parse(&b)
            .unwrap_or_else(|e| unreachable!("{e}"))
            .to_brushye("Brush size");
        assert_eq!((min, curve.is_none(), notes.len()), (Some(0), true, 1));
    }

    #[test]
    fn rejects_truncated_and_inconsistent_data() {
        assert!(Effector::parse(&[0, 0, 0]).is_err());
        let mut b = header(PRESSURE, 0, 76);
        b.extend(block(&[[0.0, 0.0], [1.0, 1.0]]));
        assert!(Effector::parse(&b).is_err());
    }
}
