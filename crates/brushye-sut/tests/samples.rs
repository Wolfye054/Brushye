//! Imports the maintainer's .sut samples (D-020). They live outside the repo, so this runs
//! only when `BRUSHYE_SUT_SAMPLES` names their folder (e.g. ~/brushye-samples/sut).

use std::path::PathBuf;

fn sample(dir: &PathBuf, id: &str) -> brushye_sut::SubTool {
    let path = std::fs::read_dir(dir)
        .unwrap_or_else(|e| unreachable!("{e}"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.extension().is_some_and(|x| x == "sut")
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with(&format!("{id}_")))
        })
        .unwrap_or_else(|| unreachable!("sample {id} missing"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| unreachable!("{e}"));
    let mut subs = brushye_sut::import(&bytes).unwrap_or_else(|e| unreachable!("{id}: {e}"));
    assert_eq!(subs.len(), 1, "{id}");
    subs.remove(0)
}

#[test]
fn samples_import_with_the_mapped_settings() {
    let Some(dir) = std::env::var_os("BRUSHYE_SUT_SAMPLES").map(PathBuf::from) else {
        eprintln!("BRUSHYE_SUT_SAMPLES not set; skipped");
        return;
    };
    let text = |id: &str| sample(&dir, id).preset.to_text();
    // Base: size 10, AA middle, horizontal axis, size pressure min 0, stabilization 6.
    assert_eq!(
        text("S-01"),
        "size=10.0 direction=horizontal stabilization=6 size_pressure=0"
    );
    let cases = [
        ("S-10", "size=1.0 "),
        ("S-14", "size=2000.0 "),
        ("S-16", "opacity=37 "),
        ("S-18", "aa=none "),
        ("S-20", "aa=strong "),
        ("S-26", "thickness=10 "),
        ("S-28", "direction=vertical "),
        ("S-30", "angle=45.0 "),
        ("S-32", "gap=narrow "),
        ("S-34", "gap=wide "),
        ("S-35", "gap=7.0 "),
        ("S-36", "density=50 "),
        ("S-37", "size_pressure=20"),
        ("S-38", "size_curve=[0.25657"),
        ("S-39", "opacity_pressure=0"),
        ("S-40", "density_pressure=0"),
        ("S-44", "ink=multiply "),
        ("S-45", "ink=compare_density "),
        ("S-46", "ink=background "),
        ("S-47", "ink=replace_alpha "),
        ("S-51", "size_pressure=0"),
        ("S-52", "stabilization=30 "),
    ];
    for (id, want) in cases {
        let t = text(id);
        assert!(t.contains(want), "{id}: {t:?} lacks {want:?}");
    }
    // Every sample imports; features Brushye lacks are reported, not dropped silently.
    let mut ids = Vec::new();
    for e in std::fs::read_dir(&dir)
        .unwrap_or_else(|e| unreachable!("{e}"))
        .filter_map(Result::ok)
    {
        let n = e.file_name().to_string_lossy().into_owned();
        if n.ends_with(".sut") {
            ids.push(n.split('_').next().unwrap_or_default().to_owned());
        }
    }
    assert!(ids.len() >= 56);
    for id in &ids {
        let _ = sample(&dir, id);
    }
    let reported = |id: &str, feature: &str| {
        sample(&dir, id)
            .notes
            .iter()
            .any(|n| n.to_string().contains(feature))
    };
    for (id, feature) in [
        ("S-41", "velocity"),
        ("S-42", "random"),
        ("S-43", "Thickness dynamics"),
        ("S-48", "Color mixing"),
        ("S-55", "Texture"),
        ("S-56", "Spray effect"),
        ("S-57", "Watercolor edge"),
        ("S-58", "Color jitter"),
        ("S-59", "Material tip"),
        ("S-61", "2-Brush shape"),
        ("S-62", "Ribbon"),
        ("S-23", "Hardness"),
        ("S-01", "adjust by speed"),
    ] {
        assert!(reported(id, feature), "{id} should report {feature}");
    }
}

#[test]
fn damaged_samples_fail_cleanly() {
    let Some(dir) = std::env::var_os("BRUSHYE_SUT_SAMPLES").map(PathBuf::from) else {
        return;
    };
    for name in ["S-01_base.sut", "S-38_pressure_size_min20_scurve.sut"] {
        let good = std::fs::read(dir.join(name)).unwrap_or_else(|e| unreachable!("{e}"));
        let mut seed = 0x9e37_79b9_u32;
        for round in 0..3000 {
            let mut b = good.clone();
            for _ in 0..1 + round % 16 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let i = seed as usize % b.len();
                b[i] ^= (seed >> 8) as u8 | 1;
            }
            let _ = brushye_sut::import(&b);
        }
        for cut in [16, 100, 4096, good.len() / 2, good.len() - 1] {
            let _ = brushye_sut::import(&good[..cut]);
        }
    }
}
