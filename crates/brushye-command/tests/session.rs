//! Session behaviour, journal replay, migrations and codec round-trips (M2).

use brushye_command::commands::{self, Command, Migration};
use brushye_command::syntax::{Line, RawLine, Value, parse_line, print_line};
use brushye_command::{Query, Session};
use serde_json::Value as Json;

fn run(s: &mut Session, script: &str) {
    if let Err(e) = s.run_script(script) {
        panic!("script failed: {e}");
    }
}

fn q(s: &mut Session, query: &str) -> Json {
    match s.run_line(query) {
        Ok(Some(j)) => j,
        other => panic!("{query}: {other:?}"),
    }
}

fn layer_ids(s: &mut Session) -> Vec<String> {
    q(s, "?layer.list")
        .as_array()
        .map(|a| {
            a.iter()
                .map(|l| l["id"].as_str().unwrap_or("").to_owned())
                .collect()
        })
        .unwrap_or_default()
}

const BASIC: &str = "\
# M2 smoke script
doc.new 800 600 dpi=350 paper=#ffffff
layer.new L1 name=\"Sketch\"
layer.new L2 name=\"Ink\" above=L1
layer.set L2 blend=multiply opacity=80
layer.new L3
layer.move L3 to=bottom
layer.select L2
";

#[test]
fn basic_script_builds_the_expected_document() {
    let mut s = Session::new();
    run(&mut s, BASIC);
    assert_eq!(layer_ids(&mut s), vec!["L3", "L1", "L2"]);
    let l2 = q(&mut s, "?layer.get L2");
    assert_eq!(l2["blend"], "multiply");
    assert_eq!(l2["opacity"], 80);
    assert_eq!(l2["name"], "Ink");
    assert_eq!(q(&mut s, "?layer.get L3")["name"], "Layer 3");
    let info = q(&mut s, "?doc.info");
    assert_eq!(
        (info["width"].as_u64(), info["dpi"].as_u64()),
        (Some(800), Some(350))
    );
    assert_eq!(info["paper"], "#ffffff");
    assert_eq!(info["active"], "L2");
}

#[test]
fn undo_redo_and_redo_branch_discard() {
    let mut s = Session::new();
    run(&mut s, "doc.new 10 10\nlayer.new L1\nlayer.new L2\n");
    run(&mut s, "history.undo\n");
    assert_eq!(layer_ids(&mut s), vec!["L1"]);
    run(&mut s, "history.redo\n");
    assert_eq!(layer_ids(&mut s), vec!["L1", "L2"]);
    run(&mut s, "history.undo\nlayer.new L3\n");
    assert_eq!(layer_ids(&mut s), vec!["L1", "L3"]);
    // The redo branch (L2) is gone.
    run(&mut s, "history.redo\n");
    assert_eq!(layer_ids(&mut s), vec!["L1", "L3"]);
    // Undo/redo at the ends are no-ops.
    run(
        &mut s,
        "history.undo\nhistory.undo\nhistory.undo\nhistory.undo\n",
    );
    assert_eq!(layer_ids(&mut s), Vec::<String>::new());
    assert_eq!(q(&mut s, "?history")["can_undo"], false);
}

#[test]
fn consecutive_same_property_changes_merge_like_csp() {
    // H-001: an opacity drag and a typed opacity right after it are one entry; undo goes back to
    // the value before the whole run.
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 10 10\nlayer.new L1\nlayer.set L1 opacity=50\n",
    );
    run(&mut s, "layer.set L1 name=\"x\"\n"); // different key set: new step
    run(
        &mut s,
        "layer.set L1 opacity=93\nlayer.set L1 opacity=70\nlayer.set L1 opacity=40\n",
    );
    assert_eq!(
        q(&mut s, "?history")["steps"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(q(&mut s, "?layer.get L1")["opacity"], 40);
    run(&mut s, "history.undo\n");
    assert_eq!(q(&mut s, "?layer.get L1")["opacity"], 50);
    run(&mut s, "history.redo\n");
    assert_eq!(q(&mut s, "?layer.get L1")["opacity"], 40);
    // The journal records the merges explicitly.
    let j = s.journal_text();
    assert!(j.contains("@6+ layer.set L1 opacity=70"), "{j}");
    assert!(j.contains("@7+ layer.set L1 opacity=40"), "{j}");
    assert!(j.contains("@5 layer.set L1 opacity=93"), "{j}");
}

#[test]
fn visibility_is_journaled_but_not_undoable() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 10 10\nlayer.new L1\nlayer.set L1 opacity=10\nlayer.show L1 visible=false\n",
    );
    assert_eq!(
        q(&mut s, "?history")["steps"].as_array().map(Vec::len),
        Some(3)
    );
    run(&mut s, "history.undo\n");
    assert_eq!(q(&mut s, "?layer.get L1")["visible"], false);
    assert_eq!(q(&mut s, "?layer.get L1")["opacity"], 100);
    assert!(s.journal_text().contains("layer.show L1 visible=false"));
}

