//! Folder, Through and clipping commands (D-023): structure, undo, journal replay, errors.

use brushye_command::Session;
use serde_json::Value as Json;

fn run(s: &mut Session, script: &str) -> Vec<Json> {
    s.run_script(script)
        .unwrap_or_else(|e| panic!("line {}: {}", e.line, e.msg))
}

fn q(s: &mut Session, query: &str) -> Json {
    s.run_line(query)
        .unwrap_or_else(|e| panic!("{query}: {e}"))
        .unwrap_or(Json::Null)
}

/// (id, parent) bottom to top.
fn tree(s: &mut Session) -> Vec<(String, String)> {
    q(s, "?layer.list")
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|l| {
            (
                l["id"].as_str().unwrap_or("").to_owned(),
                l["parent"].as_str().unwrap_or("-").to_owned(),
            )
        })
        .collect()
}

fn t(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

const SCRIPT: &str = "doc.new 64 64
layer.new L1
layer.fill L1 color=#204080
layer.new L2
layer.fill L2 color=#ff000080
layer.group L10 layer=L2
layer.new L3 into=L10 name=\"Clipped\"
layer.fill L3 color=#00ff00
layer.set L3 clip=true
layer.set L10 blend=through opacity=50
layer.expand L10 open=false";

#[test]
fn build_query_undo_and_replay() {
    let mut s = Session::new();
    run(&mut s, SCRIPT);
    assert_eq!(
        tree(&mut s),
        t(&[("L1", "-"), ("L2", "L10"), ("L3", "L10"), ("L10", "-")])
    );
    let f = q(&mut s, "?layer.get L10");
    assert_eq!(f["kind"], "folder");
    assert_eq!(f["blend"], "through");
    assert_eq!(f["expanded"], false);
    assert_eq!(q(&mut s, "?layer.get L3")["clip"], true);
    assert_eq!(q(&mut s, "?doc.info")["active"], "L3");

    // Folder open/close is not a History step: undo goes back past the folder settings.
    let steps = q(&mut s, "?history")["steps"].as_array().map(Vec::len);
    run(&mut s, "history.undo");
    assert_eq!(q(&mut s, "?layer.get L10")["blend"], "normal");
    assert_eq!(q(&mut s, "?layer.get L10")["expanded"], false);
    run(&mut s, "history.redo");
    assert_eq!(
        q(&mut s, "?history")["steps"].as_array().map(Vec::len),
        steps
    );

    // The journal replays to the same document.
    let hash = q(&mut s, "?doc.hash");
    let r = Session::replay(&s.journal_text()).unwrap_or_else(|e| panic!("{}: {}", e.line, e.msg));
    let mut r = r;
    assert_eq!(q(&mut r, "?doc.hash"), hash);

    // Ungroup keeps the contents in place and selects the topmost child.
    run(&mut s, "layer.ungroup L10");
    assert_eq!(tree(&mut s), t(&[("L1", "-"), ("L2", "-"), ("L3", "-")]));
    assert_eq!(q(&mut s, "?doc.info")["active"], "L3");
    run(&mut s, "history.undo");
    assert_eq!(
        tree(&mut s),
        t(&[("L1", "-"), ("L2", "L10"), ("L3", "L10"), ("L10", "-")])
    );

    // Moving: out to the top, into a folder; deleting a folder deletes its contents.
    run(&mut s, "layer.move L3 to=top\nlayer.move L1 into=L10");
    assert_eq!(
        tree(&mut s),
        t(&[("L2", "L10"), ("L1", "L10"), ("L10", "-"), ("L3", "-")])
    );
    run(&mut s, "layer.delete L10");
    assert_eq!(tree(&mut s), t(&[("L3", "-")]));
}

#[test]
fn ctrl_g_selects_the_folder() {
    let mut s = Session::new();
    run(&mut s, "doc.new 8 8\nlayer.new L1\nlayer.group L2 layer=L1");
    assert_eq!(q(&mut s, "?doc.info")["active"], "L2");
    assert_eq!(q(&mut s, "?layer.get L2")["name"], "Folder 2");
}

#[test]
fn errors() {
    let mut s = Session::new();
    run(
        &mut s,
        "doc.new 8 8\nlayer.new L1\nlayer.group L5 layer=L1\nlayer.new L2",
    );
    for bad in [
        "layer.set L1 blend=through",
        "layer.fill L5 color=#ff0000",
        "brush.stroke L5 color=#000000 size=3 samples=[1,1,1,0;5,5,1,1000]",
        "layer.move L5 into=L1",
        "layer.move L5 into=L5",
        "layer.new L3 into=L1",
        "layer.new L3 above=L1 into=L5",
        "layer.ungroup L1",
        "layer.expand L1 open=false",
        "layer.group L1 layer=L2",
    ] {
        assert!(s.run_line(bad).is_err(), "{bad} should fail");
    }
}

#[test]
fn json_form_of_new_commands() {
    use brushye_command::Command;
    for line in [
        "layer.group L4 layer=L2 name=\"F\"",
        "layer.ungroup L4",
        "layer.expand L4 open=true",
        "layer.set L2 clip=true",
        "layer.new L3 into=L4",
        "layer.move L3 into=L4",
    ] {
        let raw = match brushye_command::syntax::parse_line(line) {
            Ok(brushye_command::syntax::Line::Item(r)) => r,
            other => panic!("{other:?}"),
        };
        let cmd = Command::from_raw(&raw).unwrap_or_else(|e| panic!("{line}: {e}"));
        assert_eq!(
            Command::from_json(&cmd.to_json()),
            Ok(cmd.clone()),
            "{line}"
        );
        assert_eq!(brushye_command::syntax::print_line(&cmd.to_raw()), line);
    }
}
