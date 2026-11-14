//! From input samples to the path the dabs follow (specs B-002, B-007).
//!
//! 1. **Stabilization** (B-007): each event becomes the mean of the last N = S events
//!    (positions only); at release the last event is repeated so the path catches up with the
//!    release point. Observed for S ≤ 20; larger settings use the same rule (open).
//! 2. **Smoothing** (B-002 run 2): the events are control points of a quadratic B-spline: a
//!    straight piece from the first event to the first segment midpoint, a quadratic Bézier
//!    (mᵢ, Pᵢ, mᵢ₊₁) around every interior event, a straight piece to the last event. Pressure
//!    follows the same spline.
//! 3. The spline is flattened into a fine polyline (straight pieces stay single segments, so
//!    straight strokes keep exact arithmetic), along which dabs are placed by arc length.

/// A point with pressure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
    pub p: f64,
}

/// Moving mean over the last `s` events, plus the catch-up at release (B-007).
pub fn stabilize(pts: &[Pt], s: u8) -> Vec<Pt> {
    stabilize_with(pts, s, true)
}

/// [`stabilize`] while the pen is still down: no catch-up yet. Each output point depends only
/// on the events up to it, so a live stroke's prefix never changes.
pub fn stabilize_live(pts: &[Pt], s: u8) -> Vec<Pt> {
    stabilize_with(pts, s, false)
}

fn stabilize_with(pts: &[Pt], s: u8, catch_up: bool) -> Vec<Pt> {
    if s == 0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let n = usize::from(s);
    let Some(&last) = pts.last() else {
        return Vec::new();
    };
    let mut ext: Vec<Pt> = pts.to_vec();
    if catch_up {
        ext.extend(std::iter::repeat_n(last, n - 1));
    }
    let mut out = Vec::with_capacity(ext.len());
    for i in 0..ext.len() {
        let w = &ext[i.saturating_sub(n - 1)..=i];
        let k = w.len() as f64;
        let (mut sx, mut sy) = (0.0, 0.0);
        for q in w {
            sx += q.x;
            sy += q.y;
        }
        out.push(Pt {
            x: sx / k,
            y: sy / k,
            p: ext[i].p,
        });
    }
    dedup(out)
}

/// Drop repeated positions (the latest pressure wins); repeats add no path length.
pub fn dedup(pts: Vec<Pt>) -> Vec<Pt> {
    let mut out: Vec<Pt> = Vec::with_capacity(pts.len());
    for q in pts {
        match out.last_mut() {
            Some(l) if l.x == q.x && l.y == q.y => l.p = q.p,
            _ => out.push(q),
        }
    }
    out
}

fn mid(a: Pt, b: Pt) -> Pt {
    Pt {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
        p: (a.p + b.p) / 2.0,
    }
}

/// Largest flattening step along a curved piece, in px.
const FLAT_STEP: f64 = 0.5;
const MAX_PIECES: usize = 512;

/// Flatten the quadratic B-spline through control points `c` into a polyline (B-002).
pub fn spline(c: &[Pt]) -> Vec<Pt> {
    let n = c.len();
    if n <= 2 {
        return c.to_vec();
    }
    let mut out = vec![c[0], mid(c[0], c[1])];
    for i in 1..n - 1 {
        let (m0, p, m1) = (mid(c[i - 1], c[i]), c[i], mid(c[i], c[i + 1]));
        // Collinear control points: the piece is straight.
        let cross = (p.x - m0.x) * (m1.y - m0.y) - (p.y - m0.y) * (m1.x - m0.x);
        if cross == 0.0 {
            out.push(m1);
            continue;
        }
        let len = ((p.x - m0.x).powi(2) + (p.y - m0.y).powi(2)).sqrt()
            + ((m1.x - p.x).powi(2) + (m1.y - p.y).powi(2)).sqrt();
        let pieces = ((len / FLAT_STEP).ceil() as usize).clamp(2, MAX_PIECES);
        for k in 1..=pieces {
            let u = k as f64 / pieces as f64;
            let v = 1.0 - u;
            let (w0, w1, w2) = (v * v, 2.0 * u * v, u * u);
            out.push(Pt {
                x: w0 * m0.x + w1 * p.x + w2 * m1.x,
                y: w0 * m0.y + w1 * p.y + w2 * m1.y,
                p: w0 * m0.p + w1 * p.p + w2 * m1.p,
            });
        }
    }
    out.push(c[n - 1]);
    dedup(out)
}

