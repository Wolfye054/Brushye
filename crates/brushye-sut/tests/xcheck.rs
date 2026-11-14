//! M8 exit check: strokes drawn with imported sub tools against CSP 1.13.2 strokes from the
//! same sub tools (`specs/behavior/brush/B-012-sut-cross-check.md`).
//!
//! Results are pinned (B-012): hard tips match closely; soft-tip strokes and anti-aliased thin
//! tips are known engine gaps. A pin that's beaten should be tightened.
//!
//! - `xcheck_presets.txt` holds what the importer makes of each X-xx.sut. With
//!   `BRUSHYE_XCHECK_SUT` set to the samples folder (outside the repo, D-020) the test checks
//!   the importer still produces exactly that.
//! - The strokes are `brush_run2.py xcheck`, rebuilt here. CSP's exports are in
//!   `specs/behavior/brush/data/xcheck/X-xx.png`; the alpha channels are compared.

use std::path::{Path, PathBuf};

use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::brush;
use brushye_command::core::input::Sample;
use brushye_command::core::{Layer, LayerId, LayerKind};

const PRESETS: &str = include_str!("xcheck_presets.txt");
const W: u32 = 800;
const H: u32 = 600;

fn data(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/behavior/brush/data/xcheck")
        .join(file)
}

fn presets() -> Vec<(String, BrushPreset)> {
    PRESETS
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (id, text) = l.split_once('\t').unwrap_or_else(|| unreachable!("{l}"));
            let p = BrushPreset::from_text(text).unwrap_or_else(|e| unreachable!("{id}: {e}"));
            (id.to_owned(), p)
        })
        .collect()
}

/// Position as resolved by the virtual pen (0.1 px steps, Python's round-half-even).
fn q01(v: f64) -> f64 {
    (v * 10.0).round_ties_even() / 10.0
}

/// CSP pressure for a target P: brush_run2.py sends round(vpen_p(P), 5); CSP receives
/// (p − 0.0099) / 0.9901 in 1/2047 steps (B-004 input definition).
fn csp_pressure(p_target: f64) -> f32 {
    let raw = if p_target <= 0.0 {
        0.0
    } else {
        (0.0099 + 0.9901 * p_target).min(1.0)
    };
    let raw = (raw * 1e5).round() / 1e5;
    ((((raw - 0.0099) / 0.9901) * 2047.0).round() / 2047.0).clamp(0.0, 1.0) as f32
}

fn sample(x: f64, y: f64, p: f64, t_us: u32) -> Sample {
    Sample {
        x: q01(x),
        y: q01(y),
        t_us,
        pressure: Some(csp_pressure(p)),
        tilt: None,
        twist: None,
        eraser: false,
        barrel: [false; 2],
    }
}

/// brush_run2.py `xcheck_strokes()` and `XCHECK_DAB`, events 5 ms apart.
fn strokes() -> Vec<Vec<Sample>> {
    let xs: Vec<f64> = (0..341).map(|i| 60.5 + 2.0 * f64::from(i)).collect();
    let n = (xs.len() - 1) as f64;
    let mk = |f: &dyn Fn(usize, f64) -> (f64, f64)| -> Vec<Sample> {
        xs.iter()
            .enumerate()
            .map(|(i, &x)| {
                let (y, p) = f(i, x);
                sample(x, y, p, i as u32 * 5000)
            })
            .collect()
    };
    let ramp = mk(&|i, _| (100.5, 0.1 + 0.9 * i as f64 / n));
    let flat = mk(&|_, _| (250.5, 0.5));
    let wave = mk(&|_, x| {
        let y = 430.5 + 60.0 * (2.0 * std::f64::consts::PI * (x - 60.5) / 340.0).sin();
        (y, 1.0)
    });
    let dab = vec![sample(400.5, 555.5, 1.0, 0)];
    vec![ramp, flat, wave, dab]
}

fn render(p: &BrushPreset) -> Layer {
    let s = p
        .settings([0, 0, 0])
        .unwrap_or_else(|e| unreachable!("{e}"));
    let mut l = Layer::new(LayerId(1), "x", LayerKind::Raster);
    for pts in strokes() {
        brush::stroke(&mut l, (W, H), &s, &pts).unwrap_or_else(|e| unreachable!("{e}"));
    }
    l
}

