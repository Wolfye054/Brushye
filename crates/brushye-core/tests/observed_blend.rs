//! The blend/compositing math against every observed CSP 1.13.2 export (DECISIONS D-017).
//!
//! Policy: cases whose behaviour is known must match **bit-exactly**; open cases carry a stated
//! tolerance (minimum exact fraction, maximum error) that may only ever be tightened. Each open
//! tolerance names the spec that will close it.

mod common;

use brushye_core::BlendMode;
use brushye_core::blend::math::{blend_rgb, composite, effective_alpha, over_opaque};
use common::Img;

#[derive(Default)]
struct Stats {
    n: u64,
    exact: u64,
    max: u8,
}

impl Stats {
    fn add(&mut self, got: &[u8], want: &[u8]) {
        for (g, w) in got.iter().zip(want) {
            self.n += 1;
            let d = g.abs_diff(*w);
            if d == 0 {
                self.exact += 1;
            }
            self.max = self.max.max(d);
        }
    }
    fn frac(&self) -> f64 {
        self.exact as f64 / self.n.max(1) as f64
    }
}

/// (min exact fraction, max error). `EXACT` = (1.0, 0).
type Tol = (f64, u8);
const EXACT: Tol = (1.0, 0);

fn check(case: &str, s: &Stats, tol: Tol, report: &mut Vec<String>) {
    report.push(format!(
        "{case:40} exact {:8.4}%  max {:3}",
        s.frac() * 100.0,
        s.max
    ));
    if s.frac() + 1e-12 < tol.0 || s.max > tol.1 {
        panic!(
            "{case}: exact {:.4}% (need ≥ {:.4}%), max {} (allowed {})",
            s.frac() * 100.0,
            tol.0 * 100.0,
            s.max,
            tol.1
        );
    }
}

fn mode_index(m: BlendMode) -> usize {
    BlendMode::ALL.iter().position(|x| *x == m).unwrap_or(0)
}

/// Stated tolerances for open cases (opaque layers, L-001 data). Everything else is exact.
fn tol_opaque(m: BlendMode) -> Tol {
    use BlendMode as M;
    match m {
        // L-007: ties at luma x.5 are open.
        M::DarkerColor => TOL_OPAQUE_PICK[0],
        M::LighterColor => TOL_OPAQUE_PICK[1],
        // L-007: the clip step and x.5 ties are open.
        M::Hue | M::Saturation | M::Color | M::Brightness => {
            TOL_OPAQUE_NONSEP[m as usize - M::Hue as usize]
        }
        _ => EXACT,
    }
}

// Pinned from the first measurement (2026-11-13); only ever tighten. Order: Hue, Saturation,
// Color, Brightness for the 4-arrays; NONSEP order for the 6-arrays.
const TOL_OPAQUE_PICK: [Tol; 2] = [(0.99998, 116), (0.99999, 79)]; // L-007 (x.5 luma ties)
const TOL_OPAQUE_NONSEP: [Tol; 4] = [(0.9596, 4), (0.9709, 6), (0.9569, 6), (0.9575, 7)]; // L-007
const TOL_ALPHA_MULTIPLY: Tol = (0.5780, 2); // L-002 open question 1
const TOL_ALPHA_NONSEP: [Tol; 6] = [
    (0.99998, 95),
    (0.99999, 25),
    (0.9799, 3),
    (0.9853, 6),
    (0.9784, 6),
    (0.9784, 4),
]; // L-007
const TOL_OPACITY_MULTIPLY: Tol = (0.4948, 2); // L-003 (Multiply with partial coverage)
const TOL_COMBO_NONSEP: [Tol; 6] = [
    (0.9977, 247),
    (0.9977, 251),
    (0.9284, 4),
    (0.9489, 6),
    (0.9601, 4),
    (0.9601, 4),
]; // L-007
const TOL_BD_NORMAL_COLOUR: Tol = (0.9962, 1); // L-005
/// L-005 colour over a semi-transparent backdrop, non-Normal modes (open), by mode index.
const TOL_BD_COLOUR: [Tol; 28] = [
    (0.9962, 1),
    (0.4870, 58),
    (0.5484, 6),
    (0.2402, 60),
    (0.1441, 191),
    (0.1701, 55),
    (0.4825, 56),
    (0.1465, 65),
    (0.1525, 64),
    (0.1353, 145),
    (0.1698, 56),
    (0.1239, 63),
    (0.3226, 29),
    (0.1894, 34),
    (0.1473, 68),
    (0.1697, 56),
    (0.2360, 55),
    (0.1732, 59),
    (0.4986, 57),
    (0.1709, 60),
    (0.2369, 56),
    (0.4870, 58),
    (0.4825, 56),
    (0.2325, 51),
    (0.7886, 5),
    (0.7886, 5),
    (0.7886, 5),
    (0.1808, 58),
];

const NONSEP: [BlendMode; 6] = [
    BlendMode::DarkerColor,
    BlendMode::LighterColor,
    BlendMode::Hue,
    BlendMode::Saturation,
    BlendMode::Color,
    BlendMode::Brightness,
];

fn nonsep_index(m: BlendMode) -> Option<usize> {
    NONSEP.iter().position(|x| *x == m)
}

