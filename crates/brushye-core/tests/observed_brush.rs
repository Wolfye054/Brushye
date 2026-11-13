//! Reference-image tests: the brush engine against CSP 1.13.2 exports (specs B-001, B-002,
//! B-003, B-009). Each test replays the observed setup and compares the alpha channel.
//!
//! Exact where the spec found an exact model. Where a rule is still open (the AA None edge
//! rule, the soft-tip falloff), pixels away from the open part must still be exact and the
//! rest are held to pinned tolerances, so any regression or improvement shows up.

mod common;

use brushye_core::brush::{self, Aa, BrushSettings, Dab, Gap};
use brushye_core::input::Sample;
use brushye_core::{Layer, LayerId, LayerKind};
use common::Img;

const W: u32 = 600;
const H: u32 = 400;

fn smp(x: f64, y: f64) -> Sample {
    Sample {
        x,
        y,
        t_us: 0,
        pressure: Some(1.0),
        tilt: None,
        twist: None,
        eraser: false,
        barrel: [false; 2],
    }
}

/// Points of a straight stroke sent as `n` evenly spaced motion events (b002_strokes.py).
fn line(x0: f64, y0: f64, x1: f64, y1: f64, n: u32) -> Vec<Sample> {
    (0..=n)
        .map(|i| {
            let f = f64::from(i) / f64::from(n);
            smp(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f)
        })
        .collect()
}

struct Stroke {
    s: BrushSettings,
    pts: Vec<Sample>,
}

fn render(strokes: &[Stroke]) -> Layer {
    let mut l = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for st in strokes {
        brush::stroke(&mut l, (W, H), &st.s, &st.pts).unwrap_or_else(|e| panic!("{e}"));
    }
    l
}

#[derive(Debug, Default)]
struct Stats {
    /// Pixels where either side has ink.
    total: usize,
    bad: usize,
    worst: u8,
    /// Same, over "robust" pixels only (no dab edge within MARGIN px).
    robust_total: usize,
    robust_bad: usize,
}

const MARGIN: f64 = 0.15;

/// Compare alpha channels. `edges` lists hard dab discs (centre, radius); pixels whose centre
/// lies within MARGIN of one of their edges are not "robust".
fn compare(l: &Layer, img: &Img, edges: &[(f64, f64, f64)]) -> Stats {
    compare_in(l, img, edges, |_, _| true)
}

/// [`compare`] restricted to pixels where `region(x, y)` holds.
fn compare_in(
    l: &Layer,
    img: &Img,
    edges: &[(f64, f64, f64)],
    region: impl Fn(u32, u32) -> bool,
) -> Stats {
    let mut st = Stats::default();
    for y in 0..H {
        for x in 0..W {
            if !region(x, y) {
                continue;
            }
            let got = l.pixel(x, y)[3];
            let want = img.at(x as usize, y as usize)[3];
            if got == 0 && want == 0 {
                continue;
            }
            let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let robust = edges.iter().all(|&(cx, cy, r)| {
                if (px - cx).abs() > r + 1.0 || (py - cy).abs() > r + 1.0 {
                    return true;
                }
                let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                (d - r).abs() > MARGIN
            });
            st.total += 1;
            if robust {
                st.robust_total += 1;
            }
            if got != want {
                st.bad += 1;
                st.worst = st.worst.max(got.abs_diff(want));
                if robust {
                    st.robust_bad += 1;
                    if std::env::var_os("BRUSH_DEBUG").is_some() {
                        eprintln!("robust miss ({x},{y}) got {got} want {want}");
                    }
                }
            }
        }
    }
    st
}

fn hard(size: f64, aa: Aa) -> BrushSettings {
    BrushSettings {
        size,
        aa,
        ..Default::default()
    }
}

fn edges_of(strokes: &[Stroke]) -> Vec<(f64, f64, f64)> {
    strokes
        .iter()
        .flat_map(|st| {
            brush::place_dabs(&st.s, &st.pts)
                .into_iter()
                .map(|d: Dab| (d.x, d.y, d.size / 2.0))
        })
        .collect()
}

// ---------------------------------------------------------------- B-001: single dabs

