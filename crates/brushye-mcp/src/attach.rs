//! Attach mode (D-022): the running app listens on a local endpoint, and `brushye-mcp --attach`
//! forwards tool calls to it.
//!
//! - Endpoint: a Unix socket `<dir>/<pid>.sock` (mode 0600 in a 0700 directory) on Linux and
//!   macOS; the named pipe `\\.\pipe\brushye-<user>-<pid>` on Windows (local clients only).
//!   No TCP.
//! - Discovery: the app writes `<dir>/<pid>.endpoint`, holding the endpoint name, and removes
//!   it when it stops listening.
//! - Wire: one JSON object per line. Request `{"tool": "...", "args": {...}}`; reply
//!   `{"ok": [{"text": "..."} | {"png": "<base64>"}]}` or `{"err": "..."}`.
//!
//! No async runtime here: the app runs this on plain threads, and the bridge calls it from
//! tokio's blocking pool.

use std::io::{self, BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{ListenerNonblockingMode, ListenerOptions, Name, Stream};
use serde_json::{Map, Value as Json, json};

use crate::Out;

/// Overrides the endpoint directory (tests, unusual setups).
pub const ENV_DIR: &str = "BRUSHYE_RUNTIME_DIR";

/// The per-user directory holding endpoints and their discovery files.
pub fn runtime_dir() -> PathBuf {
    if let Some(d) = std::env::var_os(ENV_DIR) {
        return PathBuf::from(d);
    }
    #[cfg(target_os = "linux")]
    if let Some(d) = std::env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(d).join("brushye");
    }
    // macOS and Windows: the per-user temp dir. Elsewhere: the temp dir, per user name.
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into());
    std::env::temp_dir().join(format!("brushye-{user}"))
}

/// Create the directory, private to the user (0700 on Unix). `chmod` fails on a directory
/// owned by another user, so a directory someone else planted is refused.
fn ensure_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", dir.display())))?;
    }
    Ok(())
}

/// The endpoint name string for a process.
fn endpoint_string(dir: &Path, pid: u32) -> String {
    if cfg!(windows) {
        let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".into());
        let user: String = user
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("brushye-{user}-{pid}")
    } else {
        dir.join(format!("{pid}.sock"))
            .to_string_lossy()
            .into_owned()
    }
}

fn to_name(s: &str) -> io::Result<Name<'_>> {
    if cfg!(windows) {
        s.to_ns_name::<interprocess::local_socket::GenericNamespaced>()
    } else {
        // A file-path socket, so the directory's permissions apply (Linux "namespaced" names
        // would be abstract sockets, which have no permissions).
        s.to_fs_name::<interprocess::local_socket::GenericFilePath>()
    }
}

/// One tool call from a client, answered by the host with [`Request::reply`].
pub struct Request {
    pub tool: String,
    pub args: Map<String, Json>,
    reply: mpsc::Sender<Result<Vec<Out>, String>>,
}

impl Request {
    pub fn reply(self, r: Result<Vec<Out>, String>) {
        // The client may have gone; nothing to do then.
        let _ = self.reply.send(r);
    }
}

/// A listening endpoint. Dropping it stops listening and removes the discovery file; open
/// connections then get an error on their next request.
pub struct Endpoint {
    pub name: String,
    stop: Arc<AtomicBool>,
    connected: Arc<AtomicUsize>,
    marker: PathBuf,
    thread: Option<JoinHandle<()>>,
}

impl Endpoint {
    /// Start listening in [`runtime_dir`]. Requests arrive on the returned channel; `wake` is
    /// called after each one is queued (the app requests a repaint).
    pub fn start(
        wake: Box<dyn Fn() + Send + Sync>,
    ) -> io::Result<(Endpoint, mpsc::Receiver<Request>)> {
        Self::start_in(&runtime_dir(), std::process::id(), wake)
    }

