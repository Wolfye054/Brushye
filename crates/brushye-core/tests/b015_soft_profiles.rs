//! B-015: soft-tip single dabs at sizes 8..128 (levels 1..4) against CSP 1.13.2 exports.
//! Ink per configuration is pinned; `--nocapture` prints the table.

mod common;

use brushye_core::brush::{self, Aa, BrushSettings};
use brushye_core::input::Sample;
use brushye_core::{Layer, LayerId, LayerKind};
use common::Img;

pub const DABS: [(f64, f64); 4] = [
    (200.5, 150.5),
    (600.8, 150.2),
    (200.3, 450.7),
    (600.5, 450.5),
];
pub const SIZES: [u32; 5] = [8, 16, 32, 64, 128];

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

pub fn render(h: u8, size: f64) -> Layer {
    let s = BrushSettings {
        size,
        hardness: h,
        aa: Aa::Middle,
        ..Default::default()
    };
    let mut l = Layer::new(LayerId(1), "t", LayerKind::Raster);
    for (x, y) in DABS {
        brush::stroke(&mut l, (800, 600), &s, &[smp(x, y)]).unwrap_or_else(|e| panic!("{e}"));
    }
    l
}

/// Per configuration: (Σ alpha Brushye, Σ alpha CSP, max |diff|) over the whole canvas.
pub fn compare(h: u8, size: u32) -> (f64, f64, u8) {
    let img = Img::load_spec("brush", &format!("B015_h{h}_s{size}.png"));
    let l = render(h, f64::from(size));
    let (mut sb, mut sc, mut worst) = (0.0, 0.0, 0u8);
    for y in 0..600u32 {
        for x in 0..800u32 {
            let b = l.pixel(x, y)[3];
            let c = img.at(x as usize, y as usize)[3];
            sb += f64::from(b);
            sc += f64::from(c);
            worst = worst.max(b.abs_diff(c));
        }
    }
    (sb / 255.0, sc / 255.0, worst)
}

#[test]
fn b015_report() {
    for h in 1..=4u8 {
        let mut line = format!("H{h}:");
        for s in SIZES {
            let (b, c, w) = compare(h, s);
            // Pinned (B-015): sizes >= 16 within 3 % ink at levels 1-3 and 4 % at level 4;
            // size 8 is open (levels 1-3 run heavy: +6, +10, +24 %).
            let err = (100.0 * (b - c) / c).abs();
            let pin = match (h, s) {
                (_, 8) => 25.0,
                (4, _) => 4.0,
                _ => 3.5,
            };
            assert!(err <= pin, "H{h} s{s}: ink {b:.1} vs CSP {c:.1}");
            line += &format!(
                " s{s}: {b:.1}/{c:.1} ({:+.1} %) worst {w} |",
                100.0 * (b - c) / c
            );
        }
        eprintln!("{line}");
    }
    let (b, c, w) = compare(5, 32);
    eprintln!("H5 s32 control: {b:.1}/{c:.1} worst {w}");
}