#[derive(Debug, Default)]
pub struct Stats {
    /// Pixels where either side has ink.
    pub inked: usize,
    /// … of which differ by more than 2 alpha levels.
    pub off: usize,
    pub worst: u8,
    /// Total ink (sum of alpha / 255): Brushye and CSP.
    pub ink: (f64, f64),
}

fn compare(id: &str, p: &BrushPreset) -> Option<Stats> {
    let path = data(&format!("{id}.png"));
    let bytes = std::fs::read(&path).ok()?;
    let img =
        brushye_format::png_io::read_png(&bytes).unwrap_or_else(|e| unreachable!("{id}: {e}"));
    assert_eq!((img.width, img.height), (W, H), "{id}");
    let l = render(p);
    // XCHECK_DUMP=DIR writes Brushye's alpha as DIR/X-xx.png for analysis.
    if let Some(dir) = std::env::var_os("XCHECK_DUMP") {
        let mut out = brushye_command::core::render::Image {
            width: W,
            height: H,
            px: vec![0; (W * H * 4) as usize],
        };
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4) as usize;
                out.px[i..i + 4].copy_from_slice(&l.pixel(x, y));
            }
        }
        let bytes = brushye_format::png_io::png_bytes(&out).unwrap_or_else(|e| unreachable!("{e}"));
        std::fs::write(Path::new(&dir).join(format!("{id}.png")), bytes)
            .unwrap_or_else(|e| unreachable!("{e}"));
    }
    let mut st = Stats::default();
    for y in 0..H {
        for x in 0..W {
            let got = l.pixel(x, y)[3];
            let want = img.pixel(x, y)[3];
            if got > 0 || want > 0 {
                st.inked += 1;
                st.ink.0 += f64::from(got) / 255.0;
                st.ink.1 += f64::from(want) / 255.0;
                let d = got.abs_diff(want);
                st.worst = st.worst.max(d);
                if d > 2 {
                    st.off += 1;
                }
            }
        }
    }
    Some(st)
}

#[test]
fn importer_output_is_the_committed_presets() {
    let Some(dir) = std::env::var_os("BRUSHYE_XCHECK_SUT").map(PathBuf::from) else {
        return;
    };
    for (id, want) in presets() {
        let bytes = std::fs::read(dir.join(format!("{id}.sut")))
            .unwrap_or_else(|e| unreachable!("{id}: {e}"));
        let subs = brushye_sut::import(&bytes).unwrap_or_else(|e| unreachable!("{id}: {e}"));
        assert_eq!(subs[0].preset, want, "{id}");
    }
}

/// Pinned (B-012): most "off" pixels (% of inked) and total-ink error (%) per brush.
const PINS: &[(&str, f64, f64)] = &[
    ("X-01", 3.5, 1.0),
    ("X-02", 1.0, 0.5),
    ("X-03", 36.0, 7.0),  // soft tip in strokes (engine gap)
    ("X-04", 95.0, 33.0), // soft tip in strokes (engine gap)
    ("X-05", 20.0, 2.0),  // anti-aliased thin tip (engine gap)
    ("X-06", 17.0, 1.0),  // anti-aliased thin tip (engine gap)
    ("X-07", 2.0, 0.2),
    ("X-08", 4.5, 0.2),
    ("X-09", 1.5, 0.2),
    ("X-10", 2.2, 0.6),
    ("X-11", 3.2, 1.0),
    ("X-12", 84.0, 29.0), // soft tip in strokes (engine gap)
];

#[test]
fn imported_brushes_draw_like_csp() {
    assert_eq!(presets().len(), PINS.len());
    for (id, p) in presets() {
        let st = compare(&id, &p).unwrap_or_else(|| unreachable!("{id}: CSP export missing"));
        let off = 100.0 * st.off as f64 / st.inked.max(1) as f64;
        let ink_err = 100.0 * (st.ink.0 - st.ink.1).abs() / st.ink.1.max(1.0);
        let (_, max_off, max_ink) = PINS
            .iter()
            .find(|(k, _, _)| *k == id)
            .unwrap_or_else(|| unreachable!("{id}: no pin"));
        assert!(
            off <= *max_off && ink_err <= *max_ink,
            "{id}: {off:.2} % off (pin {max_off}), ink error {ink_err:.2} % (pin {max_ink})"
        );
        eprintln!(
            "{id}: inked {} px, {} off by >2 ({off:.2} %), worst {}, ink {:.0} vs CSP {:.0}",
            st.inked, st.off, st.worst, st.ink.0, st.ink.1
        );
    }
}
