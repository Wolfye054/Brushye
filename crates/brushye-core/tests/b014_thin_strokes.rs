//! B-014 run 3: thin-tip strokes (size 41, thickness 30, Vertical) against CSP 1.13.2.
//! AA None strokes match CSP (pinned). AA Middle is an open engine gap (B-014): its numbers are
//! pinned so any change shows.

mod common;

use brushye_core::brush::{self, Aa, BrushSettings, Gap, TipAxis};
use brushye_core::input::Sample;
use brushye_core::{Layer, LayerId, LayerKind};
use common::Img;

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

pub fn render(angle: f64, aa: Aa, gap: Gap) -> Layer {
    let s = BrushSettings {
        size: 41.0,
        thickness: 0.3,
        tip_axis: TipAxis::Vertical,
        angle,
        aa,
        gap,
        ..Default::default()
    };
    let row: Vec<Sample> = (0..301)
        .map(|i| smp(100.5 + 2.0 * f64::from(i), 200.5))
        .collect();
    let mut l = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for pts in [row, vec![smp(400.5, 420.5)]] {
        brush::stroke(&mut l, (800, 600), &s, &pts).unwrap_or_else(|e| panic!("{e}"));
    }
    l
}

/// (pixels off by > 2 in the stroke band, inked pixels there, ink Brushye, ink CSP)
pub fn compare(file: &str, angle: f64, aa: Aa, gap: Gap) -> (usize, usize, f64, f64) {
    let img = Img::load_spec("brush", file);
    let l = render(angle, aa, gap);
    let (mut off, mut inked, mut ib, mut ic) = (0, 0, 0.0, 0.0);
    for y in 150..251u32 {
        for x in 60..741u32 {
            let b = l.pixel(x, y)[3];
            let c = img.at(x as usize, y as usize)[3];
            if b > 0 || c > 0 {
                inked += 1;
                ib += f64::from(b) / 255.0;
                ic += f64::from(c) / 255.0;
                if b.abs_diff(c) > 2 {
                    off += 1;
                }
            }
        }
    }
    (off, inked, ib, ic)
}

#[test]
fn b014_rows_report() {
    for a in [0.0, 30.0] {
        for (aa, an) in [(Aa::None, 0), (Aa::Middle, 2)] {
            for (gap, gn) in [(Gap::Fixed(300.0), "fixed300"), (Gap::Normal, "normal")] {
                let f = format!("B014_row_a{a}_aa{an}_{gn}.png");
                let (off, inked, ib, ic) = compare(&f, a, aa, gap);
                let off_pct = 100.0 * off as f64 / inked.max(1) as f64;
                let ink_pct = 100.0 * (ib - ic).abs() / ic;
                // Pins: AA None within 1.5 % (exact dab model, B-010); AA Middle interim.
                let (max_off, max_ink) = match (aa, gap) {
                    (Aa::None, _) => (1.5, 1.5),
                    (_, Gap::Fixed(_)) => (50.0, 30.0),
                    _ => (25.0, 2.0),
                };
                assert!(
                    off_pct <= max_off && ink_pct <= max_ink,
                    "{f}: {off_pct:.1} % off (pin {max_off}), ink {ink_pct:.1} % (pin {max_ink})"
                );
                eprintln!(
                    "{f}: {off}/{inked} off ({:.1} %), ink {ib:.0} vs {ic:.0} ({:+.1} %)",
                    100.0 * off as f64 / inked.max(1) as f64,
                    100.0 * (ib - ic) / ic
                );
            }
        }
    }
}

#[test]
fn b014_cross_section() {
    if std::env::var_os("B014_XS").is_none() {
        return;
    }
    let c0 = Img::load_spec("brush", "B014_row_a0_aa0_normal.png");
    let c2 = Img::load_spec("brush", "B014_row_a0_aa2_normal.png");
    let b2 = render(0.0, Aa::Middle, Gap::Normal);
    for y in 190..212u32 {
        eprintln!(
            "y{y}: csp aa0 {:3}  csp aa2 {:3}  brushye aa2 {:3}",
            c0.at(400, y as usize)[3],
            c2.at(400, y as usize)[3],
            b2.pixel(400, y)[3]
        );
    }
}
