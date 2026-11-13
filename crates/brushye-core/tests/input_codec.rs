//! Property tests for the stroke sample encoding (docs/command-format.md §7).

use brushye_core::input::{Sample, canonical, decode, encode};
use proptest::prelude::*;

fn stream() -> impl Strategy<Value = Vec<Sample>> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        proptest::collection::vec(
            (
                -1.0e6f64..1.0e6,
                -1.0e6f64..1.0e6,
                0u32..20_000,
                0.0f32..=1.0,
                -90.0f32..=90.0,
                -90.0f32..=90.0,
                0.0f32..360.0,
                any::<[bool; 3]>(),
            ),
            0..60,
        ),
    )
        .prop_map(|(has_p, has_t, has_w, rows)| {
            let mut t = 0u32;
            rows.into_iter()
                .map(|(x, y, dt, p, tx, ty, tw, f)| {
                    t = t.saturating_add(dt);
                    Sample {
                        x,
                        y,
                        t_us: t,
                        pressure: has_p.then_some(p),
                        tilt: has_t.then_some([tx, ty]),
                        twist: has_w.then_some(tw),
                        eraser: f[0],
                        barrel: [f[1], f[2]],
                    }
                })
                .collect()
        })
}

proptest! {
    #[test]
    fn canonical_is_idempotent_and_decode_inverts_encode(v in stream()) {
        let c = canonical(&v).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(canonical(&c).map_err(|e| TestCaseError::fail(e.to_string()))?, c.clone());
        let blob = encode(&c).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(decode(&blob).map_err(|e| TestCaseError::fail(e.to_string()))?, c.clone());
        // Quantization error stays within half a step.
        for (a, b) in v.iter().zip(&c) {
            prop_assert!((a.x - b.x).abs() <= 0.5 / 256.0 + 1e-9);
            prop_assert!((a.y - b.y).abs() <= 0.5 / 256.0 + 1e-9);
        }
    }

    #[test]
    fn decode_never_panics_on_garbage(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let _ = decode(&bytes);
    }
}
