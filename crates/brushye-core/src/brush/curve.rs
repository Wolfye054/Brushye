//! Pressure response curves (specs B-004, B-006).
//!
//! The curve editor's points are **control points**, not interpolation points (B-004): with one
//! interior point the curve is the quadratic Bézier (0,0) → p → (1,1), measured exact to 0.0044.
//! Curves with two or more interior points are not observed yet; they use the quadratic
//! B-spline through the same control points (piecewise quadratic Béziers joined at the
//! midpoints between interior points). With one interior point that is the same Bézier.
//!
//! Only `+ - * / sqrt` are used, so results are identical on every platform (D-012).

/// A response curve from (0,0) to (1,1) shaped by interior control points.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Curve {
    /// Interior control points (x, y), x strictly increasing inside (0, 1), y in [0, 1].
    points: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurveError(pub String);

impl std::fmt::Display for CurveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pressure curve: {}", self.0)
    }
}

impl std::error::Error for CurveError {}

/// Most interior points accepted.
pub const MAX_POINTS: usize = 32;

impl Curve {
    /// The identity curve (no interior points).
    pub fn linear() -> Self {
        Self::default()
    }

    pub fn new(points: Vec<[f64; 2]>) -> Result<Self, CurveError> {
        if points.len() > MAX_POINTS {
            return Err(CurveError(format!("at most {MAX_POINTS} points")));
        }
        let mut last = 0.0;
        for (i, p) in points.iter().enumerate() {
            if !(p[0] > last && p[0] < 1.0) {
                return Err(CurveError(format!(
                    "point {i}: x must increase strictly inside (0, 1)"
                )));
            }
            if !(0.0..=1.0).contains(&p[1]) {
                return Err(CurveError(format!("point {i}: y must be in [0, 1]")));
            }
            last = p[0];
        }
        Ok(Self { points })
    }

    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
    }

    pub fn is_linear(&self) -> bool {
        self.points.is_empty()
    }

    /// Output for input `x` (clamped to [0, 1]).
    pub fn eval(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        if self.points.is_empty() {
            return x;
        }
        // Control polygon including the fixed end points.
        let n = self.points.len();
        let ctrl = |i: usize| -> [f64; 2] {
            if i == 0 {
                [0.0, 0.0]
            } else if i == n + 1 {
                [1.0, 1.0]
            } else {
                self.points[i - 1]
            }
        };
        let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        // Segment j (1..=n) is the Bézier (start, ctrl(j), end).
        for j in 1..=n {
            let start = if j == 1 {
                ctrl(0)
            } else {
                mid(ctrl(j - 1), ctrl(j))
            };
            let end = if j == n {
                ctrl(n + 1)
            } else {
                mid(ctrl(j), ctrl(j + 1))
            };
            if x <= end[0] || j == n {
                return bezier_y_at_x(start, ctrl(j), end, x).clamp(0.0, 1.0);
            }
        }
        x
    }
}

/// y of the quadratic Bézier (a, b, c) at abscissa `x`, for x-monotone control points.
fn bezier_y_at_x(a: [f64; 2], b: [f64; 2], c: [f64; 2], x: f64) -> f64 {
    // x(u) = A u² + B u + a.x, with A = a.x − 2 b.x + c.x, B = 2 (b.x − a.x).
    let qa = a[0] - 2.0 * b[0] + c[0];
    let qb = 2.0 * (b[0] - a[0]);
    let dx = x - a[0];
    // Stable root of A u² + B u − dx = 0: u = 2 dx / (B + sqrt(B² + 4 A dx)).
    let disc = (qb * qb + 4.0 * qa * dx).max(0.0);
    let den = qb + disc.sqrt();
    let u = if den > 0.0 {
        (2.0 * dx / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let v = 1.0 - u;
    v * v * a[1] + 2.0 * u * v * b[1] + u * u * c[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_is_identity() {
        let c = Curve::linear();
        for i in 0..=10 {
            let x = f64::from(i) / 10.0;
            assert_eq!(c.eval(x), x);
        }
    }

    #[test]
    fn one_point_matches_observed_bezier() {
        // B-004 run 3: point (0.5, 0.25) gives y = 0.5 P + 0.5 P².
        let c = Curve::new(vec![[0.5, 0.25]]).unwrap_or_else(|e| unreachable!("{e}"));
        for i in 0..=20 {
            let p = f64::from(i) / 20.0;
            let want = 0.5 * p + 0.5 * p * p;
            assert!((c.eval(p) - want).abs() < 1e-12, "{p}: {}", c.eval(p));
        }
    }

    #[test]
    fn multi_point_is_continuous_and_monotone_for_monotone_points() {
        let c = Curve::new(vec![[0.2, 0.05], [0.5, 0.6], [0.8, 0.7]])
            .unwrap_or_else(|e| unreachable!("{e}"));
        let mut last = 0.0;
        for i in 0..=1000 {
            let y = c.eval(f64::from(i) / 1000.0);
            assert!(y >= last - 1e-12 && y - last < 0.01, "{i}: {y}");
            last = y;
        }
        assert_eq!(c.eval(0.0), 0.0);
        assert!((c.eval(1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_points() {
        assert!(Curve::new(vec![[0.5, 0.2], [0.5, 0.3]]).is_err());
        assert!(Curve::new(vec![[0.0, 0.2]]).is_err());
        assert!(Curve::new(vec![[0.5, 1.2]]).is_err());
    }
}