const B001_SIZES: [f64; 8] = [0.7, 1.0, 1.5, 2.0, 3.0, 4.5, 10.0, 25.3];
/// Pen x (as resolved by the virtual pen) and y offset within the row.
const B001_COLS: [(f64, f64); 3] = [(60.5, 0.5), (200.8, 0.5), (341.0, 1.0)];

fn b001(aa: Aa) -> (Vec<Stroke>, Stats) {
    let level = [Aa::None, Aa::Weak, Aa::Middle, Aa::Strong]
        .iter()
        .position(|&a| a == aa)
        .unwrap_or(0);
    let mut strokes = Vec::new();
    for (r, &size) in B001_SIZES.iter().enumerate() {
        for &(cx, dy) in &B001_COLS {
            let cy = 25.0 + 46.0 * r as f64 + dy;
            strokes.push(Stroke {
                s: hard(size, aa),
                pts: vec![smp(cx, cy)],
            });
        }
    }
    let img = Img::load_spec("brush", &format!("B001_aa{level}.png"));
    let st = compare(&render(&strokes), &img, &[]);
    (strokes, st)
}

#[test]
fn b001_aa_weak_and_middle_are_exact() {
    for aa in [Aa::Weak, Aa::Middle] {
        let (_, st) = b001(aa);
        assert_eq!(st.bad, 0, "{aa:?}: {st:?}");
        assert!(st.total > 1800, "{aa:?}: {st:?}");
    }
}

#[test]
fn b001_aa_strong_within_one_level() {
    // Spec: 7 px off by one, all at size 25.3.
    let (_, st) = b001(Aa::Strong);
    assert!(st.bad <= 7 && st.worst <= 1, "{st:?}");
}

#[test]
fn b001_aa_none_exact_away_from_the_edge() {
    let (strokes, _) = b001(Aa::None);
    let img = Img::load_spec("brush", "B001_aa0.png");
    let st = compare(&render(&strokes), &img, &edges_of(&strokes));
    // Open edge-tie rule (B-001): spec measured 24 wrong pixels of 1860 with the closed disc.
    // One of them is 0.21 px outside the edge: size 1 at a pixel corner lights one pixel.
    assert!(st.robust_bad <= 1 && st.bad <= 24, "{st:?}");
}

// ---------------------------------------------------------------- B-002: spacing

fn b002_main(gap: Gap) -> Vec<Stroke> {
    let s = |size| BrushSettings {
        size,
        aa: Aa::None,
        density: 0.3,
        gap,
        ..Default::default()
    };
    vec![
        Stroke {
            s: s(40.0),
            pts: line(40.5, 40.5, 472.5, 40.5, 200),
        },
        Stroke {
            s: s(40.0),
            pts: line(40.5, 120.5, 472.5, 120.5, 200),
        },
        Stroke {
            s: s(40.0),
            pts: line(40.5, 200.5, 472.5, 200.5, 50),
        },
        Stroke {
            s: s(10.0),
            pts: line(40.5, 270.5, 472.5, 270.5, 200),
        },
        Stroke {
            s: s(10.0),
            pts: line(40.5, 320.5, 472.5, 320.5, 50),
        },
    ]
}

fn b002_sweep(gap: Gap) -> Vec<Stroke> {
    let rows = [
        (2.0, 15.5),
        (4.0, 35.5),
        (7.0, 60.5),
        (16.0, 95.5),
        (25.0, 145.5),
        (63.0, 230.5),
        (100.0, 340.5),
    ];
    rows.iter()
        .map(|&(size, y)| Stroke {
            s: BrushSettings {
                size,
                aa: Aa::None,
                density: 0.3,
                gap,
                ..Default::default()
            },
            pts: line(60.5, y, 472.5, y, 200),
        })
        .collect()
}

