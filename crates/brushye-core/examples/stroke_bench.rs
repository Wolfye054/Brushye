//! Times one long stroke at several brush settings (release build).
use brushye_core::brush::{self, BrushSettings};
use brushye_core::input::Sample;
use brushye_core::{Layer, LayerId, LayerKind};

fn main() {
    let pts: Vec<Sample> = (0..2000)
        .map(|i| {
            let t = f64::from(i) / 2000.0;
            Sample {
                x: 100.0 + 1800.0 * t,
                y: 1000.0 + 400.0 * (t * 20.0).sin(),
                t_us: i * 4000,
                pressure: Some((0.3 + 0.7 * (t * 3.0).sin().abs()) as f32),
                tilt: None,
                twist: None,
                eraser: false,
                barrel: [false; 2],
            }
        })
        .collect();
    for (size, hard) in [(10.0, 5), (40.0, 5), (100.0, 5), (100.0, 2), (300.0, 5)] {
        let s = BrushSettings {
            size,
            hardness: hard,
            size_pressure: Some(Default::default()),
            ..Default::default()
        };
        let mut l = Layer::new(LayerId(1), "t", LayerKind::Raster);
        let t0 = std::time::Instant::now();
        brush::stroke(&mut l, (2000, 2000), &s, &pts).unwrap_or_else(|e| panic!("{e}"));
        let n = brush::place_dabs(&s, &pts).len();
        let full = t0.elapsed().as_secs_f64() * 1000.0;
        // Live: 4 samples per update (≈ one display frame at 240 Hz pen input).
        let mut live = brush::LiveStroke::new(
            Layer::new(LayerId(1), "t", LayerKind::Raster),
            (2000, 2000),
            s.clone(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let (mut worst, t1) = (0.0f64, std::time::Instant::now());
        for c in pts.chunks(4) {
            let t = std::time::Instant::now();
            live.push(c);
            worst = worst.max(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "size {size} hardness {hard}: {n} dabs, full {full:.1} ms; live total {:.1} ms, worst update {worst:.2} ms",
            t1.elapsed().as_secs_f64() * 1000.0
        );
    }
}
