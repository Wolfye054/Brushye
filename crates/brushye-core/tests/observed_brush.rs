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

/// Polyline corners with extra events every `step` px along each segment (0 = corners only).
fn densify(p: &[(f64, f64)], step: f64) -> Vec<Sample> {
    let mut out = vec![smp(p[0].0, p[0].1)];
    for w in p.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let n = if step > 0.0 { (len / step) as u32 } else { 1 }.max(1);
        for i in 1..=n {
            let f = f64::from(i) / f64::from(n);
            out.push(smp(x0 + (x1 - x0) * f, y0 + (y1 - y0) * f));
        }
    }
    out
}

struct Stroke {
    s: BrushSettings,
    pts: Vec<Sample>,
}

fn render(strokes: &[Stroke]) -> Layer {
    render_on(
        Layer::new(LayerId(1), "t", LayerKind::Raster),
        strokes,
        (W, H),
    )
}

fn render_on(mut l: Layer, strokes: &[Stroke], bounds: (u32, u32)) -> Layer {
    for st in strokes {
        brush::stroke(&mut l, bounds, &st.s, &st.pts).unwrap_or_else(|e| panic!("{e}"));
    }
    l
}

/// Render on an empty layer the size of `img`.
fn render_like(strokes: &[Stroke], img: &Img) -> Layer {
    render_on(
        Layer::new(LayerId(1), "t", LayerKind::Raster),
        strokes,
        (img.w as u32, img.h as u32),
    )
}

/// Position as resolved by the virtual pen (0.1 px steps, Python's round-half-even).
fn q01(v: f64) -> f64 {
    (v * 10.0).round_ties_even() / 10.0
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
    for y in 0..img.h as u32 {
        for x in 0..img.w as u32 {
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
    eprintln!("B001_aa0: {st:?}");
    // Scanline rule (B-001 run 2): 4 px of 1858 wrong (was 24 with a closed disc); the rest
    // are exact ties (open).
    assert!(st.bad <= 4, "{st:?}");
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
            // Events every 2 px along the polyline (fits exactly; corners only does not).
            pts: densify(&p, 2.0),
        },
    ]
}

