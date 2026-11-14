//! The live (incremental) stroke preview equals the finished stroke exactly.

use brushye_core::brush::{self, Aa, BrushSettings, Dynamics, Gap, InkMode, LiveStroke};
use brushye_core::input::{Sample, canonical};
use brushye_core::{Layer, LayerId, LayerKind};
use proptest::prelude::*;

fn sample(x: f64, y: f64, p: f32, i: usize) -> Sample {
    Sample {
        x,
        y,
        t_us: i as u32 * 4000,
        pressure: Some(p),
        tilt: None,
        twist: None,
        eraser: false,
        barrel: [false; 2],
    }
}

fn base_layer() -> Layer {
    let mut l = Layer::new(LayerId(1), "base", LayerKind::Raster);
    let px: Vec<u8> = (0..120 * 90 * 4).map(|i| (i * 7 % 251) as u8).collect();
    l.put_pixels((300, 200), 40, 30, 120, 90, &px);
    l
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn preview_equals_the_finished_stroke(
        pts in proptest::collection::vec((0.0f64..300.0, 0.0f64..200.0, 0.0f32..=1.0), 1..40),
        cuts in proptest::collection::vec(1usize..6, 1..40),
        size in 1.0f64..60.0,
        hardness in 1u8..=5,
        aa in 0usize..4,
        ink in 0usize..35,
        thin in any::<bool>(),
        pressure in any::<bool>(),
        gap in 0usize..4,
    ) {
        let samples = canonical(
            &pts.iter().enumerate().map(|(i, &(x, y, p))| sample(x, y, p, i)).collect::<Vec<_>>(),
        ).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let s = BrushSettings {
            size,
            hardness,
            aa: [Aa::None, Aa::Weak, Aa::Middle, Aa::Strong][aa],
            ink: InkMode::ALL[ink],
            color: [200, 60, 30],
            density: 0.6,
            opacity: 0.8,
            thickness: if thin { 0.4 } else { 1.0 },
            angle_follows_line: thin,
            gap: [Gap::Narrow, Gap::Normal, Gap::Wide, Gap::Fixed(40.0)][gap],
            size_pressure: pressure.then(Dynamics::default),
            ..Default::default()
        };
        let bounds = (300, 200);
        let mut live = LiveStroke::new(base_layer(), bounds, s.clone()).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let mut rest: &[Sample] = &samples;
        let mut cut = cuts.iter().cycle();
        while !rest.is_empty() {
            let n = (*cut.next().unwrap_or(&1)).min(rest.len());
            live.push(&rest[..n]);
            rest = &rest[n..];
        }
        let mut done = base_layer();
        brush::stroke(&mut done, bounds, &s, &samples).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert!(live.preview() == &done, "preview differs from the finished stroke");
    }

    #[test]
    fn final_dabs_never_change(
        pts in proptest::collection::vec((0.0f64..300.0, 0.0f64..200.0, 0.0f32..=1.0), 3..40),
        stab in 0u8..12,
        pressure in any::<bool>(),
    ) {
        let samples: Vec<Sample> = pts.iter().enumerate().map(|(i, &(x, y, p))| sample(x, y, p, i)).collect();
        let s = BrushSettings {
            size: 12.0,
            stabilization: stab,
            size_pressure: pressure.then(Dynamics::default),
            ..Default::default()
        };
        let full = brush::place_dabs_live(&s, &samples).0;
        for n in 1..samples.len() {
            let (dabs, stable) = brush::place_dabs_live(&s, &samples[..n]);
            prop_assert!(stable <= dabs.len());
            prop_assert_eq!(&dabs[..stable], &full[..stable], "prefix of {} samples", n);
        }
    }
}
