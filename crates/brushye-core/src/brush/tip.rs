//! Round-tip dab coverage (specs B-001, B-009).
//!
//! `coverage` returns the fraction of a dab covering one pixel, before density. Distances are
//! measured from the dab centre to the pixel centre.

use super::hardness_tables::{PROFILES, REF_RADIUS, STEP};

/// Anti-aliasing level (CSP's four-step control).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Aa {
    None,
    Weak,
    #[default]
    Middle,
    Strong,
}

impl Aa {
    pub fn name(self) -> &'static str {
        match self {
            Aa::None => "none",
            Aa::Weak => "weak",
            Aa::Middle => "middle",
            Aa::Strong => "strong",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        Some(match s {
            "none" => Aa::None,
            "weak" => Aa::Weak,
            "middle" => Aa::Middle,
            "strong" => Aa::Strong,
            _ => return None,
        })
    }
    /// Edge ramp width limit W_L (B-001).
    fn ramp(self) -> f64 {
        match self {
            Aa::None => 0.0,
            Aa::Weak => 1.5,
            Aa::Middle => 2.5,
            Aa::Strong => 3.5,
        }
    }
}

/// Hardest level of the 5-step hardness control (B-009).
pub const HARDNESS_MAX: u8 = 5;

/// Offset of a soft dab's centre from the input point, in px on both axes: fixed, at every
/// sub-pixel position and size (B-009 run 2). The hard round tip has no offset (B-001).
pub const SOFT_CENTRE_OFFSET: f64 = 0.5;
/// Offset of a thinned (elliptic) hard dab's centre in x (B-010 run 2).
pub const THIN_CENTRE_OFFSET_X: f64 = -0.5;
/// Extra half-width of each AA None scanline span, in px (B-001 run 2).
pub const SCANLINE_SLACK: f64 = 0.1;

/// Which axis tip thickness shortens (CSP's "Direction").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TipAxis {
    /// Shortens the vertical axis at angle 0: a flat, wide dab.
    #[default]
    Vertical,
    Horizontal,
}

impl TipAxis {
    pub fn name(self) -> &'static str {
        match self {
            TipAxis::Vertical => "vertical",
            TipAxis::Horizontal => "horizontal",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "vertical" => Some(TipAxis::Vertical),
            "horizontal" => Some(TipAxis::Horizontal),
            _ => None,
        }
    }
}

/// One dab's shape: radius, thickness and orientation of its long axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    /// Radius along the long axis, px.
    pub r: f64,
    /// Short axis / long axis, (0, 1].
    pub thickness: f64,
    /// Unit vector of the long axis (screen coordinates, y down).
    pub axis: [f64; 2],
    pub aa: Aa,
    pub hardness: u8,
}

impl Shape {
    pub fn round(r: f64, aa: Aa, hardness: u8) -> Self {
        Self {
            r,
            thickness: 1.0,
            axis: [1.0, 0.0],
            aa,
            hardness,
        }
    }

    fn thin(&self) -> bool {
        self.thickness < 1.0
    }

    /// Offset of the dab centre from the input point.
    pub fn centre_offset(&self) -> (f64, f64) {
        if self.hardness < HARDNESS_MAX {
            if soft_offset(2.0 * self.r, self.aa, self.hardness) {
                (SOFT_CENTRE_OFFSET, SOFT_CENTRE_OFFSET)
            } else {
                (0.0, 0.0)
            }
        } else if self.thin() {
            (THIN_CENTRE_OFFSET_X, 0.0)
        } else {
            (0.0, 0.0)
        }
    }

    /// Radius beyond which the dab covers nothing.
    pub fn reach(&self) -> f64 {
        reach(self.r, self.hardness)
    }

    /// Coverage in [0, 1] of a pixel whose centre is (`dx`, `dy`) from the dab centre.
    pub fn coverage(&self, dx: f64, dy: f64) -> f64 {
        if !self.thin() {
            return coverage(dx, dy, self.r, self.aa, self.hardness);
        }
        // Thin tip: an ellipse with semi-axes R and R·T (B-010). Distances are measured in
        // the ellipse's frame with the short axis stretched back to R.
        let along = dx * self.axis[0] + dy * self.axis[1];
        let across = (dy * self.axis[0] - dx * self.axis[1]) / self.thickness;
        if self.hardness >= HARDNESS_MAX && self.aa == Aa::None {
            // Closed ellipse, exact on 3690 of 3694 px (B-010).
            let r = self.r;
            return if along * along + across * across <= r * r {
                1.0
            } else {
                0.0
            };
        }
        // AA edges and soft tips on thin dabs are not observed: the round model applied in the
        // ellipse's frame.
        coverage_radial(
            (along * along + across * across).sqrt(),
            self.r,
            self.aa,
            self.hardness,
        )
    }
}