#[test]
fn b003_accumulation_exact_away_from_dab_edges() {
    let cases = [
        // (file, gap, opacity, density)
        ("B003_narrow_op50_den100.png", Gap::Narrow, 0.5, 1.0),
        ("B003_narrow_op100_den50.png", Gap::Narrow, 1.0, 0.5),
        ("B003_narrow_op50_den50.png", Gap::Narrow, 0.5, 0.5),
        ("B003_fixed90_op50_den50.png", Gap::Fixed(90.0), 0.5, 0.5),
    ];
    let mut report = Vec::new();
    for (file, gap, o, d) in cases {
        let strokes = b003(gap, o, d);
        let img = Img::load_spec("brush", file);
        let layer = render(&strokes);
        let edges = edges_of(&strokes);
        // H and V (x < 300): straight strokes, exact away from dab edges.
        let st = compare_in(&layer, &img, &edges, |x, _| x < 300);
        report.push(format!("{file} H+V: {st:?}"));
        assert_eq!(st.robust_bad, 0, "{file}: {st:?}");
        // P (x >= 300): one stroke along a polyline, events every 2 px, smoothed by the
        // B-spline (B-002 run 2): exact away from dab edges too, corners included.
        let sp = compare_in(&layer, &img, &edges, |x, _| x >= 300);
        report.push(format!("{file} P: {sp:?}"));
        assert_eq!(sp.robust_bad, 0, "{file} P: {sp:?}");
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
            // Hard tip, AA None: exact with the scanline rule.
            _ => (0, 0),
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

// ================================================================ run 2 (800×800 canvas)

fn print_pin(name: &str, st: &Stats) {
    eprintln!("{name}: {st:?}");
}

fn none_hard(size: f64) -> BrushSettings {
    BrushSettings {
        size,
        aa: Aa::None,
        ..Default::default()
    }
}

/// B-001 run 2: 10×10 single dabs per size at every 0.1 px offset (brush_run2.py sweep).
fn b001_sweep(size: u32, aa: Aa) -> Vec<Stroke> {
    let (ox, pitch) = match size {
        63 => (40.0, 75.0),
        16 => (20.0, 25.0),
        5 => (20.0, 14.0),
        3 => (20.0, 12.0),
        _ => (20.0, 10.0),
    };
    let mut out = Vec::new();
    for j in 0..10 {
        for i in 0..10 {
            let (fi, fj) = (f64::from(i), f64::from(j));
            out.push(Stroke {
                s: BrushSettings {
                    aa,
                    ..none_hard(f64::from(size))
                },
                pts: vec![smp(
                    ox + pitch * fi + fi / 10.0,
                    ox + pitch * fj + fj / 10.0,
                )],
            });
        }
    }
    out
}

#[test]
fn b001_run2_scanline_edge_rule() {
    // (size, AA, pinned wrong px). AA None misses are exact ties (B-001 run 2, open).
    let cases = [
        (1, Aa::None, "B001_sweep_s1.png", 16),
        (2, Aa::None, "B001_sweep_s2.png", 9),
        (3, Aa::None, "B001_sweep_s3.png", 13),
        (5, Aa::None, "B001_sweep_s5.png", 20),
        (16, Aa::None, "B001_sweep_s16.png", 11),
        (63, Aa::None, "B001_sweep_s63.png", 15),
        // AA Weak: 100 dabs, off by 1 only.
        (16, Aa::Weak, "B001_sweep_aa1_s16.png", 59),
    ];
    for (size, aa, file, pin) in cases {
        let img = Img::load_spec("brush", file);
        let strokes = b001_sweep(size, aa);
        let st = compare(&render_like(&strokes, &img), &img, &[]);
        print_pin(file, &st);
        assert!(st.bad <= pin, "{file}: {st:?}");
        if aa == Aa::Weak {
            assert!(st.worst <= 1, "{file}: {st:?}");
        }
    }
}

/// Run-2 stroke with pen positions resolved to 0.1 px.
fn pen(pts: impl IntoIterator<Item = (f64, f64)>) -> Vec<Sample> {
    pts.into_iter().map(|(x, y)| smp(q01(x), q01(y))).collect()
}

fn arc_points(n: u32) -> Vec<(f64, f64)> {
    let (cx, cy, r) = (300.5, 330.5, 150.0);
    (0..=n)
        .map(|i| {
            let a = std::f64::consts::PI * f64::from(i) / f64::from(n);
            (cx - r * a.cos(), cy - r * a.sin())
        })
        .collect()
}

const POLY: [(f64, f64); 5] = [
    (60.5, 60.5),
    (300.5, 60.5),
    (300.5, 300.5),
    (540.5, 300.5),
    (420.5, 120.5),
];

fn poly_dense() -> Vec<(f64, f64)> {
    let mut out = vec![POLY[0]];
    for w in POLY.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        let n = ((x1 - x0).hypot(y1 - y0) / 2.0) as u32;
        for i in 1..=n {
            let f = f64::from(i) / f64::from(n);
            out.push((x0 + (x1 - x0) * f, y0 + (y1 - y0) * f));
        }
    }
    out
}

#[test]
fn b002_run2_paths_follow_the_b_spline() {
    let s = BrushSettings {
        gap: Gap::Fixed(300.0),
        ..none_hard(6.0)
    };
    // Pinned wrong px (whole 6 px dabs move if the path is wrong; these are edge pixels).
    type Case = (&'static str, Vec<(f64, f64)>, usize);
    let cases: [Case; 4] = [
        ("B002_arc_200.png", arc_points(200), 1),
        ("B002_arc_12.png", arc_points(12), 4),
        ("B002_poly_1.png", poly_dense(), 15),
        ("B002_poly_0.png", POLY.to_vec(), 4),
    ];
    for (file, pts, pin) in cases {
        let img = Img::load_spec("brush", file);
        let strokes = [Stroke {
            s: s.clone(),
            pts: pen(pts),
        }];
        let st = compare(&render_like(&strokes, &img), &img, &[]);
        print_pin(file, &st);
        assert!(st.bad <= pin, "{file}: {st:?}");
    }
}

/// The three ROWS strokes of brush_run2.py at one setting.
fn rows(s: &BrushSettings) -> Vec<Stroke> {
    [(80.5, 260), (200.5, 260), (320.5, 52)]
        .iter()
        .map(|&(y, n)| Stroke {
            s: s.clone(),
            pts: line(40.5, y, 560.5, y, n),
        })
        .collect()
}

#[test]
fn b002_run2_options() {
    let base = BrushSettings {
        density: 0.3,
        ..none_hard(40.0)
    };
    let adj = |gap, density| BrushSettings {
        gap,
        density,
        density_by_gap: true,
        ..base.clone()
    };
    let cases = [
        ("B002_rows_base.png", base.clone()),
        // "Correct velocity input" OFF: byte-identical to the base (B-002 run 2).
        ("B002_rows_novelocity.png", base.clone()),
        ("B002_rows_adjustgap_narrow.png", adj(Gap::Narrow, 0.3)),
        ("B002_rows_adjustgap.png", adj(Gap::Normal, 0.3)),
        ("B002_rows_adjustgap_wide.png", adj(Gap::Wide, 0.3)),
        ("B002_rows_adjustgap_d60.png", adj(Gap::Normal, 0.6)),
    ];
    for (file, s) in cases {
        let img = Img::load_spec("brush", file);
        let strokes = rows(&s);
        let st = compare(&render_like(&strokes, &img), &img, &edges_of(&strokes));
        print_pin(file, &st);
        assert_eq!(st.robust_bad, 0, "{file}: {st:?}");
    }
    // Pressure ramp: the gap after a dab follows that dab's size.
    let img = Img::load_spec("brush", "B002_ramp.png");
    let s = BrushSettings {
        gap: Gap::Fixed(300.0),
        size_pressure: Some(brush::Dynamics::default()),
        ..none_hard(20.0)
    };
    let pts = (0..=260u32)
        .map(|i| {
            let f = f64::from(i) / 260.0;
            // brush_run2.py sends this ramp as raw vpen values; CSP receives
            // P = (p − 0.0099) / 0.9901 in 1/2047 steps (B-004 input definition).
            let raw = 0.2 + 0.8 * f;
            let p = (((raw - 0.0099) / 0.9901) * 2047.0).round() / 2047.0;
            Sample {
                pressure: Some(p as f32),
                ..smp(40.5 + 520.0 * f, 200.5)
            }
        })
        .collect();
    let l = render_like(&[Stroke { s, pts }], &img);
    let ours = dab_columns(|x, y| l.pixel(x, y)[3], img.w as u32, img.h as u32);
    let theirs = dab_columns(
        |x, y| img.at(x as usize, y as usize)[3],
        img.w as u32,
        img.h as u32,
    );
    let worst = ours
        .iter()
        .zip(&theirs)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    eprintln!(
        "B002_ramp.png: {} dabs (CSP {}), max centre error {worst:.2} px",
        ours.len(),
        theirs.len()
    );
    assert_eq!(ours.len(), theirs.len());
    assert!(worst <= 0.9, "{worst}");
}

/// x centroids of isolated dabs along a horizontal stroke (runs of inked columns).
fn dab_columns(get: impl Fn(u32, u32) -> u8, w: u32, h: u32) -> Vec<f64> {
    let col: Vec<f64> = (0..w)
        .map(|x| (0..h).map(|y| f64::from(get(x, y))).sum())
        .collect();
    let mut out = Vec::new();
    let (mut s, mut n) = (0.0, 0.0);
    for (x, &v) in col.iter().enumerate() {
        if v > 0.0 {
            s += v * (x as f64 + 0.5);
            n += v;
        } else if n > 0.0 {
            out.push(s / n);
            (s, n) = (0.0, 0.0);
        }
    }
    if n > 0.0 {
        out.push(s / n);
    }
    out
}

#[test]
fn b003_run2_layer_commit_is_exact() {
    let img = Img::load_spec("brush", "B003_layers_op10.png");
    let s = BrushSettings {
        opacity: 0.1,
        ..none_hard(40.0)
    };
    let strokes: Vec<Stroke> = (0..25u32)
        .map(|k| Stroke {
            s: s.clone(),
            pts: (0..=(520 - 20 * k) / 2)
                .map(|i| smp(40.5 + f64::from(20 * k + 2 * i), 200.5))
                .collect(),
        })
        .collect();
    let st = compare(&render_like(&strokes, &img), &img, &edges_of(&strokes));
    print_pin("B003_layers_op10.png", &st);
    assert_eq!(st.robust_bad, 0, "{st:?}");
}

#[test]
fn b004_run2_multi_point_curves() {
    use brushye_core::brush::{Curve, Dynamics};
    let cases = [
        ("B004_pgrid_linear.png", vec![]),
        ("B004_pgrid_2pt.png", vec![[0.25, 0.75], [0.75, 0.25]]),
        (
            "B004_pgrid_3pt.png",
            vec![[0.25, 0.75], [0.5, 0.25], [0.75, 0.25]],
        ),
    ];
    for (file, pts) in cases {
        let img = Img::load_spec("brush", file);
        let s = BrushSettings {
            size: 100.0,
            size_pressure: Some(Dynamics {
                min: 0.0,
                curve: Curve::new(pts).unwrap_or_else(|e| panic!("{e}")),
            }),
            ..Default::default()
        };
        let mut strokes = Vec::new();
        let mut cols = Vec::new();
        for i in 0..21u32 {
            let p = ((f64::from(i) / 20.0 * 2047.0).round() / 2047.0) as f32;
            let x0 = f64::from(20 + (i % 3) * 260 + 50);
            let y = f64::from(55 + (i / 3) * 110);
            let mut pts = vec![Sample {
                pressure: Some(0.0),
                ..smp(x0, y)
            }];
            pts.extend((0..=60).map(|k| Sample {
                pressure: Some(p),
                ..smp(
                    if k == 0 {
                        x0 + 0.5
                    } else {
                        x0 + 2.0 * f64::from(k)
                    },
                    y,
                )
            }));
            strokes.push(Stroke { s: s.clone(), pts });
            cols.push(((x0 + 60.0) as u32, y as u32));
        }
        let l = render_like(&strokes, &img);
        let mut worst = 0u8;
        for (cx, cy) in cols {
            for y in cy - 54..cy + 54 {
                worst = worst.max(l.pixel(cx, y)[3].abs_diff(img.at(cx as usize, y as usize)[3]));
            }
        }
        eprintln!("{file}: max centre-column error {worst}");
        // Outermost AA pixel only (pressure precision, B-004).
        let pin = match file {
            "B004_pgrid_linear.png" => 9,
            "B004_pgrid_2pt.png" => 16,
            _ => 19,
        };
        assert!(worst <= pin, "{file}: {worst}");
    }
}

/// Alpha-weighted centre y per column (columns with ink).
fn centre_line(get: impl Fn(u32, u32) -> u8, w: u32, h: u32) -> Vec<Option<f64>> {
    (0..w)
        .map(|x| {
            let (mut s, mut n) = (0.0, 0.0);
            for y in 0..h {
                let a = f64::from(get(x, y));
                s += a * (f64::from(y) + 0.5);
                n += a;
            }
            (n > 0.0).then(|| s / n)
        })
        .collect()
}

fn zigzag_points() -> Vec<(f64, f64)> {
    let (x0, y0, w, amp) = (50.5, 200.5, 50.0, 40.0);
    let corners: Vec<(f64, f64)> = (0..11)
        .map(|k| {
            let y = if k == 0 {
                y0
            } else if k % 2 == 1 {
                y0 + amp
            } else {
                y0 - amp
            };
            (x0 + w * f64::from(k), y)
        })
        .collect();
    let mut pts = vec![corners[0]];
    for c in corners.windows(2) {
        let ((xa, ya), (xb, yb)) = (c[0], c[1]);
        let n = ((xb - xa).hypot(yb - ya) / 2.0) as u32;
        for i in 1..=n {
            let f = f64::from(i) / f64::from(n);
            pts.push((xa + (xb - xa) * f, ya + (yb - ya) * f));
        }
    }
    pts
}

#[test]
fn b007_stabilization_moving_mean() {
    // (setting, file, pinned centre-line rms in px). 50 and 100 are open (B-007).
    for (st_level, file, pin) in [
        (0u8, "B007_zigzag_s0_ms5.png", 0.04),
        (6, "B007_zigzag_s6_ms5.png", 0.07),
        (20, "B007_zigzag_s20_ms5.png", 0.16),
        (50, "B007_zigzag_s50_ms5.png", 2.24),
        (100, "B007_zigzag_s100_ms5.png", 6.43),
    ] {
        let img = Img::load_spec("brush", file);
        let s = BrushSettings {
            gap: Gap::Narrow,
            stabilization: st_level,
            ..none_hard(4.0)
        };
        let l = render_like(
            &[Stroke {
                s,
                pts: pen(zigzag_points()),
            }],
            &img,
        );
        let (w, h) = (img.w as u32, img.h as u32);
        let ours = centre_line(|x, y| l.pixel(x, y)[3], w, h);
        let theirs = centre_line(|x, y| img.at(x as usize, y as usize)[3], w, h);
        let d: Vec<f64> = ours
            .iter()
            .zip(&theirs)
            .filter_map(|(a, b)| Some(a.as_ref()? - b.as_ref()?))
            .collect();
        let rms = (d.iter().map(|v| v * v).sum::<f64>() / d.len() as f64).sqrt();
        let max = d.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        eprintln!(
            "{file}: centre line rms {rms:.3} max {max:.3} over {} columns",
            d.len()
        );
        assert!(rms <= pin && d.len() > 500, "{file}: {rms}");
    }
}

#[test]
fn b008_ink_modes() {
    use brushye_core::brush::InkMode;
    let base = Img::load_spec("brush", "B008_base.png");
    let mut layer = Layer::new(LayerId(1), "t", LayerKind::Raster);
    let raw: Vec<u8> = base.px.iter().flatten().copied().collect();
    layer.put_pixels((256, 256), 0, 0, 256, 256, &raw);
    for op in [100u32, 50] {
        for (i, &m) in InkMode::ALL.iter().enumerate() {
            let file = format!("B008_op{op}_{i:02}_{}.png", m.name());
            let img = Img::load_spec("brush", &file);
            let s = BrushSettings {
                color: [64, 128, 192],
                opacity: f64::from(op) / 100.0,
                ink: m,
                ..none_hard(400.0)
            };
            let l = render_on(
                layer.clone(),
                &[Stroke {
                    s,
                    pts: vec![smp(128.5, 128.5)],
                }],
                (256, 256),
            );
            let (mut bad, mut worst) = (0usize, 0u8);
            for y in 0..256u32 {
                for x in 0..256u32 {
                    let (g, w) = (l.pixel(x, y), img.at(x as usize, y as usize));
                    // Colour of fully transparent pixels is not meaningful.
                    let n = if g[3] == 0 && w[3] == 0 { 0 } else { 4 };
                    let e = (0..n).map(|c| g[c].abs_diff(w[c])).max().unwrap_or(0);
                    if e > 0 {
                        bad += 1;
                        worst = worst.max(e);
                    }
                }
            }
            eprintln!("{file}: {bad} px wrong, max {worst}");
            // Pinned (B-008 open questions): (max wrong px, max error).
            use InkMode as M;
            let (pin_n, pin_e) = match (m, op) {
                (M::Background, 100) => (53, 1),
                (M::Background, _) => (7249, 1),
                (M::DarkerColor | M::LighterColor, _) => (2, 54),
                (M::Hue, _) => (18736, 3),
                (M::Saturation, _) => (601, 1),
                (M::Color, _) => (10441, 2),
                (M::Brightness, _) => (7556, 3),
                (M::Multiply, 50) => (54342, 2),
                (M::Normal | M::Darken | M::LinearBurn | M::Subtract, 50) => (300, 1),
                (M::HardLight | M::Difference | M::Exclusion, 50) => (377, 1),
                _ => (0, 0),
            };
            assert!(
                bad <= pin_n && worst <= pin_e,
                "{file}: {bad} px, max {worst}"
            );
        }
    }
}

#[test]
fn b009_run2_soft_tip_sizes() {
    let mut dabs: Vec<(f64, f64, f64)> = vec![(11.0, 40.5, 40.5), (41.0, 120.5, 60.5)];
    for k in 0..5 {
        let fk = f64::from(k);
        dabs.push((41.0, 60.5 + 60.0 * fk + fk / 10.0, 160.5));
        dabs.push((41.0, 60.5 + 60.0 * fk + (fk + 5.0) / 10.0, 260.5));
    }
    for k in 0..10 {
        let fk = f64::from(k);
        dabs.push((11.0, 40.5 + 30.0 * fk + fk / 10.0, 340.5));
    }
    for h in 1..=5u8 {
        for size in [11.0, 41.0] {
            let file = format!("B009_run2_h{h}_s{size}.png");
            let img = Img::load_spec("brush", &file);
            let strokes: Vec<Stroke> = dabs
                .iter()
                .filter(|d| d.0 == size)
                .map(|&(sz, x, y)| Stroke {
                    s: BrushSettings {
                        hardness: h,
                        ..none_hard(sz)
                    },
                    pts: vec![smp(x, y)],
                })
                .collect();
            let st = compare(&render_like(&strokes, &img), &img, &[]);
            print_pin(&file, &st);
            // Interim profile table scaled by R (B-009): pinned max error per level/size.
            let pin = match (h, size as u32) {
                (1, _) => 1,
                (2, 11) => 4,
                (2, _) | (3, 41) => 2,
                (3, _) => 15,
                (4, 41) => 97,
                _ => 255, // h4 at 11 flips single edge pixels; h5 is the hard tip
            };
            assert!(st.worst <= pin, "{file}: {st:?}");
            if h == 5 {
                assert!(st.bad <= 1, "{file}: {st:?}");
            }
        }
    }
}

#[test]
fn b010_thin_tips_and_direction_of_line() {
    use brushye_core::brush::TipAxis;
    let cases = [
        ("B010_tip_t30_a0_vert.png", 0.3, 0.0, TipAxis::Vertical),
        ("B010_tip_t30_a0_horiz.png", 0.3, 0.0, TipAxis::Horizontal),
        ("B010_tip_t30_a30_vert.png", 0.3, 30.0, TipAxis::Vertical),
        ("B010_tip_t50_a90_vert.png", 0.5, 90.0, TipAxis::Vertical),
    ];
    for (file, t, a, axis) in cases {
        let img = Img::load_spec("brush", file);
        let s = BrushSettings {
            thickness: t,
            angle: a,
            tip_axis: axis,
            ..none_hard(41.0)
        };
        let strokes: Vec<Stroke> = [(200.5, 200.5), (500.3, 200.7)]
            .iter()
            .map(|&(x, y)| Stroke {
                s: s.clone(),
                pts: vec![smp(x, y)],
            })
            .collect();
        let st = compare(&render_like(&strokes, &img), &img, &[]);
        print_pin(file, &st);
        // Spec: 4 edge ties in 3694 px.
        assert!(st.bad <= 4, "{file}: {st:?}");
    }
    let img = Img::load_spec("brush", "B010_dirs_line.png");
    let s = BrushSettings {
        thickness: 0.3,
        angle_follows_line: true,
        gap: Gap::Fixed(150.0),
        ..none_hard(41.0)
    };
    let strokes: Vec<Stroke> = (0..8)
        .map(|k| {
            let a = std::f64::consts::PI * f64::from(k) / 4.0;
            let (cx, cy) = (
                100.5 + 200.0 * f64::from(k % 4),
                200.5 + 400.0 * f64::from(k / 4),
            );
            let (sx, sy) = (cx - a.cos() * 60.0, cy - a.sin() * 60.0);
            Stroke {
                s: s.clone(),
                pts: pen((0..=60).map(|i| {
                    let t = f64::from(2 * i);
                    (sx + a.cos() * t, sy + a.sin() * t)
                })),
            }
        })
        .collect();
    let l = render_like(&strokes, &img);
    // Per stroke: CSP places the first dab ≈ 10 px late in strokes 4, 5 and 7 (B-010, open).
    for k in 0..8u32 {
        let (cx, cy) = (100 + 200 * (k % 4), 200 + 400 * (k / 4));
        let st = compare_in(&l, &img, &[], |x, y| {
            x.abs_diff(cx) < 100 && y.abs_diff(cy) < 100
        });
        print_pin(&format!("B010_dirs_line stroke {k}"), &st);
        let pin = if matches!(k, 4 | 5 | 7) { 525 } else { 4 };
        assert!(st.bad <= pin, "stroke {k}: {st:?}");
    }
}

#[test]
fn b006_global_curve_applies_before_the_setting_curve() {
    use brushye_core::brush::{Curve, Dynamics};
    // The wizard's curve, read from CSP's graph to about ±0.01 (B-006 observation 1).
    let global = Curve::new(vec![[0.08, 0.02], [0.24, 0.30], [0.41, 0.76], [0.50, 1.00]])
        .unwrap_or_else(|e| panic!("{e}"));
    let one_pt = Curve::new(vec![[0.5, 0.25]]).unwrap_or_else(|e| panic!("{e}"));
    let s = |sub: Curve, g: Curve| BrushSettings {
        size: 100.0,
        size_pressure: Some(Dynamics {
            min: 0.0,
            curve: sub,
        }),
        global_pressure: g,
        ..Default::default()
    };
    // Stroke widths (ink rows) for P = 0.05 … 0.45. Above 0.5 CSP's strokes are already full
    // size while a quadratic B-spline through these graph readings isn't: the wizard curve's
    // exact type is open (B-006).
    let worst = |file: &str, settings: &BrushSettings| {
        let r = pressure_rows(file, settings);
        r[1..10]
            .iter()
            .map(|&(a, b)| a.abs_diff(b))
            .max()
            .unwrap_or(0)
    };
    let a = worst(
        "B006_global_auto_grid.png",
        &s(Curve::linear(), global.clone()),
    );
    let b = worst(
        "B006_global_auto_plus_sub1pt.png",
        &s(one_pt.clone(), global.clone()),
    );
    eprintln!("rows off by at most {a} (global only), {b} (global then sub)");
    assert!(a <= 2 && b <= 2, "{a} {b}");
    // The other order, global(sub(P)), fits clearly worse: sub(global(P)) is CSP's order.
    let swapped = |p: f64| global.eval(one_pt.eval(p));
    let mine = |p: f64| one_pt.eval(global.eval(p));
    let gap = (1..10)
        .map(|i| (swapped(f64::from(i) * 0.05) - mine(f64::from(i) * 0.05)).abs())
        .fold(0.0f64, f64::max);
    assert!(gap > 0.1, "{gap}");
}

/// Ink rows in each stroke's centre column, ours vs CSP's, for the pressure grid.
fn pressure_rows(file: &str, s: &BrushSettings) -> Vec<(usize, usize)> {
    use brushye_core::{Document, Rgba8};
    let grid = pressure_grid(s);
    let mut layer = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for (st, _) in &grid {
        brush::stroke(&mut layer, (1200, 900), &st.s, &st.pts).unwrap_or_else(|e| panic!("{e}"));
    }
    let mut doc =
        Document::new(1200, 900, 72, Some(Rgba8::WHITE)).unwrap_or_else(|e| panic!("{e}"));
    doc.add_layer(layer, None).unwrap_or_else(|e| panic!("{e}"));
    let out = brushye_core::render::flatten(&doc);
    let img = Img::load_spec("brush", file);
    grid.iter()
        .map(|(_, (cx, cy))| {
            let n = |f: &dyn Fn(u32) -> u8| (cy - 60..cy + 60).filter(|&y| f(y) < 250).count();
            (
                n(&|y| out.pixel(*cx, y)[0]),
                n(&|y| img.at(*cx as usize, y as usize)[0]),
            )
        })
        .collect()
}