/// Arc length of the part of `spline(c)` that can't change when more control points are
/// appended, or a later event at the same position updates the last pressure: everything up
/// to the midpoint m₍ₙ₋₂₎ of the last-but-one segment. `None` while nothing is final yet.
///
/// The prefix is computed from the same points in the same order as the full spline, so its
/// length compares exactly with the full polyline's arc lengths.
pub fn final_length(c: &[Pt]) -> Option<f64> {
    let n = c.len();
    if n < 3 {
        return None;
    }
    // spline(c[..n-1]) ends with the straight piece m₍ₙ₋₂₎ → c₍ₙ₋₂₎; drop it.
    let prefix = spline(&c[..n - 1]);
    let k = prefix.len();
    if k < 2 {
        return None;
    }
    let line = Polyline::new(prefix[..k - 1].to_vec());
    Some(line.len())
}

/// A polyline with cumulative arc length, for placing dabs by distance.
pub struct Polyline {
    pts: Vec<Pt>,
    cum: Vec<f64>,
}

/// A point on the path and the unit direction of travel there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct At {
    pub pt: Pt,
    pub dir: [f64; 2],
}

impl Polyline {
    pub fn new(pts: Vec<Pt>) -> Self {
        let mut cum = Vec::with_capacity(pts.len());
        let mut total = 0.0;
        cum.push(0.0);
        for w in pts.windows(2) {
            let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
            total += (dx * dx + dy * dy).sqrt();
            cum.push(total);
        }
        Self { pts, cum }
    }

    pub fn len(&self) -> f64 {
        self.cum.last().copied().unwrap_or(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.pts.is_empty()
    }

    /// Point at arc length `t`. `seg` is a cursor that only moves forward (t never decreases).
    pub fn at(&self, t: f64, seg: &mut usize) -> Option<At> {
        let first = *self.pts.first()?;
        if self.pts.len() == 1 {
            return Some(At {
                pt: first,
                dir: [1.0, 0.0],
            });
        }
        while *seg + 2 < self.cum.len() && self.cum[*seg + 1] < t {
            *seg += 1;
        }
        let (a, b) = (self.pts[*seg], self.pts[*seg + 1]);
        let len = self.cum[*seg + 1] - self.cum[*seg];
        let along = (t - self.cum[*seg]).clamp(0.0, len);
        let (ux, uy) = if len > 0.0 {
            ((b.x - a.x) / len, (b.y - a.y) / len)
        } else {
            (1.0, 0.0)
        };
        let u = if len > 0.0 { along / len } else { 0.0 };
        Some(At {
            pt: Pt {
                x: a.x + along * ux,
                y: a.y + along * uy,
                p: a.p + (b.p - a.p) * u,
            },
            dir: [ux, uy],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> Pt {
        Pt { x, y, p: 1.0 }
    }

    #[test]
    fn straight_input_stays_one_exact_line() {
        let pts: Vec<Pt> = (0..=10)
            .map(|i| pt(10.5 + 4.0 * f64::from(i), 7.5))
            .collect();
        let s = spline(&pts);
        assert!(s.iter().all(|q| q.y == 7.5));
        assert_eq!(s.first().map(|q| q.x), Some(10.5));
        assert_eq!(s.last().map(|q| q.x), Some(50.5));
        assert_eq!(Polyline::new(s).len(), 40.0);
    }

    #[test]
    fn sparse_corner_is_cut_through_the_midpoints() {
        // Corner at (100, 0): the path goes (0,0) → (50,0), curves to (100,50), → (100,100).
        let s = spline(&[pt(0.0, 0.0), pt(100.0, 0.0), pt(100.0, 100.0)]);
        assert_eq!((s[1].x, s[1].y), (50.0, 0.0));
        let apex = s.iter().map(|q| q.x + -q.y).fold(f64::MIN, f64::max);
        assert!((apex - 75.0).abs() < 1e-9, "{apex}"); // Bézier midpoint (87.5, 12.5)
    }

    #[test]
    fn stabilization_keeps_ends_and_smooths() {
        let zig: Vec<Pt> = (0..20)
            .map(|i| pt(f64::from(i), if i % 2 == 0 { 0.0 } else { 4.0 }))
            .collect();
        let s = stabilize(&zig, 3);
        assert_eq!((s[0].x, s[0].y), (0.0, 0.0));
        let last = s[s.len() - 1];
        assert_eq!((last.x, last.y), (19.0, 4.0));
        assert!(s[10].y > 1.0 && s[10].y < 3.0);
    }
}