#[test]
fn failed_commands_change_nothing_and_are_not_journaled() {
    let mut s = Session::new();
    run(&mut s, "doc.new 10 10\nlayer.new L1\n");
    let before = s.journal_text();
    for bad in [
        "layer.new L1",                // duplicate id
        "layer.delete L9",             // no such layer
        "layer.set L1 opacity=101",    // out of range
        "layer.set L1",                // nothing to set
        "layer.move L1",               // no target
        "layer.set L1 colour=#fff000", // unknown key
        "frobnicate",                  // unknown command
        "layer.set/2 L1 opacity=1",    // newer than this Brushye
    ] {
        assert!(s.run_line(bad).is_err(), "{bad} should fail");
    }
    assert_eq!(s.journal_text(), before);
}

#[test]
fn locked_layers_reject_edits_except_unlocking() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 10 10\nlayer.new L1\nlayer.set L1 locked=true\n",
    );
    assert!(s.run_line("layer.set L1 opacity=5").is_err());
    assert!(s.run_line("layer.set L1 locked=false opacity=5").is_err());
    run(
        &mut s,
        "layer.set L1 locked=false\nlayer.set L1 opacity=5\n",
    );
    assert_eq!(q(&mut s, "?layer.get L1")["opacity"], 5);
}

#[test]
fn pixels_fill_queries_and_undo() {
    let mut s = Session::new();
    // 2x1 block: opaque red, half-transparent blue.
    run(
        &mut s,
        "doc.new 4 4 paper=#ffffff\nlayer.new L1\nlayer.pixels L1 x=1 y=2 w=2 h=1 data=b64:/wAA/wAA/4A=\n",
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=1 y=2"),
        serde_json::json!([255, 0, 0, 255])
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=2 y=2"),
        serde_json::json!([0, 0, 255, 128])
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=0 y=0"),
        serde_json::json!([0, 0, 0, 0])
    );
    // Rendered over white paper: 15-bit lerp (L-002) of white towards blue at alpha 128.
    assert_eq!(
        q(&mut s, "?pixel x=2 y=2"),
        serde_json::json!([127, 127, 255, 255])
    );
    run(
        &mut s,
        "layer.set L1 blend=multiply\nlayer.fill L1 color=#808080\n",
    );
    assert_eq!(
        q(&mut s, "?pixel x=0 y=0"),
        serde_json::json!([128, 128, 128, 255])
    );
    run(&mut s, "history.undo\n");
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=0 y=0"),
        serde_json::json!([0, 0, 0, 0])
    );
    assert!(s.run_line("?pixel x=4 y=0").is_err());
    assert!(
        s.run_line("layer.pixels L1 x=0 y=0 w=2 h=2 data=b64:AAAA")
            .is_err()
    ); // wrong size
}

#[test]
fn locks_apply_to_pixel_writes() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 2 1 paper=none\nlayer.new L1\nlayer.pixels L1 x=0 y=0 w=1 h=1 data=b64:AQIDQA==\n",
    );
    run(
        &mut s,
        "layer.set L1 lock_alpha=true\nlayer.fill L1 color=#ff0000\n",
    );
    // Alpha kept (0x40 and 0): only existing coverage gets the new colour (L-006).
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=0 y=0"),
        serde_json::json!([255, 0, 0, 64])
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=1 y=0"),
        serde_json::json!([0, 0, 0, 0])
    );
    run(&mut s, "layer.set L1 locked=true\n");
    assert!(s.run_line("layer.fill L1 color=#00ff00").is_err());
}

