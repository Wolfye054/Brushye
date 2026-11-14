//! The `brushye-mcp` binary, driven over stdio with raw JSON-RPC as an MCP client would.

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};

use serde_json::{Value as Json, json};

#[test]
fn stdio_session() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_brushye-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| unreachable!("spawn: {e}"));
    let mut stdin = child.stdin.take().unwrap_or_else(|| unreachable!());
    let mut stdout = BufReader::new(child.stdout.take().unwrap_or_else(|| unreachable!()));
    let mut send = |j: Json| {
        writeln!(stdin, "{j}").unwrap_or_else(|e| unreachable!("{e}"));
    };
    let mut recv = || -> Json {
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .unwrap_or_else(|e| unreachable!("{e}"));
        serde_json::from_str(&line).unwrap_or_else(|e| unreachable!("{e}: {line}"))
    };

    send(
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "test", "version": "0"}}}),
    );
    let init = recv();
    assert_eq!(init["result"]["serverInfo"]["name"], "brushye");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));

    send(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let list = recv();
    let tools = list["result"]["tools"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(tools.len(), brushye_mcp::tools().len());
    assert_eq!(tools[1]["name"], "run");
    assert_eq!(tools[0]["annotations"]["readOnlyHint"], true);

    send(
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
        "name": "run", "arguments": {"script": "doc.new 20 10\n?doc.info"}}}),
    );
    let r = recv();
    assert_eq!(r["result"]["isError"], false);
    let info: Json = serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap_or(""))
        .unwrap_or_default();
    assert_eq!(info[0]["width"], 20);

    send(
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
        "name": "render", "arguments": {}}}),
    );
    let r = recv();
    assert_eq!(r["result"]["content"][0]["type"], "image");
    assert_eq!(r["result"]["content"][0]["mimeType"], "image/png");

    send(
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {
        "name": "run", "arguments": {"script": "layer.delete L7"}}}),
    );
    let r = recv();
    assert_eq!(r["result"]["isError"], true);

    drop(send);
    drop(stdin);
    let status = child.wait().unwrap_or_else(|e| unreachable!("{e}"));
    assert!(status.success());
}