#[test]
fn b002_spacing_exact_away_from_dab_edges() {
    let cases: [(&str, Vec<Stroke>, usize); 8] = [
        ("B002_narrow.png", b002_main(Gap::Narrow), 0),
        ("B002_normal.png", b002_main(Gap::Normal), 0),
        ("B002_wide.png", b002_main(Gap::Wide), 0),
        ("B002_fixed10.png", b002_main(Gap::Fixed(10.0)), 0),
        ("B002_sweep_narrow.png", b002_sweep(Gap::Narrow), 0),
        ("B002_sweep_normal.png", b002_sweep(Gap::Normal), 0),
        ("B002_sweep_wide.png", b002_sweep(Gap::Wide), 0),
        ("B002_sweep_fixed37.png", b002_sweep(Gap::Fixed(37.0)), 0),
    ];
    let mut report = Vec::new();
    for (file, strokes, _) in &cases {
        let img = Img::load_spec("brush", file);
        let st = compare(&render(strokes), &img, &edges_of(strokes));
        report.push(format!("{file}: {st:?}"));
        assert_eq!(st.robust_bad, 0, "{file}: {st:?}");
    }
    eprintln!("{}", report.join("\n"));
}

// ---------------------------------------------------------------- B-003: build-up

fn b003(gap: Gap, opacity: f64, density: f64) -> Vec<Stroke> {
    let s = BrushSettings {
        size: 40.0,
        aa: Aa::None,
        opacity,
        density,
        gap,
        ..Default::default()
    };
    let p = [
        (330.5, 200.5),
        (570.5, 200.5),
        (570.5, 350.5),
        (450.5, 350.5),
        (450.5, 40.5),
    ];
    vec![
        Stroke {
            s: s.clone(),
            pts: line(40.5, 100.5, 280.5, 100.5, 120),
        },
        Stroke {
            s: s.clone(),
            pts: line(160.5, 30.5, 160.5, 370.5, 170),
        },
        Stroke {
            s,
            pts: p.iter().map(|&(x, y)| smp(x, y)).collect(),
        },
    ]
}

#[test]
fn b003_accumulation_exact_away_from_dab_edges() {
    let cases = [
        // (file, gap, opacity, density, pinned P-region misses away from dab edges)
        ("B003_narrow_op50_den100.png", Gap::Narrow, 0.5, 1.0, 13),
        ("B003_narrow_op100_den50.png", Gap::Narrow, 1.0, 0.5, 174),
        ("B003_narrow_op50_den50.png", Gap::Narrow, 0.5, 0.5, 152),
        (
            "B003_fixed90_op50_den50.png",
            Gap::Fixed(90.0),
            0.5,
            0.5,
            833,
        ),
    ];
    let mut report = Vec::new();
    for (file, gap, o, d, p_limit) in cases {
        let strokes = b003(gap, o, d);
        let img = Img::load_spec("brush", file);
        let layer = render(&strokes);
        let edges = edges_of(&strokes);
        // H and V (x < 300): straight strokes, exact away from dab edges.
        let st = compare_in(&layer, &img, &edges, |x, _| x < 300);
        report.push(format!("{file} H+V: {st:?}"));
        assert_eq!(st.robust_bad, 0, "{file}: {st:?}");
        // P (x >= 300), a 5-point polyline: CSP leaves its corners and its end short of the
        // input polyline (B-002 open question: spacing along curves). Pinned from the first run.
        let sp = compare_in(&layer, &img, &edges, |x, _| x >= 300);
        report.push(format!("{file} P: {sp:?}"));
        assert!(sp.robust_bad <= p_limit, "{file} P: {sp:?}");
    }
    eprintln!("{}", report.join("\n"));
}

// ---------------------------------------------------------------- B-009: hardness (interim)

#[test]
fn b009_soft_tips_within_interim_tolerance() {
    for h in 1..=5u8 {
        let strokes = [Stroke {
            s: BrushSettings {
                size: 101.0,
                aa: Aa::None,
                hardness: h,
                ..Default::default()
            },
            pts: vec![smp(300.5, 200.5)],
        }];
        let img = Img::load_spec("brush", &format!("B009_h{h}_aa0.png"));
        let st = compare(&render(&strokes), &img, &[]);
        eprintln!("h{h}: {st:?}");
        // Interim table model, fitted at this size (B-009 open): pinned from the first run.
        // Other sizes are not observed yet.
        let (bad_max, worst_max) = match h {
            1 | 2 => (usize::MAX, 1),
            3 => (usize::MAX, 2),
            4 => (usize::MAX, 16),
            // Hard tip, AA None: open edge-tie rule only.
            _ => (12, 255),
        };
        assert!(st.bad <= bad_max && st.worst <= worst_max, "h{h}: {st:?}");
    }
}

// ---------------------------------------------------------------- B-004/B-005: pressure

