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

/// Offset of a soft dab's centre from the input point, in px on both axes. Observed at one
/// position only (B-009: the size-101 soft dabs centre on the pixel corner next to the press
/// point); the hard tip has no offset (B-001).
pub const SOFT_CENTRE_OFFSET: f64 = 0.5;

/// Radius beyond which a dab of radius `r` covers nothing.
pub fn reach(r: f64, hardness: u8) -> f64 {
    if hardness >= HARDNESS_MAX {
        r.max(1.0)
    } else {
        (PROFILES[0].len() as f64 - 1.0) * STEP * r / REF_RADIUS
    }
}

/// Coverage in [0, 1] of a pixel whose centre is (`dx`, `dy`) from the dab centre.
pub fn coverage(dx: f64, dy: f64, r: f64, aa: Aa, hardness: u8) -> f64 {
    let d2 = dx * dx + dy * dy;
    if hardness < HARDNESS_MAX {
        return soft(d2.sqrt(), r, hardness);
    }
    if aa == Aa::None {
        // Closed disc. The exact rule for pixels exactly on the edge is open (B-001).
        return if d2 <= r * r { 1.0 } else { 0.0 };
    }
    // B-001: dabs under 2 px keep a 1 px radius and scale their amplitude by R instead.
    let re = r.max(1.0);
    if d2 >= re * re {
        return 0.0;
    }
    let w = aa.ramp().min(r).max(1.0);
    r.min(1.0) * ((re - d2.sqrt()) / w).clamp(0.0, 1.0)
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
        // Closed disc.
        assert_eq!(coverage(1.0, 0.0, 1.0, Aa::None, 5), 1.0);
    }
}
