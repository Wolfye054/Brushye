//! Attach mode (D-022): `brushye-mcp --attach` forwards tool calls over the local endpoint to a
//! host. Here the host is a headless `Server` on a thread, standing in for the app.

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

use brushye_mcp::attach::{Client, Endpoint, endpoints};
use brushye_mcp::{Host as _, Out, Server};
use serde_json::{Map, Value as Json, json};

fn tmpdir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("brushye-attach-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// Start an endpoint whose host is a headless session; returns it and the host thread.
fn host(dir: &std::path::Path, pid: u32) -> (Endpoint, std::thread::JoinHandle<Server>) {
    let (ep, rx) = Endpoint::start_in(dir, pid, Box::new(|| {}))
        .unwrap_or_else(|e| unreachable!("start: {e}"));
    let t = std::thread::spawn(move || {
        let mut s = Server::new();
        for req in rx {
            let r = s.call(&req.tool, &req.args);
            req.reply(r);
        }
        s
    });
    (ep, t)
}

#[test]
fn client_round_trip_and_discovery() {
    let dir = tmpdir("client");
    assert!(Client::connect(&dir, None).is_err());
    let (ep, t) = host(&dir, 4101);
    assert_eq!(endpoints(&dir).len(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&dir)
            .map(|m| m.permissions().mode())
            .unwrap_or(0);
        assert_eq!(mode & 0o777, 0o700);
    }
    let mut c = Client::connect(&dir, None).unwrap_or_else(|e| unreachable!("{e}"));
    assert_eq!(c.pid, 4101);
    let mut args = Map::new();
    args.insert("script".into(), json!("doc.new 30 20\n?doc.info"));
    let r = c.call("run", &args).unwrap_or_else(|e| unreachable!("{e}"));
    let Ok(out) = r else { unreachable!("{r:?}") };
    assert!(matches!(&out[0], Out::Text(t) if t.contains("\"width\":30")));
    // Images survive the wire.
    let r = c
        .call("render", &Map::new())
        .unwrap_or_else(|e| unreachable!("{e}"));
    assert!(matches!(r.as_deref(), Ok([Out::Png(p), Out::Text(_)]) if &p[1..4] == b"PNG"));
    // Tool errors come back as errors, not connection failures.
    args.insert("script".into(), json!("layer.delete L9"));
    assert!(c.call("run", &args).is_ok_and(|r| r.is_err()));
    assert!(c.call("nope", &Map::new()).is_ok_and(|r| r.is_err()));
    // Wait until the connection is counted.
    for _ in 0..50 {
        if ep.connected() == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(ep.connected(), 1);

    // A second host: connecting without a pid is ambiguous; with one it works.
    let (ep2, t2) = host(&dir, 4102);
    let e = Client::connect(&dir, None).err().unwrap_or_default();
    assert!(e.contains("--attach PID"), "{e}");
    assert!(Client::connect(&dir, Some(4102)).is_ok_and(|c| c.pid == 4102));
    drop(ep2);
    drop(t2.join());

    // Turning the endpoint off: discovery file gone, the open connection gets an error.
    drop(ep);
    assert!(endpoints(&dir).is_empty());
    let r = c.call("journal", &Map::new());
    assert!(!matches!(r, Ok(Ok(_))), "{r:?}");
    let s = t.join().unwrap_or_else(|_| unreachable!());
    assert_eq!(s.session().document().map(|d| d.width), Some(30));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mcp_binary_attaches() {
    let dir = tmpdir("bin");
    let mut child = Command::new(env!("CARGO_BIN_EXE_brushye-mcp"))
        .arg("--attach")
        .env(brushye_mcp::attach::ENV_DIR, &dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| unreachable!("spawn: {e}"));
    let mut stdin = child.stdin.take().unwrap_or_else(|| unreachable!());
    let mut stdout = BufReader::new(child.stdout.take().unwrap_or_else(|| unreachable!()));
    let mut rpc = |id: u64, method: &str, params: Json| -> Json {
        writeln!(
            stdin,
            "{}",
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        )
        .unwrap_or_else(|e| unreachable!("{e}"));
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .unwrap_or_else(|e| unreachable!("{e}"));
        if method == "initialize" {
            writeln!(
                stdin,
                "{}",
                json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
            )
            .unwrap_or_else(|e| unreachable!("{e}"));
        }
        serde_json::from_str(&line).unwrap_or_else(|e| unreachable!("{e}: {line}"))
    };
    let init = rpc(
        1,
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "test", "version": "0"}}),
    );
    assert!(
        init["result"]["instructions"]
            .as_str()
            .unwrap_or("")
            .contains("attached to a running Brushye window")
    );
    let call = |name: &str, args: Json| json!({"name": name, "arguments": args});

    // No app listening yet: a tool error that says what to do.
    let r = rpc(2, "tools/call", call("journal", json!({})));
    assert_eq!(r["result"]["isError"], true);
    assert!(
        r["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .contains("Allow AI clients")
    );

    // The app turns AI access on: the next call connects.
    let (ep, t) = host(&dir, 4201);
    let r = rpc(
        3,
        "tools/call",
        call("run", json!({"script": "doc.new 40 30\nlayer.new L2"})),
    );
    assert_eq!(r["result"]["isError"], false, "{r}");
    let r = rpc(4, "tools/call", call("render", json!({})));
    assert_eq!(r["result"]["content"][0]["type"], "image");

    // The app restarts its endpoint: the bridge reconnects transparently.
    drop(ep);
    let (ep, t2) = host(&dir, 4202);
    let r = rpc(
        5,
        "tools/call",
        call("run", json!({"script": "doc.new 8 8"})),
    );
    assert_eq!(r["result"]["isError"], false, "{r}");
    // The old connection closed on that request, so the first host has finished.
    let s = t.join().unwrap_or_else(|_| unreachable!());
    assert_eq!(s.session().document().map(|d| d.layers().len()), Some(1));
    let t = t2;

    drop(stdin);
    let status = child.wait().unwrap_or_else(|e| unreachable!("{e}"));
    assert!(status.success());
    drop(ep);
    drop(t.join());
    let _ = std::fs::remove_dir_all(&dir);
}