/// Smallest size at which an anti-aliased soft dab of level 1..=4 is offset by +0.5 px (B-015,
/// AA Middle: level 1 offset at 60 (B-012 X-04) and 64 but not 32, levels 2–3 at 128, level 4 never up to 128; the switch
/// between the sampled sizes 8/16/32/64/128 is interim). With AA None every soft dab is offset
/// (B-009).
const SOFT_OFFSET_FROM_SIZE: [f64; 4] = [60.0, 128.0, 128.0, f64::INFINITY];

/// Where level 4's AA ramp ends, px beyond the radius (B-015 fit: dab ink within 1–2 % at
/// sizes 16–128, +10 % at 8; strokes within 3 %).
const LEVEL4_AA_EDGE: f64 = 0.25;

fn soft_offset(size: f64, aa: Aa, hardness: u8) -> bool {
    aa == Aa::None || size >= SOFT_OFFSET_FROM_SIZE[usize::from(hardness.clamp(1, 4) - 1)]
}

/// Radius beyond which a dab of radius `r` covers nothing.
pub fn reach(r: f64, hardness: u8) -> f64 {
    if hardness >= HARDNESS_MAX {
        r.max(1.0) + SCANLINE_SLACK
    } else {
        (PROFILES[0].len() as f64 - 1.0) * STEP * r / REF_RADIUS
    }
}

/// Coverage in [0, 1] of a pixel whose centre is (`dx`, `dy`) from a round dab's centre.
pub fn coverage(dx: f64, dy: f64, r: f64, aa: Aa, hardness: u8) -> f64 {
    if hardness >= HARDNESS_MAX && aa == Aa::None {
        // Scanline fill (B-001 run 2): rows with −R ≤ dy < R (the top tangent row counts, the
        // bottom one doesn't); in a row, pixels with |dx| ≤ √(R² − dy²) + 0.1.
        if dy < -r || dy >= r {
            return 0.0;
        }
        let half = (r * r - dy * dy).max(0.0).sqrt() + SCANLINE_SLACK;
        return if dx.abs() <= half { 1.0 } else { 0.0 };
    }
    coverage_radial((dx * dx + dy * dy).sqrt(), r, aa, hardness)
}

/// Round-tip coverage at distance `d` from the centre (AA levels and soft tips).
fn coverage_radial(d: f64, r: f64, aa: Aa, hardness: u8) -> f64 {
    if hardness < HARDNESS_MAX {
        let v = soft(d, r, hardness);
        // Level 4 keeps a hard edge, and anti-aliasing applies to it: the profile is capped by
        // the round tip's AA ramp (B-015: ink at size 8 went from +90 % to −5 %).
        if hardness == HARDNESS_MAX - 1 && aa != Aa::None {
            let re = r.max(1.0);
            let w = aa.ramp().min(r).max(1.0);
            return v.min(((re + LEVEL4_AA_EDGE - d) / w).clamp(0.0, 1.0));
        }
        return v;
    }
    if aa == Aa::None {
        return if d <= r { 1.0 } else { 0.0 };
    }
    // B-001: dabs under 2 px keep a 1 px radius and scale their amplitude by R instead.
    let re = r.max(1.0);
    if d >= re {
        return 0.0;
    }
    let w = aa.ramp().min(r).max(1.0);
    r.min(1.0) * ((re - d) / w).clamp(0.0, 1.0)
}

/// Interim soft tip: the observed size-101 profile, scaled to radius `r` (B-009 open).
fn soft(d: f64, r: f64, hardness: u8) -> f64 {
    let t = &PROFILES[usize::from(hardness.clamp(1, 4) - 1)];
    if r <= 0.0 {
        return 0.0;
    }
    let pos = d * (REF_RADIUS / r) / STEP;
    let last = t.len() - 1;
    if pos >= last as f64 {
        return 0.0;
    }
    let i = pos as usize; // pos >= 0, so this is floor
    let f = pos - i as f64;
    let (a, b) = (f64::from(t[i]), f64::from(t[i + 1]));
    (a + (b - a) * f) / (255.0 * 256.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_examples() {
        // B-001: size 1 at a pixel corner gives 4 × 37 (coverage 0.5·(1−√0.5)).
        let c = coverage(0.5, 0.5, 0.5, Aa::Middle, 5);
        assert_eq!(((65535.0 * c) as u32) >> 8, 37);
        // AA None draws nothing for size 0.7 at a pixel corner.
        assert_eq!(coverage(0.5, 0.5, 0.35, Aa::None, 5), 0.0);
        // Scanline rule: size 2 at a pixel centre lights the top, left and right neighbours
        // but not the bottom one (B-001).
        assert_eq!(coverage(0.0, -1.0, 1.0, Aa::None, 5), 1.0);
        assert_eq!(coverage(1.0, 0.0, 1.0, Aa::None, 5), 1.0);
        assert_eq!(coverage(-1.0, 0.0, 1.0, Aa::None, 5), 1.0);
        assert_eq!(coverage(0.0, 1.0, 1.0, Aa::None, 5), 0.0);
    }
}
