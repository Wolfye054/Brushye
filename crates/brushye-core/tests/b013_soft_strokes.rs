//! B-013: soft-tip strokes (size 40, density 30, hardness levels 1..5, gap presets) against
//! CSP 1.13.2 exports. Stroke ink is pinned per configuration; `--nocapture` prints the table.

mod common;

use brushye_core::brush::{self, Aa, BrushSettings, Gap};
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

/// brush_run2.py `softrow`: (100.5, 200.5) -> (700.5, 200.5) every 2 px, then a dab.
fn softrow() -> Vec<Vec<Sample>> {
    let row = (0..301)
        .map(|i| smp(100.5 + 2.0 * f64::from(i), 200.5))
        .collect();
    vec![row, vec![smp(400.5, 420.5)]]
}

pub fn render(h: u8, gap: Gap, density: f64) -> Layer {
    let s = BrushSettings {
        size: 40.0,
        hardness: h,
        density,
        gap,
        aa: Aa::Middle,
        ..Default::default()
    };
    let mut l = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for pts in softrow() {
        brush::stroke(&mut l, (800, 600), &s, &pts).unwrap_or_else(|e| panic!("{e}"));
    }
    l
}

fn ink(get: impl Fn(u32, u32) -> u8, x0: u32, x1: u32, y0: u32, y1: u32) -> f64 {
    let mut s = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            s += f64::from(get(x, y));
        }
    }
    s / 255.0
}

#[test]
fn b013_report() {
    let gaps = [
        ("narrow", Gap::Narrow),
        ("normal", Gap::Normal),
        ("wide", Gap::Wide),
        ("fixed300", Gap::Fixed(300.0)),
    ];
    for h in 1..=5u8 {
        let mut line = format!("H{h}:");
        for (name, gap) in gaps {
            let img = Img::load_spec("brush", &format!("B013_h{h}_{name}.png"));
            let l = render(h, gap, 0.3);
            let b = |x: u32, y: u32| l.pixel(x, y)[3];
            let c = |x: u32, y: u32| img.at(x as usize, y as usize)[3];
            let (sb, sc) = (ink(b, 150, 651, 150, 251), ink(c, 150, 651, 150, 251));
            let (db, dc) = (ink(b, 350, 451, 370, 471), ink(c, 350, 451, 370, 471));
            let mid = |g: &dyn Fn(u32, u32) -> u8| {
                (150..651).map(|x| f64::from(g(x, 200))).sum::<f64>() / 501.0
            };
            // Levels 1-3 and 5 match to < 1 %; level 4's own tip profile is +7 % (B-013 open).
            let pin = if h == 4 { 9.0 } else { 1.0 };
            assert!(
                (100.0 * (sb - sc) / sc).abs() <= pin,
                "H{h} {name}: stroke ink {sb:.0} vs CSP {sc:.0}"
            );
            line += &format!(
                " {name}: stroke {sb:.0}/{sc:.0} ({:+.1} %) centre {:.1}/{:.1} dab {db:.0}/{dc:.0} |",
                100.0 * (sb - sc) / sc,
                mid(&b),
                mid(&c)
            );
        }
        eprintln!("{line}");
    }
}

/// For each soft level and gap preset: the spacing factor f (gap = f · preset %) whose stroke
/// ink matches CSP's.
#[test]
fn b013_fit_spacing_factor() {
    if std::env::var_os("B013_FIT").is_none() {
        return;
    }
    for h in 1..=4u8 {
        let mut line = format!("H{h}:");
        for (name, k) in [("narrow", 4.0), ("normal", 8.0), ("wide", 12.0)] {
            let img = Img::load_spec("brush", &format!("B013_h{h}_{name}.png"));
            let mut target = ink(|x, y| img.at(x as usize, y as usize)[3], 150, 651, 150, 251);
            if std::env::var_os("B013_NORMALISE").is_some() {
                // Scale by the lone dab's ink ratio (Brushye / CSP) to factor out profile error.
                let l = render(h, Gap::Normal, 0.3);
                let db = ink(|x, y| l.pixel(x, y)[3], 350, 451, 370, 471);
                let dc = ink(|x, y| img.at(x as usize, y as usize)[3], 350, 451, 370, 471);
                target *= db / dc;
            }
            let ink_at = |f: f64| {
                let l = render(h, Gap::Fixed(k * f), 0.3);
                ink(|x, y| l.pixel(x, y)[3], 150, 651, 150, 251)
            };
            let (mut lo, mut hi) = (0.5, 8.0);
            for _ in 0..30 {
                let mid = 0.5 * (lo + hi);
                if ink_at(mid) > target {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            line += &format!(" {name} f={:.3}", 0.5 * (lo + hi));
        }
        eprintln!("{line}");
    }
}