#[test]
fn l001_opaque_all_modes() {
    let base = Img::load("inputs/base.png");
    let top = Img::load("inputs/top.png");
    let mut report = Vec::new();
    for &m in BlendMode::ALL {
        let obs = Img::load(&format!(
            "observed/opaque_{:02}_{}.png",
            mode_index(m),
            m.name()
        ));
        let mut s = Stats::default();
        for y in 0..obs.h {
            for x in 0..obs.w {
                let (b, t, o) = (base.at(x, y), top.at(x, y), obs.at(x, y));
                s.add(
                    &blend_rgb(m, [b[0], b[1], b[2]], [t[0], t[1], t[2]]),
                    &o[..3],
                );
            }
        }
        check(
            &format!("L-001 opaque {}", m.name()),
            &s,
            tol_opaque(m),
            &mut report,
        );
    }
    println!("{}", report.join("\n"));
}

#[test]
fn l002_source_alpha_all_modes() {
    let base = Img::load("inputs/base.png");
    let top = Img::load("inputs/top_alpha.png");
    let mut report = Vec::new();
    for &m in BlendMode::ALL {
        let obs = Img::load(&format!(
            "observed/alpha_{:02}_{}.png",
            mode_index(m),
            m.name()
        ));
        let mut s = Stats::default();
        for y in 0..obs.h {
            for x in 0..obs.w {
                let (b, t, o) = (base.at(x, y), top.at(x, y), obs.at(x, y));
                s.add(
                    &over_opaque(m, [b[0], b[1], b[2]], [t[0], t[1], t[2]], t[3]),
                    &o[..3],
                );
            }
        }
        let tol = match (m, nonsep_index(m)) {
            (BlendMode::Multiply, _) => TOL_ALPHA_MULTIPLY, // L-002 open question 1
            (_, Some(i)) => TOL_ALPHA_NONSEP[i],            // inherits L-007
            _ => EXACT,
        };
        check(&format!("L-002 alpha {}", m.name()), &s, tol, &mut report);
    }
    println!("{}", report.join("\n"));
}

/// Observed combo row → input row (L-003 crop: rows 0..191 then 512..1279).
fn combo_input_row(y: usize) -> usize {
    if y < 192 { y } else { y - 192 + 512 }
}

#[test]
fn l003_layer_opacity() {
    let base = Img::load("inputs2/combo_base.png");
    let top = Img::load("inputs2/combo_top.png");
    let mut report = Vec::new();
    for m in [
        BlendMode::Normal,
        BlendMode::Screen,
        BlendMode::Add,
        BlendMode::AddGlow,
        BlendMode::Multiply,
    ] {
        for op in [1u8, 37, 50, 99, 100] {
            let obs = Img::load(&format!("observed2/combo_{}_op{op}.png", m.name()));
            let mut s = Stats::default();
            for y in 0..obs.h {
                let iy = combo_input_row(y);
                for x in 0..obs.w {
                    let (b, t, o) = (base.at(x, iy), top.at(x, iy), obs.at(x, y));
                    s.add(
                        &over_opaque(
                            m,
                            [b[0], b[1], b[2]],
                            [t[0], t[1], t[2]],
                            effective_alpha(t[3], op),
                        ),
                        &o[..3],
                    );
                }
            }
            let tol = if m == BlendMode::Multiply {
                TOL_OPACITY_MULTIPLY
            } else {
                EXACT
            };
            check(&format!("L-003 {} {op}%", m.name()), &s, tol, &mut report);
        }
    }
    println!("{}", report.join("\n"));
}

#[test]
fn l007_nonseparable_designed_data() {
    let base = Img::load("inputs2/combo_base.png");
    let top = Img::load("inputs2/combo_top.png");
    let mut report = Vec::new();
    for (i, m) in NONSEP.into_iter().enumerate() {
        let obs = Img::load(&format!("observed2/combo_{}.png", m.name()));
        let mut s = Stats::default();
        for y in 0..obs.h {
            for x in 0..obs.w {
                let (b, t, o) = (base.at(x, y), top.at(x, y), obs.at(x, y));
                s.add(
                    &over_opaque(m, [b[0], b[1], b[2]], [t[0], t[1], t[2]], t[3]),
                    &o[..3],
                );
            }
        }
        check(
            &format!("L-007 {}", m.name()),
            &s,
            TOL_COMBO_NONSEP[i],
            &mut report,
        );
    }
    println!("{}", report.join("\n"));
}

#[test]
fn l005_semi_transparent_backdrop() {
    let base = Img::load("inputs2/bd_base.png");
    let top = Img::load("inputs2/bd_top.png");
    let mut report = Vec::new();
    for &m in BlendMode::ALL {
        let obs = Img::load(&format!(
            "observed2/bd_{:02}_{}.png",
            mode_index(m),
            m.name()
        ));
        let (mut alpha, mut colour) = (Stats::default(), Stats::default());
        for y in 0..obs.h {
            for x in 0..obs.w {
                let (b, t, o) = (base.at(x, y), top.at(x, y), obs.at(x, y));
                let r = composite(m, b, [t[0], t[1], t[2]], t[3]);
                alpha.add(&[r[3]], &[o[3]]);
                // Colour is undefined where the result is fully transparent.
                if o[3] > 0 {
                    colour.add(&r[..3], &o[..3]);
                }
            }
        }
        check(
            &format!("L-005 alpha {}", m.name()),
            &alpha,
            EXACT,
            &mut report,
        );
        let tol = if m == BlendMode::Normal {
            TOL_BD_NORMAL_COLOUR
        } else {
            TOL_BD_COLOUR[mode_index(m)]
        };
        check(
            &format!("L-005 colour {}", m.name()),
            &colour,
            tol,
            &mut report,
        );
    }
    println!("{}", report.join("\n"));
}
