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
