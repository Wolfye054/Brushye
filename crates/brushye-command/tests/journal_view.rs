//! The journal viewer model (M9).

use brushye_command::Session;
use brushye_command::journal::view;

const SCRIPT: &str = "doc.new 64 64
layer.new L2
brush.stroke L2 color=#000000 size=4 samples=[4,4,0.25,0;60,30,0.75,500000]
layer.set L2 opacity=50
layer.set L2 opacity=40
layer.new L3
history.undo
history.undo
history.redo
layer.show L2 visible=false";

fn journal() -> String {
    let mut s = Session::new();
    s.run_script(SCRIPT)
        .unwrap_or_else(|e| unreachable!("{e:?}"));
    s.journal_text()
}

#[test]
fn entries_strokes_and_undo_state() {
    let j = journal();
    let v = view(&j);
    assert!(v.errors.is_empty(), "{:?}", v.errors);
    assert!(
        v.header
            .as_deref()
            .is_some_and(|h| h.starts_with("#brushye-journal v1"))
    );
    let names: Vec<_> = v.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "doc.new",
            "layer.new",
            "brush.stroke",
            "layer.set",
            "layer.set",
            "layer.new",
            "history.undo",
            "history.undo",
            "history.redo",
            "layer.show"
        ]
    );
    // The second opacity change merged into the first (spec H-001).
    assert!(v.entries[4].merge);
    let s = v.entries[2]
        .stroke
        .as_ref()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(s.samples, 2);
    assert_eq!(s.bounds, [4.0, 4.0, 60.0, 30.0]);
    assert_eq!(s.duration_ms, 500.0);
    assert!(
        v.entries[2]
            .text
            .contains("<2 samples, 0.50 s, (4,4)–(60,30), pressure 0.25–0.75>"),
        "{}",
        v.entries[2].text
    );
    assert!(!v.entries[2].text.contains("b64:"));
    // Undo, undo, redo: layer.new L3 is undone; the merged opacity step is not.
    let undone: Vec<_> = v
        .entries
        .iter()
        .filter(|e| e.undone)
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(undone, ["layer.new"]);
    assert_eq!(v.entries[5].text, "@6 layer.new L3");

    // Agrees with the session's own history after replay.
    let r = Session::replay(&j).unwrap_or_else(|e| unreachable!("{e:?}"));
    let h = r
        .query(
            &brushye_command::Query::from_raw(
                &match brushye_command::syntax::parse_line("?history") {
                    Ok(brushye_command::syntax::Line::Item(raw)) => raw,
                    _ => unreachable!(),
                },
            )
            .unwrap_or_else(|e| unreachable!("{e}")),
        )
        .unwrap_or_default();
    let steps = h["steps"].as_array().map(Vec::len).unwrap_or(0);
    let pos = h["position"].as_u64().unwrap_or(0) as usize;
    assert_eq!(steps - pos - 1, 1);
}

#[test]
fn bad_lines_are_reported_not_fatal() {
    let v =
        view("#brushye-journal v1 app=x\n@1 doc.new 10 10\n@2 layer.new \"oops\n@3 bogus.cmd\n");
    assert_eq!(v.errors.len(), 1);
    assert_eq!(v.errors[0].0, 3);
    assert_eq!(v.entries.len(), 2);
    assert_eq!(v.entries[1].class, None);
    assert!(v.to_json()["entries"].as_array().is_some());
}
