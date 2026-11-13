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
            if k == "opacity" {
                if let Value::Num(n) = v {
                    let pct = (n.parse::<f64>().map_err(|e| e.to_string())? * 100.0).round();
                    *v = Value::Num(format!("{pct}"));
                }
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
                    above
                })),
            id().prop_map(|layer| Command::LayerDelete(LayerDelete { layer })),
            (id(), proptest::option::of(id())).prop_map(|(layer, above)| Command::LayerMove(
                LayerMove {
                    layer,
                    above,
                    to: Some(Place::Top)
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
