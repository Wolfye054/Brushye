//! Attach mode, app side (D-022): with File > Allow AI clients (MCP) on, `brushye-mcp --attach`
//! sends tool calls here. They run on the UI thread between frames, on the app's own session,
//! so undo, History, the Journal window and autosave see them like any user action.
//!
//! The setting is never saved: every app run starts with it off.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use brushye_command::Session;
use brushye_mcp::Host;
use brushye_mcp::attach::{Endpoint, Request};
use eframe::egui;

use crate::app::{App, Dialog};

pub struct McpLink {
    endpoint: Endpoint,
    rx: Receiver<Request>,
    /// Requests received but not run yet (waiting for a stroke or a dialog to finish).
    queue: VecDeque<Request>,
}

impl McpLink {
    pub fn connected(&self) -> usize {
        self.endpoint.connected()
    }
}

/// The app as an MCP tool host.
struct AppHost<'a>(&'a mut App);

impl Host for AppHost<'_> {
    fn session(&self) -> &Session {
        &self.0.session
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.0.session
    }
    fn replace(&mut self, s: Session, path: Option<&Path>) {
        self.0.replace_session(s, path.map(Path::to_path_buf));
        if let Some(p) = path {
            self.0.remember_recent(&p.to_string_lossy());
        }
    }
    fn saved(&mut self, path: &Path) {
        // Saving to the open document's own file counts as a save.
        let same = self.0.path.as_deref().is_some_and(|p| same_file(p, path));
        if same {
            self.0.mark_saved();
        }
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| PathBuf::from(p));
    canon(a) == canon(b)
}

impl App {
    pub(crate) fn mcp_enabled(&self) -> bool {
        self.mcp.is_some()
    }

    /// Turn AI access on or off. Turning it off closes connected clients on their next call.
    pub(crate) fn set_mcp(&mut self, on: bool, ctx: &egui::Context) {
        if !on {
            self.mcp = None;
            if matches!(self.dialog, Some(Dialog::McpReplace(_))) {
                self.dialog = None;
            }
            return;
        }
        if self.mcp.is_some() {
            return;
        }
        let waker = ctx.clone();
        match Endpoint::start(Box::new(move || waker.request_repaint())) {
            Ok((endpoint, rx)) => {
                self.mcp = Some(McpLink {
                    endpoint,
                    rx,
                    queue: VecDeque::new(),
                });
            }
            Err(e) => {
                self.dialog = Some(Dialog::Notice(format!(
                    "Couldn't start listening for AI clients: {e}"
                )));
            }
        }
    }

    /// Run queued tool calls. Called once per frame, before the UI.
    pub(crate) fn poll_mcp(&mut self, ctx: &egui::Context) {
        let Some(link) = self.mcp.as_mut() else {
            return;
        };
        link.queue.extend(link.rx.try_iter());
        loop {
            // Wait while the user is drawing or a dialog is open, then continue.
            let busy = self.drawing() || self.dialog.is_some() || self.file_dialog.is_some();
            let Some(link) = self.mcp.as_mut() else {
                return;
            };
            if link.queue.is_empty() {
                return;
            }
            if busy {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return;
            }
            let Some(req) = link.queue.pop_front() else {
                return;
            };
            if brushye_mcp::replaces_document(&req.tool) && self.dirty() {
                self.dialog = Some(Dialog::McpReplace(req));
                return;
            }
            self.run_mcp(req);
        }
    }

    /// Run one tool call now and answer it.
    pub(crate) fn run_mcp(&mut self, req: Request) {
        let r = AppHost(self).call(&req.tool, &req.args);
        req.reply(r);
    }
}