#[test]
fn undo_limit_drops_oldest_steps() {
    let mut s = Session::new();
    s.set_undo_limit(2);
    run(
        &mut s,
        "doc.new 10 10\nlayer.new L1\nlayer.new L2\nlayer.new L3\n",
    );
    run(&mut s, "history.undo\nhistory.undo\nhistory.undo\n");
    assert_eq!(layer_ids(&mut s), vec!["L1"]);
}

#[test]
fn replay_reproduces_document_and_journal() {
    let mut s = Session::new();
    run(&mut s, BASIC);
    run(
        &mut s,
        "layer.set L1 opacity=30\nlayer.set L1 opacity=60\nhistory.undo\nlayer.show L3 visible=false\nhistory.redo\nlayer.delete L3\n",
    );
    let journal = s.journal_text();
    let r = Session::replay(&journal).unwrap_or_else(|e| panic!("replay: {e}"));
    assert_eq!(r.journal_text(), journal);
    let h1 = s.query(&Query::DocHash(commands::QDocHash {})).ok();
    let h2 = r.query(&Query::DocHash(commands::QDocHash {})).ok();
    assert_eq!(h1, h2);
    assert_eq!(r.active_layer(), s.active_layer());
}

#[test]
fn replay_rejects_malformed_journals() {
    let ok = "#brushye-journal v1 app=0\n@1 doc.new 10 10\n@2 layer.new L1\n";
    assert!(Session::replay(ok).is_ok());
    let cases = [
        "@1 doc.new 10 10\n",                                             // no header
        "#brushye-journal v9 app=0\n@1 doc.new 10 10\n",                  // unknown format
        "#brushye-journal v1 app=0\n@1 doc.new 10 10\n@3 layer.new L1\n", // gap
        "#brushye-journal v1 app=0\ndoc.new 10 10\n",                     // no seq
        "#brushye-journal v1 app=0\n@1 doc.new 10 10\n@2+ layer.select L9\n", // merge on app state
        "#brushye-journal v1 app=0\n@1 doc.new 10 10\n@2+ layer.new L1\n", // nothing to merge into
        "#brushye-journal v1 app=0\n@1 doc.new 10 10\n?doc.hash\n",       // query in journal
    ];
    for c in cases {
        assert!(Session::replay(c).is_err(), "should reject:\n{c}");
    }
    let e =
        Session::replay("#brushye-journal v1 app=0\n@1 doc.new 10 10\n@2 layer.new L1 x=1\n").err();
    assert_eq!(e.map(|e| e.line), Some(3));
}

/// Cross-platform golden value (D-012): CI runs this on Windows, macOS and Linux.
#[test]
fn golden_document_hash_is_platform_independent() {
    let mut s = Session::new();
    run(&mut s, BASIC);
    assert_eq!(q(&mut s, "?doc.hash"), "4c9b1d8e370d0616");
}

const STROKE: &str = "\
doc.new 64 48 paper=none
layer.new L1
brush.stroke L1 color=#203040 size=9.5 aa=weak opacity=80 density=60 gap=wide size_pressure=20 size_curve=[0.5,0.25] samples=[4,4,0.2,0;30,10.5,0.9,8000;50,40,0.5,16000]
brush.stroke L1 color=#ff0000 size=12 hardness=2 gap=37 samples=[10,40,1,0;40,20,1,5000]
brush.stroke L1 color=#00ff00 size=14 aa=none thickness=30 angle_line=true stabilization=6 ink=multiply density_by_gap=true samples=[5,30,1,0;20,35,1,4000;35,25,1,8000;50,30,1,12000]
brush.stroke L1 color=#000000 size=8 ink=erase opacity=50 samples=[0,0,1,0;60,45,1,9000]
brush.stroke L1 color=#8040c0 size=20 hardness=3 thickness=50 angle=30 direction=horizontal samples=[32,24,0.7,0]
";

