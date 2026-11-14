//! The MCP server core, called directly (no protocol).

use brushye_mcp::{Out, Server, tools};
use serde_json::{Map, Value as Json, json};

fn args(j: Json) -> Map<String, Json> {
    match j {
        Json::Object(m) => m,
        _ => unreachable!(),
    }
}

fn text(out: &[Out]) -> String {
    out.iter()
        .filter_map(|o| match o {
            Out::Text(t) => Some(t.as_str()),
            Out::Png(_) => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn call(s: &mut Server, name: &str, a: Json) -> Result<Vec<Out>, String> {
    s.call(name, &args(a))
}

#[test]
fn tool_list_is_well_formed() {
    let t = tools();
    let names: Vec<_> = t.iter().map(|t| t.name).collect();
    assert_eq!(
        names,
        [
            "help",
            "run",
            "command",
            "query",
            "render",
            "stroke",
            "journal",
            "doc_open",
            "doc_save",
            "export_png",
            "replay"
        ]
    );
    for t in &t {
        assert_eq!(t.schema["type"], "object", "{}", t.name);
        assert!(t.name.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }
}

#[test]
fn run_command_query_and_journal() {
    let mut s = Server::new();
    let out = call(
        &mut s,
        "run",
        json!({"script": "doc.new 64 48\nlayer.new L2 name=\"Ink\"\n?layer.list"}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    let layers: Json = serde_json::from_str(&text(&out)).unwrap_or_default();
    assert_eq!(layers[0].as_array().map(Vec::len), Some(1));

    // JSON form, with readable stroke samples.
    call(
        &mut s,
        "command",
        json!({"cmd": "brush.stroke", "args": {
            "layer": "L2", "color": "#ff0000", "size": 6.0,
            "samples": [[10, 10, 1, 0], [50, 30, 1, 10000]]
        }}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    let px = call(
        &mut s,
        "query",
        json!({"query": "layer.pixel L2 x=30 y=20"}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    let px: Json = serde_json::from_str(&text(&px)).unwrap_or_default();
    assert_eq!(px[0], 255, "stroke painted red: {px}");

    let journal = text(&call(&mut s, "journal", json!({})).unwrap_or_default());
    assert!(journal.starts_with("#brushye-journal v1"));
    assert_eq!(journal.lines().count(), 4, "{journal}");

    // Replaying the journal gives the same document.
    let hash = text(&call(&mut s, "query", json!({"query": "?doc.hash"})).unwrap_or_default());
    let mut r = Server::new();
    call(&mut r, "replay", json!({"text": journal})).unwrap_or_else(|e| unreachable!("{e}"));
    let hash2 = text(&call(&mut r, "query", json!({"query": "?doc.hash"})).unwrap_or_default());
    assert_eq!(hash, hash2);
}

#[test]
fn errors_are_reported_with_lines() {
    let mut s = Server::new();
    let e = call(
        &mut s,
        "run",
        json!({"script": "doc.new 10 10\n?doc.info\nlayer.bogus"}),
    )
    .err()
    .unwrap_or_default();
    assert!(e.starts_with("line 3: layer.bogus:"), "{e}");
    assert!(e.contains("query results before the error"), "{e}");
    // Earlier lines stay applied.
    assert!(call(&mut s, "query", json!({"query": "?doc.info"})).is_ok());
    assert!(call(&mut s, "run", json!({"script": "x", "extra": 1})).is_err());
    assert!(call(&mut s, "nope", json!({})).is_err());
    assert!(
        call(
            &mut s,
            "command",
            json!({"cmd": "layer.set", "args": {"layer": "L9"}})
        )
        .is_err()
    );
}

#[test]
fn render_files_and_reopen() {
    let dir = std::env::temp_dir().join(format!("brushye-mcp-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_default();
    let mut s = Server::new();
    call(
        &mut s,
        "run",
        json!({"script": "doc.new 300 200\nlayer.new L2\nlayer.fill L2 color=#00ff0080"}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));

    let out = call(&mut s, "render", json!({"max_size": 100})).unwrap_or_default();
    let Some(Out::Png(png)) = out.first() else {
        unreachable!()
    };
    assert_eq!(&png[1..4], b"PNG");
    assert!(
        text(&out).contains("downscaled to 100x66"),
        "{}",
        text(&out)
    );
    let out = call(
        &mut s,
        "render",
        json!({"layer": "L2", "x": 10, "y": 10, "w": 20, "h": 5}),
    )
    .unwrap_or_default();
    assert_eq!(text(&out), "20x5 px");
    assert!(call(&mut s, "render", json!({"x": 290, "w": 20})).is_err());

    let brsh = dir.join("a.brsh").to_string_lossy().into_owned();
    let png = dir.join("a.png").to_string_lossy().into_owned();
    call(
        &mut s,
        "doc_save",
        json!({"path": brsh, "embed_journal": true}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    call(&mut s, "export_png", json!({"path": png})).unwrap_or_else(|e| unreachable!("{e}"));
    assert!(
        std::fs::metadata(&png)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    );
    let hash = text(&call(&mut s, "query", json!({"query": "doc.hash"})).unwrap_or_default());

    let mut o = Server::new();
    let msg = text(&call(&mut o, "doc_open", json!({"path": brsh})).unwrap_or_default());
    assert!(
        msg.contains("1 layer;") && msg.contains("embeds a journal"),
        "{msg}"
    );
    let hash2 = text(&call(&mut o, "query", json!({"query": "doc.hash"})).unwrap_or_default());
    assert_eq!(hash, hash2);
    // A journal recorded on top of the opened file replays on that base.
    call(&mut o, "run", json!({"script": "layer.set L2 opacity=50"})).unwrap_or_default();
    let j = text(&call(&mut o, "journal", json!({})).unwrap_or_default());
    let mut r = Server::new();
    assert!(call(&mut r, "replay", json!({"text": j})).is_err());
    call(&mut r, "replay", json!({"text": j, "base": brsh}))
        .unwrap_or_else(|e| unreachable!("{e}"));
    std::fs::remove_dir_all(&dir).unwrap_or_default();
}

#[test]
fn help_has_syntax_and_reference() {
    let h = text(&Server::new().call("help", &Map::new()).unwrap_or_default());
    assert!(h.contains("JSON form") && h.contains("brush.stroke"));
}

#[test]
fn stroke_uses_current_subtool_and_colour() {
    let mut s = Server::new();
    // No layer yet: an error, not a panic.
    call(&mut s, "run", json!({"script": "doc.new 64 64"})).unwrap_or_default();
    assert!(call(&mut s, "stroke", json!({"samples": [[1, 1, 1, 0]]})).is_err());
    call(
        &mut s,
        "run",
        json!({"script": "layer.new L2\nlayer.select L2\ntool.select pen\nsubtool.set size=9\ncolor.set main color=#0000ff"}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    let msg = text(
        &call(
            &mut s,
            "stroke",
            json!({"samples": [[10, 32, 1, 0], [54, 32, 1, 20000]]}),
        )
        .unwrap_or_else(|e| unreachable!("{e}")),
    );
    assert!(
        msg.contains("2 samples") && msg.contains("color=#0000ff") && msg.contains("size=9.0"),
        "{msg}"
    );
    let px: Json = serde_json::from_str(&text(
        &call(
            &mut s,
            "query",
            json!({"query": "layer.pixel L2 x=32 y=32"}),
        )
        .unwrap_or_default(),
    ))
    .unwrap_or_default();
    assert_eq!(px, json!([0, 0, 255, 255]));

    // The transparent colour erases.
    call(
        &mut s,
        "run",
        json!({"script": "color.active transparent\nsubtool.set size=20"}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    call(
        &mut s,
        "stroke",
        json!({"samples": [[10, 32, 1, 0], [54, 32, 1, 20000]]}),
    )
    .unwrap_or_else(|e| unreachable!("{e}"));
    let px: Json = serde_json::from_str(&text(
        &call(
            &mut s,
            "query",
            json!({"query": "layer.pixel L2 x=32 y=32"}),
        )
        .unwrap_or_default(),
    ))
    .unwrap_or_default();
    assert_eq!(px[3], 0, "{px}");

    // A non-painting tool is refused.
    call(&mut s, "run", json!({"script": "tool.select hand"})).unwrap_or_default();
    assert!(call(&mut s, "stroke", json!({"samples": [[1, 1, 1, 0]]})).is_err());
}

/// Every script in `examples/scripts/` runs cleanly through the `run` tool.
#[test]
fn example_scripts_run() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/scripts");
    let mut n = 0;
    for e in std::fs::read_dir(&dir).unwrap_or_else(|e| unreachable!("{e}")) {
        let p = e.unwrap_or_else(|e| unreachable!("{e}")).path();
        if p.extension().is_some_and(|x| x == "brushye") {
            let script = std::fs::read_to_string(&p).unwrap_or_default();
            let mut s = Server::new();
            if let Err(e) = call(&mut s, "run", json!({ "script": script })) {
                unreachable!("{}: {e}", p.display());
            }
            n += 1;
        }
    }
    assert!(n >= 3, "found {n} example scripts");
}