    /// As [`Endpoint::start`], in a given directory and for a given id.
    pub fn start_in(
        dir: &Path,
        pid: u32,
        wake: Box<dyn Fn() + Send + Sync>,
    ) -> io::Result<(Endpoint, mpsc::Receiver<Request>)> {
        ensure_dir(dir)?;
        let name = endpoint_string(dir, pid);
        let opts = || -> io::Result<ListenerOptions<'_>> {
            Ok(ListenerOptions::new()
                .name(to_name(&name)?)
                .nonblocking(ListenerNonblockingMode::Accept)
                .try_overwrite(true))
        };
        // The 0700 directory is the protection. A 0600 socket is a second layer where the OS
        // supports a socket mode (Linux; not macOS, which reports Unsupported).
        #[cfg(unix)]
        let listener = {
            use interprocess::os::unix::local_socket::ListenerOptionsExt as _;
            match opts()?.mode(0o600).create_sync() {
                Err(e) if e.kind() == io::ErrorKind::Unsupported => opts()?.create_sync()?,
                r => r?,
            }
        };
        #[cfg(not(unix))]
        let listener = opts()?.create_sync()?;
        let marker = dir.join(format!("{pid}.endpoint"));
        std::fs::write(&marker, &name)?;

        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let connected = Arc::new(AtomicUsize::new(0));
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::from(wake);
        let thread = {
            let (stop, connected) = (Arc::clone(&stop), Arc::clone(&connected));
            std::thread::Builder::new()
                .name("brushye-mcp-listen".into())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        match listener.accept() {
                            Ok(conn) => {
                                let (tx, wake, connected, stop) = (
                                    tx.clone(),
                                    Arc::clone(&wake),
                                    Arc::clone(&connected),
                                    Arc::clone(&stop),
                                );
                                connected.fetch_add(1, Ordering::Relaxed);
                                wake();
                                let _ = std::thread::Builder::new()
                                    .name("brushye-mcp-conn".into())
                                    .spawn(move || {
                                        let _ = serve(conn, &tx, &*wake, &stop);
                                        connected.fetch_sub(1, Ordering::Relaxed);
                                        wake();
                                    });
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(50));
                            }
                            Err(_) => std::thread::sleep(Duration::from_millis(200)),
                        }
                    }
                    // Dropping the listener removes the socket file (Unix).
                })?
        };
        Ok((
            Endpoint {
                name,
                stop,
                connected,
                marker,
                thread: Some(thread),
            },
            rx,
        ))
    }

    /// Clients connected now.
    pub fn connected(&self) -> usize {
        self.connected.load(Ordering::Relaxed)
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        let _ = std::fs::remove_file(&self.marker);
    }
}

/// Serve one connection: read requests, hand them to the host, write the replies. Once the
/// endpoint is stopped (or the host dropped its receiver), the next request closes the
/// connection without a reply; the bridge then reconnects or reports that no app is listening.
fn serve(
    conn: Stream,
    tx: &mpsc::Sender<Request>,
    wake: &dyn Fn(),
    stop: &AtomicBool,
) -> io::Result<()> {
    let mut conn = BufReader::new(conn);
    let mut line = String::new();
    loop {
        line.clear();
        if conn.read_line(&mut line)? == 0 || stop.load(Ordering::Relaxed) {
            return Ok(());
        }
        let reply = match parse_request(&line) {
            Err(e) => Err(e),
            Ok((tool, args)) => {
                let (rtx, rrx) = mpsc::channel();
                let req = Request {
                    tool,
                    args,
                    reply: rtx,
                };
                if tx.send(req).is_err() {
                    return Ok(());
                }
                wake();
                match rrx.recv() {
                    Ok(r) => r,
                    // The host stopped without answering.
                    Err(_) => return Ok(()),
                }
            }
        };
        let mut out = reply_json(&reply).to_string();
        out.push('\n');
        conn.get_mut().write_all(out.as_bytes())?;
    }
}