#[test]
fn brush_strokes_replay_undo_and_print_samples_as_blobs() {
    let mut s = Session::new();
    run(&mut s, STROKE);
    let hash = q(&mut s, "?doc.hash");
    let journal = s.journal_text();
    // Matrix samples are journaled as the canonical blob; defaults are omitted.
    let line = journal
        .lines()
        .find(|l| l.contains("brush.stroke") && l.contains("#203040"))
        .unwrap_or_else(|| panic!("{journal}"));
    assert!(line.contains("samples=b64:"), "{line}");
    assert!(!line.contains("hardness="), "{line}");
    assert!(line.contains("size_curve=[0.5,0.25]"), "{line}");
    // Replay reproduces the document exactly.
    let mut r = Session::replay(&journal).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(q(&mut r, "?doc.hash"), hash);
    assert_eq!(r.journal_text(), journal);
    // Ink landed, and each stroke is one undo step.
    assert_ne!(q(&mut s, "?layer.pixel L1 x=30 y=10")[3], 0);
    run(
        &mut s,
        "history.undo\nhistory.undo\nhistory.undo\nhistory.undo\nhistory.undo",
    );
    assert_eq!(q(&mut s, "?layer.get L1")["tiles"], 0);
}

#[test]
fn brush_stroke_rejects_bad_settings_without_side_effects() {
    let mut s = Session::new();
    run(&mut s, "doc.new 32 32\nlayer.new L1");
    for bad in [
        "brush.stroke L1 color=#000000 size=0 samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 hardness=6 samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 opacity=101 samples=[1,1,1,0]",
        "brush.stroke L1 color=#00000080 size=5 samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 size_curve=[0.5,0.2] samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 size_pressure=0 size_curve=[0.5,0.2;0.4,0.3] samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 gap=huge samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 thickness=0 samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 stabilization=101 samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 ink=smudge samples=[1,1,1,0]",
        "brush.stroke L1 color=#000000 size=5 samples=[]",
        "brush.stroke L9 color=#000000 size=5 samples=[1,1,1,0]",
    ] {
        assert!(s.run_line(bad).is_err(), "{bad}");
    }
    run(&mut s, "layer.set L1 locked=true");
    assert!(
        s.run_line("brush.stroke L1 color=#000000 size=5 samples=[1,1,1,0]")
            .is_err()
    );
    assert_eq!(
        q(&mut s, "?history")["steps"].as_array().map(Vec::len),
        Some(3)
    );
}

#[test]
fn brush_stroke_respects_lock_transparency() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 16 16 paper=none\nlayer.new L1\nlayer.pixels L1 x=2 y=2 w=1 h=1 data=b64:AAAA/w==\nlayer.set L1 lock_alpha=true\nbrush.stroke L1 color=#ffffff size=12 samples=[3,3,1,0]",
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=2 y=2"),
        serde_json::json!([255, 255, 255, 255])
    );
    assert_eq!(
        q(&mut s, "?layer.pixel L1 x=4 y=4"),
        serde_json::json!([0, 0, 0, 0])
    );
}

