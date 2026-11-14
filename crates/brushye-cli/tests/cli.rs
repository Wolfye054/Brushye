//! End-to-end: run a script, write its journal, replay it, compare hashes (M2 exit criterion).

use std::process::Command;

fn brushye(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    use std::io::Write as _;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_brushye"));
    cmd.args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().unwrap_or_else(|e| panic!("spawn: {e}"));
    if let (Some(input), Some(mut si)) = (stdin, child.stdin.take()) {
        si.write_all(input.as_bytes())
            .unwrap_or_else(|e| panic!("stdin: {e}"));
    }
    let out = child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("wait: {e}"));
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn run_then_replay_gives_the_same_hash() {
    let dir = std::env::temp_dir().join(format!("brushye-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let journal = dir.join("session.bcmd");
    let script = "doc.new 640 480\nlayer.new L1 name=\"Ink\"\nlayer.set L1 blend=screen\nlayer.set L1 opacity=40\nhistory.undo\n?doc.hash\n";
    let (code, out, err) = brushye(
        &["run", "-", "--journal", journal.to_str().unwrap_or("")],
        Some(script),
    );
    assert_eq!(code, 0, "{err}");
    let hash_run = out.trim().to_owned();
    assert!(
        hash_run.starts_with('"') && hash_run.len() == 18,
        "{hash_run}"
    );
    let (code, out, err) = brushye(
        &[
            "replay",
            journal.to_str().unwrap_or(""),
            "--query",
            "?doc.hash",
        ],
        None,
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.trim(), hash_run);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn errors_report_file_and_line() {
    let (code, _, err) = brushye(
        &["run", "-"],
        Some("doc.new 10 10\nlayer.set L7 opacity=3\n"),
    );
    assert_eq!(code, 1);
    assert!(err.contains("-:2:") && err.contains("no layer L7"), "{err}");
    let (code, _, _) = brushye(&["frob"], None);
    assert_eq!(code, 2);
}

#[test]
fn commands_lists_the_registry() {
    let (code, out, _) = brushye(&["commands"], None);
    assert_eq!(code, 0);
    let j: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}"));
    let names: Vec<&str> = j["commands"]
        .as_array()
        .map(|a| a.iter().filter_map(|c| c["name"].as_str()).collect())
        .unwrap_or_default();
    assert!(names.contains(&"layer.set") && names.contains(&"history.undo"));
}

#[test]
fn render_exports_the_flattened_image() {
    let dir = std::env::temp_dir().join(format!("brushye-cli-render-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let (journal, png_run, png_render) = (
        dir.join("j.bcmd"),
        dir.join("run.png"),
        dir.join("render.png"),
    );
    // White paper, a 50 % gray Multiply layer at 50 % opacity.
    let script = "doc.new 3 2 paper=#ffffff\nlayer.new L1\nlayer.fill L1 color=#808080\nlayer.set L1 blend=multiply opacity=50\n";
    let s = |p: &std::path::Path| p.to_str().unwrap_or("").to_owned();
    let (code, _, err) = brushye(
        &["run", "-", "--journal", &s(&journal), "--png", &s(&png_run)],
        Some(script),
    );
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = brushye(&["render", &s(&journal), &s(&png_render)], None);
    assert_eq!(code, 0, "{err}");
    let a = std::fs::read(&png_run).unwrap_or_default();
    let b = std::fs::read(&png_render).unwrap_or_default();
    assert_eq!(a, b, "run --png and render must write identical bytes");
    let img = brushye_format::png_io::read_png(&a).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((img.width, img.height), (3, 2));
    // Multiply(255, 128) = 128; opacity 50 % → o8 = 127, ae8 = (255·128) >> 8 = 127,
    // a15 = 16320; 15-bit lerp from 255 to 128 → 192 (L-002/L-003).
    assert_eq!(img.pixel(0, 0), [192, 192, 192, 255]);
    let _ = std::fs::remove_dir_all(&dir);
}

fn tmpdir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("brushye-cli-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap_or_else(|e| panic!("{e}"));
    d
}

fn p(d: &std::path::Path, f: &str) -> String {
    d.join(f).to_string_lossy().into_owned()
}

const DOC: &str = "doc.new 300 200\nlayer.new L1\nlayer.fill L1 color=#d0e0f0\nlayer.new L2\nbrush.stroke L2 color=#102030 size=12 samples=[20,20,1,0;280,180,1,8000]\n?doc.hash\n";

#[test]
fn save_open_dump_render_and_continue_a_document() {
    let d = tmpdir("brsh");
    let (doc, j1, j2, png_a, png_b) = (
        p(&d, "a.brsh"),
        p(&d, "a.bcmd"),
        p(&d, "more.bcmd"),
        p(&d, "a.png"),
        p(&d, "b.png"),
    );
    let (code, out, err) = brushye(
        &[
            "run",
            "-",
            "--brsh",
            &doc,
            "--embed-journal",
            "--journal",
            &j1,
            "--png",
            &png_a,
        ],
        Some(DOC),
    );
    assert_eq!(code, 0, "{err}");
    let h = out.trim().to_owned();
    // Open answers queries on the saved document.
    let (code, out, err) = brushye(
        &["open", &doc, "--query", "?doc.hash", "--query", "?doc.info"],
        None,
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.lines().next(), Some(h.as_str()));
    assert!(out.contains("\"active\":\"L2\""), "{out}");
    // Rendering the file and the journal gives the same bytes as the run's PNG.
    assert_eq!(brushye(&["render", &doc, &png_b], None).0, 0);
    let a = std::fs::read(&png_a).unwrap_or_default();
    assert_eq!(a, std::fs::read(&png_b).unwrap_or_default());
    assert_eq!(brushye(&["render", &j1, &png_b], None).0, 0);
    assert_eq!(a, std::fs::read(&png_b).unwrap_or_default());
    // Dump: manifest data, layer statistics, embedded journal, layer PNGs.
    let layers = p(&d, "layers");
    let (code, out, err) = brushye(&["dump", &doc, "--layers", &layers], None);
    assert_eq!(code, 0, "{err}");
    let j: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    assert_eq!(j["format_version"], brushye_format::brsh::FORMAT_VERSION);
    assert_eq!(format!("\"{}\"", j["hash"].as_str().unwrap_or("")), h);
    assert_eq!(j["layers"][0]["tiles"], 2);
    assert_eq!(j["journal_lines"], 6);
    assert!(d.join("layers").join("L2.png").exists());
    // Continue the document: the new journal has a base and replays only on it.
    let (code, out, err) = brushye(
        &["run", "-", "--open", &doc, "--journal", &j2],
        Some("layer.set L2 opacity=50\n?doc.hash\n"),
    );
    assert_eq!(code, 0, "{err}");
    let h2 = out.trim().to_owned();
    assert_ne!(h2, h);
    let (code, out, err) = brushye(
        &["replay", &j2, "--base", &doc, "--query", "?doc.hash"],
        None,
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.trim(), h2);
    let (code, _, err) = brushye(&["replay", &j2], None);
    assert_eq!(code, 1);
    assert!(err.contains("base="), "{err}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn user_brushes_save_show_and_list() {
    let d = tmpdir("brush");
    let f = p(&d, "soft.brshbrush");
    let (code, _, err) = brushye(
        &[
            "brush",
            "new",
            &f,
            "Soft round",
            "size=24",
            "hardness=2",
            "gap=wide",
            "size_pressure=30",
        ],
        None,
    );
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = brushye(&["brush", "show", &f], None);
    assert_eq!(code, 0, "{err}");
    let j: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    assert_eq!(j["name"], "Soft round");
    assert_eq!(
        j["stroke_args"],
        "size=24.0 hardness=2 gap=wide size_pressure=30"
    );
    // The printed settings work on a brush.stroke line.
    let script = format!(
        "doc.new 64 64\nlayer.new L1\nbrush.stroke L1 color=#000000 {} samples=[10,10,1,0;50,50,0.5,5000]\n",
        j["stroke_args"].as_str().unwrap_or("")
    );
    assert_eq!(brushye(&["run", "-"], Some(&script)).0, 0);
    // Bad settings are refused; the library lists good and broken brushes.
    assert_eq!(
        brushye(
            &[
                "brush",
                "new",
                &p(&d, "bad.brshbrush"),
                "Bad",
                "size=5",
                "hardness=9"
            ],
            None
        )
        .0,
        1
    );
    std::fs::write(d.join("broken.brshbrush"), b"not a zip").unwrap_or_else(|e| panic!("{e}"));
    let (code, out, err) = brushye(&["brush", "list", &d.to_string_lossy()], None);
    assert_eq!(code, 0, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(lines[0].contains("error"), "{out}");
    assert!(lines[1].contains("Soft round"), "{out}");
    let _ = std::fs::remove_dir_all(&d);
}

/// docs/commands.md is the generated reference; regenerate with
/// `brushye commands --markdown > docs/commands.md` when commands change.
#[test]
fn command_reference_is_current() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/commands.md");
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    let (code, out, err) = brushye(&["commands", "--markdown"], None);
    assert_eq!(code, 0, "{err}");
    assert!(
        committed.replace("\r\n", "\n") == out,
        "docs/commands.md is out of date: run `cargo run -p brushye-cli -- commands --markdown > docs/commands.md`"
    );
    // Every command and query is documented.
    for name in [
        "brush.stroke",
        "layer.new",
        "input.pressure_curve",
        "?doc.hash",
    ] {
        assert!(out.contains(&format!("`{name}`")), "{name}");
    }
}

#[test]
fn journal_show_lists_commands() {
    let dir = tmpdir("journal-show");
    let j = p(&dir, "j.bcmd");
    let script = "doc.new 32 32\nlayer.new L2\nbrush.stroke L2 color=#000000 size=3 samples=[2,2,1,0;30,30,1,250000]\nlayer.new L3\nhistory.undo\n";
    let (code, _, err) = brushye(&["run", "-", "--journal", &j], Some(script));
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = brushye(&["journal", "show", &j], None);
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("<2 samples, 0.25 s, (2,2)–(30,30), pressure 1.00–1.00>"),
        "{out}"
    );
    assert!(out.contains("@4 layer.new L3  (undone)"), "{out}");
    assert!(
        out.ends_with("-- 5 commands, 1 strokes, 1 undone at the end\n"),
        "{out}"
    );
    let (code, out, _) = brushye(&["journal", "show", &j, "--json"], None);
    assert_eq!(code, 0);
    assert!(out.contains("\"undone\": true"));
    let (code, _, _) = brushye(&["journal", "list"], None);
    assert_eq!(code, 2);
    let _ = std::fs::remove_dir_all(&dir);
}