fn parse_request(line: &str) -> Result<(String, Map<String, Json>), String> {
    let j: Json = serde_json::from_str(line).map_err(|e| format!("bad request: {e}"))?;
    let tool = j["tool"]
        .as_str()
        .ok_or("bad request: missing \"tool\"")?
        .to_owned();
    let args = match j.get("args") {
        None | Some(Json::Null) => Map::new(),
        Some(Json::Object(m)) => m.clone(),
        Some(_) => return Err("bad request: \"args\" must be an object".into()),
    };
    Ok((tool, args))
}

fn reply_json(r: &Result<Vec<Out>, String>) -> Json {
    match r {
        Ok(out) => json!({ "ok": out.iter().map(|o| match o {
            Out::Text(t) => json!({ "text": t }),
            Out::Png(p) => json!({ "png": B64.encode(p) }),
        }).collect::<Vec<_>>() }),
        Err(e) => json!({ "err": e }),
    }
}

fn parse_reply(line: &str) -> Result<Result<Vec<Out>, String>, String> {
    let j: Json = serde_json::from_str(line).map_err(|e| format!("bad reply from Brushye: {e}"))?;
    if let Some(e) = j.get("err") {
        return Ok(Err(e.as_str().unwrap_or("error").to_owned()));
    }
    let items = j["ok"].as_array().ok_or("bad reply from Brushye")?;
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        if let Some(t) = it["text"].as_str() {
            out.push(Out::Text(t.to_owned()));
        } else if let Some(p) = it["png"].as_str() {
            out.push(Out::Png(
                B64.decode(p).map_err(|e| format!("bad image: {e}"))?,
            ));
        } else {
            return Err("bad reply item from Brushye".into());
        }
    }
    Ok(Ok(out))
}

/// Running apps that advertise an endpoint: (pid, endpoint name), lowest pid first.
pub fn endpoints(dir: &Path) -> Vec<(u32, String)> {
    let mut v: Vec<(u32, String)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let pid = p
                .file_name()?
                .to_str()?
                .strip_suffix(".endpoint")?
                .parse()
                .ok()?;
            Some((pid, std::fs::read_to_string(&p).ok()?.trim().to_owned()))
        })
        .collect();
    v.sort();
    v
}

/// A connection to a running app.
pub struct Client {
    conn: BufReader<Stream>,
    pub pid: u32,
}

/// Message when no app is listening.
pub const NOT_LISTENING: &str = "No Brushye window is accepting AI clients. In Brushye, turn on \
File > Allow AI clients (MCP), then try again.";

impl Client {
    /// Connect to the app with `pid`, or to the only app listening when `pid` is `None`.
    pub fn connect(dir: &Path, pid: Option<u32>) -> Result<Client, String> {
        let mut live = Vec::new();
        for (p, name) in endpoints(dir) {
            if pid.is_some_and(|want| want != p) {
                continue;
            }
            // Stale discovery files (a crashed app) fail to connect and are skipped.
            if let Ok(s) = to_name(&name).and_then(Stream::connect) {
                live.push((p, s));
            }
        }
        match live.len() {
            0 => Err(match pid {
                Some(p) => {
                    format!("Brushye process {p} isn't accepting AI clients. {NOT_LISTENING}")
                }
                None => NOT_LISTENING.into(),
            }),
            1 => {
                let (pid, s) = live.remove(0);
                Ok(Client {
                    conn: BufReader::new(s),
                    pid,
                })
            }
            _ => Err(format!(
                "Several Brushye windows accept AI clients (processes {}); start brushye-mcp with --attach PID",
                live.iter()
                    .map(|(p, _)| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    /// Call a tool in the app. The outer `Err` means the connection failed.
    pub fn call(
        &mut self,
        tool: &str,
        args: &Map<String, Json>,
    ) -> Result<Result<Vec<Out>, String>, String> {
        let mut req = json!({ "tool": tool, "args": args }).to_string();
        req.push('\n');
        self.conn
            .get_mut()
            .write_all(req.as_bytes())
            .map_err(|e| e.to_string())?;
        let mut line = String::new();
        match self.conn.read_line(&mut line) {
            Ok(0) => Err("Brushye closed the connection".into()),
            Ok(_) => parse_reply(&line),
            Err(e) => Err(e.to_string()),
        }
    }
}