#[test]
fn global_pressure_curve_is_journaled_applied_first_and_not_undoable() {
    let stroke = "brush.stroke L1 color=#000000 size=30 size_pressure=0 samples=[10,30,0.3,0;90,30,0.3,8000]";
    let mut s = Session::new();
    run(&mut s, "doc.new 100 60 paper=none\nlayer.new L1");
    run(&mut s, stroke);
    let linear = q(&mut s, "?doc.hash");
    // The same stroke after a "stronger" global curve: thinner (sub(global(P)), B-006).
    run(
        &mut s,
        "history.undo\ninput.pressure_curve points=[0.5,0.1]",
    );
    assert_eq!(
        q(&mut s, "?input.pressure_curve"),
        serde_json::json!([[0.5, 0.1]])
    );
    run(&mut s, stroke);
    assert_ne!(q(&mut s, "?doc.hash"), linear);
    assert_eq!(q(&mut s, "?layer.pixel L1 x=50 y=40")[3], 0);
    let curved = q(&mut s, "?doc.hash");
    // Not undoable: undo removes the stroke, not the setting.
    run(&mut s, "history.undo");
    assert_eq!(
        q(&mut s, "?input.pressure_curve"),
        serde_json::json!([[0.5, 0.1]])
    );
    run(&mut s, "history.redo");
    // Changing the curve later doesn't touch strokes already made; replay reproduces both.
    run(&mut s, "input.pressure_curve points=[]");
    assert_eq!(q(&mut s, "?doc.hash"), curved);
    let mut r = Session::replay(&s.journal_text()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(q(&mut r, "?doc.hash"), curved);
    assert_eq!(q(&mut r, "?input.pressure_curve"), serde_json::json!([]));
    // It's an application setting: doc.new keeps it.
    run(
        &mut s,
        "input.pressure_curve points=[0.4,0.6]\ndoc.new 10 10",
    );
    assert_eq!(
        q(&mut s, "?input.pressure_curve"),
        serde_json::json!([[0.4, 0.6]])
    );
}

/// Cross-platform golden value for brush output (D-012): CI runs this on all three OSes.
#[test]
fn golden_brush_hash_is_platform_independent() {
    let mut s = Session::new();
    run(&mut s, STROKE);
    // Updated for engine fidelity fixes before 1.0 (D-021): B-015 soft-dab offset.
    assert_eq!(q(&mut s, "?doc.hash"), "33d37c16b6046ffd");
}

#[test]
fn migration_engine_runs_the_chain() {
    fn v1_to_v2(mut r: RawLine) -> Result<RawLine, String> {
        // v2 renamed `alpha` to `opacity`
        for (k, _) in &mut r.named {
            if k == "alpha" {
                *k = "opacity".into();
            }
        }
        Ok(r)
    }
    fn v2_to_v3(mut r: RawLine) -> Result<RawLine, String> {
        // v3 takes opacity in percent instead of 0..1
        for (k, v) in &mut r.named {
            if k == "opacity"
                && let Value::Num(n) = v
            {
                let pct = (n.parse::<f64>().map_err(|e| e.to_string())? * 100.0).round();
                *v = Value::Num(format!("{pct}"));
            }
        }
        Ok(r)
    }
    const TABLE: &[Migration] = &[("demo.set", 1, v1_to_v2), ("demo.set", 2, v2_to_v3)];
    fn current(name: &str) -> Option<u32> {
        (name == "demo.set").then_some(3)
    }
    let Ok(Line::Item(raw)) = parse_line("demo.set L1 alpha=0.25") else {
        panic!("parse")
    };
    let out = commands::migrate_with(TABLE, current, raw).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(print_line(&out), "demo.set/3 L1 opacity=25");
    let Ok(Line::Item(newer)) = parse_line("demo.set/4 L1") else {
        panic!("parse")
    };
    assert!(commands::migrate_with(TABLE, current, newer).is_err());
}

mod roundtrip {
    use super::*;
    use brushye_command::commands::*;
    use brushye_core::{BlendMode, LayerId, LayerKind, Rgba8};
    use proptest::prelude::*;

    fn id() -> impl Strategy<Value = LayerId> {
        (0u32..5000).prop_map(LayerId)
    }
    fn name() -> impl Strategy<Value = String> {
        // Any text, including quotes, backslashes, control and non-ASCII characters.
        proptest::collection::vec(any::<char>(), 0..12).prop_map(|v| v.into_iter().collect())
    }
    fn blend() -> impl Strategy<Value = BlendMode> {
        proptest::sample::select(BlendMode::ALL.to_vec())
    }
    fn command() -> impl Strategy<Value = Command> {
        prop_oneof![
            (
                1u32..40000,
                1u32..40000,
                proptest::option::of(1u32..2400),
                proptest::option::of(prop_oneof![
                    any::<[u8; 4]>().prop_map(|c| Paper::Color(Rgba8(c))),
                    Just(Paper::None)
                ])
            )
                .prop_map(|(width, height, dpi, paper)| Command::DocNew(DocNew {
                    width,
                    height,
                    dpi,
                    paper
                })),
            (
                id(),
                proptest::option::of(name()),
                proptest::option::of(id())
            )
                .prop_map(|(layer, name, above)| Command::LayerNew(LayerNew {
                    layer,
                    kind: Some(LayerKind::Raster),
                    name,
                    above,
                    into: None
                })),
            id().prop_map(|layer| Command::LayerDelete(LayerDelete { layer })),
            (id(), proptest::option::of(id())).prop_map(|(layer, above)| Command::LayerMove(
                LayerMove {
                    layer,
                    above,
                    to: Some(Place::Top),
                    into: None,
                    below: None,
                }
            )),
            (
                id(),
                proptest::option::of(name()),
                proptest::option::of(0u8..=100),
                proptest::option::of(blend()),
                proptest::option::of(any::<bool>()),
                proptest::option::of(any::<bool>())
            )
                .prop_map(|(layer, name, opacity, blend, locked, lock_alpha)| {
                    Command::LayerSet(LayerSet {
                        layer,
                        name,
                        opacity,
                        blend,
                        locked,
                        lock_alpha,
                        clip: None,
                    })
                }),
            (id(), any::<bool>())
                .prop_map(|(layer, visible)| Command::LayerShow(LayerShow { layer, visible })),
            id().prop_map(|layer| Command::LayerSelect(LayerSelect { layer })),
            Just(Command::HistoryUndo(HistoryUndo {})),
            (id(), any::<[u8; 4]>()).prop_map(|(layer, c)| Command::LayerFill(LayerFill {
                layer,
                color: Rgba8(c)
            })),
            (
                id(),
                -300i64..300,
                -300i64..300,
                0u32..4,
                0u32..4,
                proptest::collection::vec(any::<u8>(), 0..64)
            )
                .prop_map(|(layer, x, y, w, h, data)| Command::LayerPixels(
                    LayerPixels {
                        layer,
                        x,
                        y,
                        w,
                        h,
                        data: brushye_command::params::Blob(data)
                    }
                )),
            (
                id(),
                any::<[u8; 3]>(),
                0.1f64..500.0,
                proptest::option::of(proptest::sample::select(vec![
                    brushye_core::brush::Aa::None,
                    brushye_core::brush::Aa::Strong
                ])),
                proptest::option::of(1u8..=5),
                proptest::option::of(prop_oneof![
                    Just(brushye_core::brush::Gap::Narrow),
                    (0.1f64..1000.0).prop_map(brushye_core::brush::Gap::Fixed)
                ]),
                proptest::option::of(0u8..=100),
                proptest::collection::vec((0.01f64..0.99, 0.0f64..=1.0), 0..4),
                proptest::collection::vec((-50.0f64..500.0, -50.0f64..500.0, 0.0f32..=1.0), 1..20),
            )
                .prop_map(
                    |(layer, c, size, aa, hardness, gap, size_pressure, pts, smp)| {
                        let mut xs: Vec<f64> = pts.iter().map(|p| p.0).collect();
                        xs.sort_by(f64::total_cmp);
                        xs.dedup();
                        let curve = brushye_core::brush::Curve::new(
                            xs.iter().zip(&pts).map(|(&x, p)| [x, p.1]).collect(),
                        )
                        .ok()
                        .filter(|_| size_pressure.is_some());
                        let samples = smp
                            .iter()
                            .enumerate()
                            .map(|(i, &(x, y, p))| brushye_core::input::Sample {
                                x,
                                y,
                                t_us: i as u32 * 4000,
                                pressure: Some(p),
                                tilt: None,
                                twist: None,
                                eraser: false,
                                barrel: [false; 2],
                            })
                            .collect::<Vec<_>>();
                        Command::BrushStroke(BrushStroke {
                            layer,
                            color: Rgba8([c[0], c[1], c[2], 255]),
                            size,
                            aa,
                            hardness,
                            opacity: None,
                            density: Some(50),
                            ink: None,
                            gap,
                            density_by_gap: None,
                            thickness: Some(40),
                            direction: None,
                            angle: Some(22.5),
                            angle_line: Some(true),
                            stabilization: Some(5),
                            size_pressure,
                            size_curve: curve,
                            opacity_pressure: None,
                            opacity_curve: None,
                            density_pressure: None,
                            density_curve: None,
                            samples: brushye_command::params::Samples(
                                brushye_core::input::canonical(&samples).unwrap_or_default(),
                            ),
                        })
                    }
                ),
        ]
    }

    proptest! {
        #[test]
        fn text_form_round_trips(cmd in command(), seq in proptest::option::of(1u64..1_000_000), merge in any::<bool>()) {
            let mut raw = cmd.to_raw();
            raw.seq = seq;
            raw.merge = merge && seq.is_some();
            let text = print_line(&raw);
            let Ok(Line::Item(back)) = parse_line(&text) else { panic!("unparsable: {text}") };
            prop_assert_eq!(print_line(&back), text.clone());
            prop_assert_eq!(&back, &raw);
            prop_assert_eq!(Command::from_raw(&back), Ok(cmd));
        }

        #[test]
        fn json_form_round_trips(cmd in command()) {
            let j = cmd.to_json();
            prop_assert_eq!(Command::from_json(&j), Ok(cmd));
        }
    }
}

/// Undo/redo fuzzing (M5): random command sequences against a model of the history.
mod undo_fuzz {
    use super::*;
    use proptest::prelude::*;

    #[derive(Clone, Debug)]
    enum Op {
        New(u8),
        Delete(u8),
        Move(u8, u8),
        Opacity(u8, u8),
        Blend(u8, bool),
        Lock(u8, bool),
        LockAlpha(u8, bool),
        Fill(u8, [u8; 4]),
        Pixels(u8, i16, i16, u8),
        Stroke(u8, Vec<(u8, u8, u8)>, u8),
        Show(u8, bool),
        Select(u8),
        Undo,
        Redo,
    }

    fn op() -> impl Strategy<Value = Op> {
        let l = 1u8..5;
        prop_oneof![
            l.clone().prop_map(Op::New),
            l.clone().prop_map(Op::Delete),
            (l.clone(), l.clone()).prop_map(|(a, b)| Op::Move(a, b)),
            (l.clone(), 0u8..=100).prop_map(|(a, o)| Op::Opacity(a, o)),
            (l.clone(), any::<bool>()).prop_map(|(a, b)| Op::Blend(a, b)),
            (l.clone(), any::<bool>()).prop_map(|(a, b)| Op::Lock(a, b)),
            (l.clone(), any::<bool>()).prop_map(|(a, b)| Op::LockAlpha(a, b)),
            (l.clone(), any::<[u8; 4]>()).prop_map(|(a, c)| Op::Fill(a, c)),
            (l.clone(), -10i16..70, -10i16..70, any::<u8>())
                .prop_map(|(a, x, y, s)| Op::Pixels(a, x, y, s)),
            (
                l.clone(),
                proptest::collection::vec((0u8..64, 0u8..64, 0u8..=255), 1..6),
                0u8..35
            )
                .prop_map(|(a, pts, ink)| Op::Stroke(a, pts, ink)),
            (l.clone(), any::<bool>()).prop_map(|(a, v)| Op::Show(a, v)),
            l.clone().prop_map(Op::Select),
            Just(Op::Undo),
            Just(Op::Undo),
            Just(Op::Redo),
        ]
    }

    fn line(op: &Op) -> String {
        use brushye_core::brush::InkMode;
        match op {
            Op::New(a) => format!("layer.new L{a}"),
            Op::Delete(a) => format!("layer.delete L{a}"),
            Op::Move(a, b) => format!("layer.move L{a} above=L{b}"),
            Op::Opacity(a, o) => format!("layer.set L{a} opacity={o}"),
            Op::Blend(a, m) => format!(
                "layer.set L{a} blend={}",
                if *m { "multiply" } else { "screen" }
            ),
            Op::Lock(a, v) => format!("layer.set L{a} locked={v}"),
            Op::LockAlpha(a, v) => format!("layer.set L{a} lock_alpha={v}"),
            Op::Fill(a, c) => format!(
                "layer.fill L{a} color=#{:02x}{:02x}{:02x}{:02x}",
                c[0], c[1], c[2], c[3]
            ),
            Op::Pixels(a, x, y, s) => format!(
                "layer.pixels L{a} x={x} y={y} w=1 h=2 data=b64:{}",
                ["AAAA/wAAAP8=", "/wAAgP8AAIA=", "EiNFZ3iavN4="][usize::from(*s) % 3]
            ),
            Op::Stroke(a, pts, ink) => {
                let rows: Vec<String> = pts
                    .iter()
                    .enumerate()
                    .map(|(i, (x, y, p))| format!("{x},{y},{},{}", f64::from(*p) / 255.0, i * 4000))
                    .collect();
                format!(
                    "brush.stroke L{a} color=#305070 size=6 ink={} samples=[{}]",
                    InkMode::ALL[usize::from(*ink) % InkMode::ALL.len()].name(),
                    rows.join(";")
                )
            }
            Op::Show(a, v) => format!("layer.show L{a} visible={v}"),
            Op::Select(a) => format!("layer.select L{a}"),
            Op::Undo => "history.undo".into(),
            Op::Redo => "history.redo".into(),
        }
    }

    fn pos(s: &mut Session) -> usize {
        q(s, "?history")["position"].as_u64().unwrap_or(0) as usize
    }

    proptest! {
        // BRUSHYE_FUZZ_CASES=5000 for a deep run.
        #![proptest_config(ProptestConfig::with_cases(
            std::env::var("BRUSHYE_FUZZ_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(48)
        ))]
        #[test]
        fn history_model_holds(ops in proptest::collection::vec(op(), 1..60)) {
            let mut s = Session::new();
            run(&mut s, "doc.new 64 64 paper=none");
            // model[p] = hash of the document at history position p, when known.
            let mut model: Vec<Option<String>> = vec![Some(q(&mut s, "?doc.hash").to_string())];
            for o in &ops {
                let (h0, p0, j0) = (q(&mut s, "?doc.hash"), pos(&mut s), s.journal_text());
                let text = line(o);
                let ok = s.run_line(&text).is_ok();
                let (h, p) = (q(&mut s, "?doc.hash").to_string(), pos(&mut s));
                if !ok {
                    // A failed command changes nothing and is not journaled.
                    prop_assert_eq!(q(&mut s, "?doc.hash"), h0);
                    prop_assert_eq!(p, p0);
                    prop_assert_eq!(s.journal_text(), j0);
                    continue;
                }
                match o {
                    Op::Undo | Op::Redo => {
                        if let Some(Some(want)) = model.get(p) {
                            prop_assert_eq!(&h, want, "{} at position {}", text, p);
                        }
                        if model.len() <= p {
                            model.resize(p + 1, None);
                        }
                        model[p] = Some(h);
                    }
                    Op::Select(_) => prop_assert_eq!(h, h0.to_string()),
                    Op::Show(..) => {
                        // Visibility is not undoable: it changes every version.
                        model = vec![None; model.len()];
                        model[p] = Some(h);
                        prop_assert_eq!(p, p0);
                    }
                    _ => {
                        // A new step (or a merge into the current one) drops the redo branch.
                        model.truncate(p + 1);
                        model.resize(p + 1, None);
                        model[p] = Some(h);
                    }
                }
            }
            // Undo everything, redo everything: back to the same document.
            let end = q(&mut s, "?doc.hash");
            let n = pos(&mut s);
            for _ in 0..n { run(&mut s, "history.undo"); }
            prop_assert_eq!(pos(&mut s), 0);
            for _ in 0..n { run(&mut s, "history.redo"); }
            prop_assert_eq!(q(&mut s, "?doc.hash"), end.clone());
            // The journal replays to the same document, history and journal.
            let mut r = Session::replay(&s.journal_text()).map_err(|e| TestCaseError::fail(e.to_string()))?;
            prop_assert_eq!(q(&mut r, "?doc.hash"), end);
            prop_assert_eq!(q(&mut r, "?history"), q(&mut s, "?history"));
            prop_assert_eq!(q(&mut r, "?doc.info"), q(&mut s, "?doc.info"));
            prop_assert_eq!(r.journal_text(), s.journal_text());
        }
    }
}