/// The pen_grid.py layout on a 1200×900 white canvas: 21 constant-pressure strokes, P = 0,
/// 0.05 … 1.0. Pressure reaches CSP in 1/2047 steps (B-004 input definition).
fn pressure_grid(s: &BrushSettings) -> Vec<(Stroke, (u32, u32))> {
    (0..21u32)
        .map(|i| {
            let p_target = f64::from(i) * 0.05;
            let p = ((p_target * 2047.0).round() / 2047.0) as f32;
            let (c, r) = (i % 7, i / 7);
            let x0 = f64::from(20 + c * 170 + 25);
            let y = f64::from(150 + r * 300);
            let mut pts = vec![Sample {
                pressure: Some(0.0),
                ..smp(x0, y)
            }];
            pts.push(Sample {
                pressure: Some(p),
                ..smp(x0 + 0.5, y)
            });
            for k in 1..=60 {
                pts.push(Sample {
                    pressure: Some(p),
                    ..smp(x0 + 2.0 * f64::from(k), y)
                });
            }
            (Stroke { s: s.clone(), pts }, ((x0 + 60.0) as u32, y as u32))
        })
        .collect()
}

/// Max |ink difference| over each stroke's centre column (ink = 255 − red on white paper).
fn pressure_column_error(file: &str, s: &BrushSettings) -> u8 {
    use brushye_core::{Document, Rgba8};
    let (w, h) = (1200u32, 900u32);
    let grid = pressure_grid(s);
    let mut layer = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for (st, _) in &grid {
        brush::stroke(&mut layer, (w, h), &st.s, &st.pts).unwrap_or_else(|e| panic!("{e}"));
    }
    let mut doc = Document::new(w, h, 72, Some(Rgba8::WHITE)).unwrap_or_else(|e| panic!("{e}"));
    doc.add_layer(layer, None).unwrap_or_else(|e| panic!("{e}"));
    let out = brushye_core::render::flatten(&doc);
    let img = Img::load_spec("brush", file);
    let mut worst = 0u8;
    for (_, (cx, cy)) in &grid {
        for y in cy - 60..cy + 60 {
            let got = out.pixel(*cx, y)[0];
            let want = img.at(*cx as usize, y as usize)[0];
            if std::env::var_os("BRUSH_DEBUG").is_some() && got.abs_diff(want) > 2 {
                eprintln!(
                    "{file} ({cx},{y}) dy {} got {got} want {want}",
                    i64::from(y) - i64::from(*cy)
                );
            }
            worst = worst.max(got.abs_diff(want));
        }
    }
    worst
}

#[test]
fn b004_b005_pressure_dynamics_match_within_tolerance() {
    use brushye_core::brush::{Curve, Dynamics};
    let dynm = |min: f64, curve: Curve| Some(Dynamics { min, curve });
    let size100 = |d| BrushSettings {
        size: 100.0,
        size_pressure: d,
        ..Default::default()
    };
    let one_pt = Curve::new(vec![[0.5, 0.25]]).unwrap_or_else(|e| panic!("{e}"));
    let cases = [
        (
            "B004_min0_linear.png",
            size100(dynm(0.0, Curve::linear())),
            9,
        ),
        (
            "B004_min20_linear.png",
            size100(dynm(0.2, Curve::linear())),
            13,
        ),
        ("B004_curve1pt.png", size100(dynm(0.0, one_pt)), 11),
        (
            "B005_opacity_min0.png",
            BrushSettings {
                size: 40.0,
                opacity_pressure: dynm(0.0, Curve::linear()),
                ..Default::default()
            },
            3,
        ),
        (
            "B005_density_min0.png",
            BrushSettings {
                size: 40.0,
                density_pressure: dynm(0.0, Curve::linear()),
                ..Default::default()
            },
            3,
        ),
    ];
    // Pinned from the first run. Size errors sit only on the outermost anti-aliased edge
    // pixel (about 0.05 px of diameter: pressure reaches CSP only to ±0.5/2047, B-004).
    for (file, s, limit) in cases {
        let e = pressure_column_error(file, &s);
        eprintln!("{file}: max centre-column error {e}");
        assert!(e <= limit, "{file}: {e} > {limit}");
    }
}
