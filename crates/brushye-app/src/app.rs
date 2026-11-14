//! The app state and frame loop. Every change to the document is a command executed by the
//! session (D-011); the UI only reads document versions. Strokes are drawn with a live
//! preview (`LiveStroke`) and committed as one `brush.stroke` command when the pen lifts.

use std::path::PathBuf;
use std::sync::Arc;

use brushye_command::Session;
use brushye_command::brush_preset::BrushPreset;
use brushye_command::commands::{self, Command};
use brushye_command::core::brush::LiveStroke;
use brushye_command::core::input::{Sample, canonical};
use brushye_command::core::{Document, LayerId};
use brushye_command::params::Samples;
use brushye_format::brsh::{self, SaveOptions, TileCache};
use brushye_tablet::{Delivery, PenEvent, PenPose, Tablet, Tool as PenTool};
use eframe::egui;

use crate::canvas_gpu::{CanvasCallback, CanvasGpu, Pending, ViewUniform};
use crate::display::Display;
use crate::view::View;

/// Drawing tools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Pen,
    Eraser,
}

/// Where the samples of the current stroke come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Pen,
    Mouse,
}

struct Live {
    stroke: LiveStroke,
    layer: LayerId,
    preset: BrushPreset,
    color: [u8; 3],
    source: Source,
    /// Time origin: pen µs or egui seconds.
    t0_us: u64,
    t0_s: f64,
    last: Option<[f64; 2]>,
}

/// Pen state from the tablet backend.
#[derive(Default)]
struct Pen {
    near: bool,
    eraser: bool,
    last: Option<PenPose>,
}

/// Modal dialogs.
pub enum Dialog {
    New {
        width: u32,
        height: u32,
        dpi: u32,
        paper: bool,
    },
    Open {
        path: String,
    },
    SaveAs {
        path: String,
        embed_journal: bool,
    },
    ExportPng {
        path: String,
    },
    /// Discard unsaved changes, then run the action.
    Confirm(Box<Action>),
}

/// Actions that may need a "discard changes?" confirmation.
#[derive(Clone, Debug)]
pub enum Action {
    New,
    Open,
    Quit,
}

pub struct App {
    pub(crate) session: Session,
    pub(crate) path: Option<PathBuf>,
    pub(crate) embed_journal: bool,
    saved_len: usize,
    cache: TileCache,
    pub(crate) view: View,
    fit_pending: bool,
    display: Display,
    reset_gpu: bool,
    gpu_ready: bool,
    pending: Arc<std::sync::Mutex<Pending>>,
    tablet: Result<Tablet, String>,
    pen_events: Vec<PenEvent>,
    pen: Pen,
    live: Option<Live>,
    pub(crate) tool: ToolKind,
    pub(crate) pen_preset: BrushPreset,
    pub(crate) eraser_preset: BrushPreset,
    pub(crate) color: egui::Color32,
    pub(crate) dialog: Option<Dialog>,
    pub(crate) status: String,
    canvas_rect: egui::Rect,
    ppp: f32,
    cursor: Option<[f64; 2]>,
    pressure: Option<f32>,
    pan_drag: bool,
    rotate_drag: bool,
    pub(crate) allow_close: bool,
    pub(crate) shown_title: String,
    pub(crate) brush_dir: String,
    pub(crate) brush_name: String,
}

fn default_pen() -> BrushPreset {
    BrushPreset {
        size: 12.0,
        aa: None,
        hardness: None,
        opacity: None,
        density: None,
        ink: None,
        gap: None,
        density_by_gap: None,
        thickness: None,
        direction: None,
        angle: None,
        angle_line: None,
        stabilization: None,
        size_pressure: Some(20),
        size_curve: None,
        opacity_pressure: None,
        opacity_curve: None,
        density_pressure: None,
        density_curve: None,
    }
}

fn default_eraser() -> BrushPreset {
    let mut p = default_pen();
    p.size = 40.0;
    p.ink = Some(brushye_command::core::brush::InkMode::Erase);
    p.size_pressure = None;
    p
}

/// Default brush library folder (per-user configuration directory).
fn default_brush_dir() -> String {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    };
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("brushye")
        .join("brushes")
        .to_string_lossy()
        .into_owned()
}

/// A fresh session with a document like CSP's default: white paper and one raster layer.
pub fn new_session(width: u32, height: u32, dpi: u32, paper: bool) -> Result<Session, String> {
    let mut s = Session::new();
    let paper = if paper { "" } else { " paper=none" };
    s.run_script(&format!(
        "doc.new {width} {height} dpi={dpi}{paper}\nlayer.new L1 name=\"Layer 1\"\nlayer.select L1"
    ))
    .map_err(|e| e.to_string())?;
    Ok(s)
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        let mut adapter = String::new();
        let gpu_ready = if let Some(rs) = cc.wgpu_render_state.as_ref() {
            let info = rs.adapter.get_info();
            adapter = format!("{} ({:?}, {:?})", info.name, info.backend, rs.target_format);
            let gpu = CanvasGpu::new(&rs.device, rs.target_format);
            rs.renderer.write().callback_resources.insert(gpu);
            true
        } else {
            false
        };
        let tablet = Tablet::new(cc, cc).map_err(|e| e.to_string());
        if let Ok(t) = &tablet {
            let ctx = cc.egui_ctx.clone();
            t.set_waker(Arc::new(move || ctx.request_repaint()));
        }
        let session = new_session(1920, 1080, 72, true).unwrap_or_else(|_| Session::new());
        let mut app = Self {
            saved_len: session.journal_len(),
            session,
            path: None,
            embed_journal: false,
            cache: TileCache::default(),
            view: View::default(),
            fit_pending: true,
            display: Display::default(),
            reset_gpu: true,
            gpu_ready,
            pending: Arc::default(),
            tablet,
            pen_events: Vec::new(),
            pen: Pen::default(),
            live: None,
            tool: ToolKind::Pen,
            pen_preset: default_pen(),
            eraser_preset: default_eraser(),
            color: egui::Color32::from_rgb(32, 32, 40),
            dialog: None,
            status: String::new(),
            canvas_rect: egui::Rect::NOTHING,
            ppp: 1.0,
            cursor: None,
            pressure: None,
            pan_drag: false,
            rotate_drag: false,
            allow_close: false,
            shown_title: String::new(),
            brush_dir: default_brush_dir(),
            brush_name: String::new(),
        };
        app.status = if gpu_ready {
            format!("GPU: {adapter}")
        } else {
            "No wgpu renderer: the canvas can't be shown".into()
        };
        if let Some(p) = open {
            app.open_path(&p.to_string_lossy());
        }
        app
    }

    pub(crate) fn tablet_status(&self) -> String {
        match &self.tablet {
            Ok(t) => format!("tablet: {} ({:?})", t.backend(), t.delivery()),
            Err(e) => format!("tablet: {e}"),
        }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.session.journal_len() != self.saved_len
    }

    pub(crate) fn title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());
        format!("{name}{} — Brushye", if self.dirty() { " *" } else { "" })
    }

    /// Execute a command; failures go to the status line.
    pub(crate) fn exec(&mut self, cmd: Command) -> bool {
        match self.session.execute(cmd) {
            Ok(()) => true,
            Err(e) => {
                self.status = e.to_string();
                false
            }
        }
    }

    pub(crate) fn doc(&self) -> Option<Arc<Document>> {
        self.session.document()
    }

    pub(crate) fn active_preset(&mut self) -> &mut BrushPreset {
        match self.tool {
            ToolKind::Pen => &mut self.pen_preset,
            ToolKind::Eraser => &mut self.eraser_preset,
        }
    }

    fn replace_session(&mut self, s: Session, path: Option<PathBuf>) {
        self.live = None;
        self.saved_len = s.journal_len();
        self.session = s;
        self.path = path;
        self.cache = TileCache::default();
        self.fit_pending = true;
        self.reset_gpu = true;
        self.display = Display::default();
    }

    pub(crate) fn new_document(&mut self, w: u32, h: u32, dpi: u32, paper: bool) {
        match new_session(w, h, dpi, paper) {
            Ok(s) => {
                self.replace_session(s, None);
                self.status = format!("New {w}×{h} document");
            }
            Err(e) => self.status = e,
        }
    }

    pub(crate) fn open_path(&mut self, path: &str) {
        match brsh::load_file(std::path::Path::new(path)) {
            Ok(l) => {
                let warnings = l.warnings.len();
                let s = Session::from_document(l.doc, l.active);
                self.replace_session(s, Some(PathBuf::from(path)));
                self.status = if warnings > 0 {
                    format!("Opened {path} ({warnings} warnings: unknown content ignored)")
                } else {
                    format!("Opened {path}")
                };
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    /// Save to `path` (crash-safe), with the session journal next to it (D-014 4).
    pub(crate) fn save_to(&mut self, path: PathBuf) {
        let Some(doc) = self.doc() else { return };
        let journal = self.session.journal_text();
        let opts = SaveOptions {
            active: self.session.active_layer(),
            journal: self.embed_journal.then_some(journal.as_str()),
        };
        let result = brsh::save_file(&path, &doc, &opts, &mut self.cache).and_then(|()| {
            brsh::write_atomic(&path.with_extension("bcmd"), journal.as_bytes())
                .map_err(|e| brsh::BrshError(e.to_string()))
        });
        match result {
            Ok(()) => {
                self.status = format!(
                    "Saved {} ({} tiles compressed)",
                    path.display(),
                    self.cache.last_compressed
                );
                self.saved_len = self.session.journal_len();
                self.path = Some(path);
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    pub(crate) fn save(&mut self) {
        match self.path.clone() {
            Some(p) => self.save_to(p),
            None => self.dialog = Some(self.save_as_dialog()),
        }
    }

    pub(crate) fn save_as_dialog(&self) -> Dialog {
        Dialog::SaveAs {
            path: self
                .path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "untitled.brsh".into()),
            embed_journal: self.embed_journal,
        }
    }

    pub(crate) fn export_png(&mut self, path: &str) {
        let Some(doc) = self.doc() else { return };
        let img = brushye_command::core::render::flatten(&doc);
        let result = brushye_format::png_io::png_bytes(&img)
            .map_err(|e| e.to_string())
            .and_then(|b| {
                brsh::write_atomic(std::path::Path::new(path), &b).map_err(|e| e.to_string())
            });
        self.status = match result {
            Ok(()) => format!("Exported {path}"),
            Err(e) => e,
        };
    }

    /// Run an action, asking first if it would discard unsaved changes.
    pub(crate) fn request(&mut self, a: Action, ctx: &egui::Context) {
        if self.dirty() {
            self.dialog = Some(Dialog::Confirm(Box::new(a)));
        } else {
            self.run_action(a, ctx);
        }
    }

    pub(crate) fn run_action(&mut self, a: Action, ctx: &egui::Context) {
        match a {
            Action::New => {
                let d = self.doc();
                self.dialog = Some(Dialog::New {
                    width: d.as_ref().map_or(1920, |d| d.width),
                    height: d.as_ref().map_or(1080, |d| d.height),
                    dpi: d.as_ref().map_or(72, |d| d.dpi),
                    paper: true,
                });
            }
            Action::Open => {
                self.dialog = Some(Dialog::Open {
                    path: self
                        .path
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                });
            }
            Action::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    pub(crate) fn undo(&mut self) {
        if self.live.is_none() {
            self.exec(Command::HistoryUndo(commands::HistoryUndo {}));
        }
    }

    pub(crate) fn redo(&mut self) {
        if self.live.is_none() {
            self.exec(Command::HistoryRedo(commands::HistoryRedo {}));
        }
    }

    pub(crate) fn fit(&mut self) {
        if let Some(d) = self.doc() {
            let r = self.canvas_px();
            self.view.fit(d.width, d.height, r.0, r.1);
        }
    }

    pub(crate) fn actual_size(&mut self) {
        if let Some(d) = self.doc() {
            let r = self.canvas_px();
            self.view.actual_size(d.width, d.height, r.0, r.1);
        }
    }

    /// Canvas area in physical px: (min, size).
    fn canvas_px(&self) -> ([f64; 2], [f64; 2]) {
        let p = f64::from(self.ppp);
        let r = self.canvas_rect;
        (
            [f64::from(r.min.x) * p, f64::from(r.min.y) * p],
            [f64::from(r.width()) * p, f64::from(r.height()) * p],
        )
    }

    pub(crate) fn canvas_center(&self) -> [f64; 2] {
        let (m, s) = self.canvas_px();
        [m[0] + s[0] / 2.0, m[1] + s[1] / 2.0]
    }

    /// Window point (logical) → canvas coordinates.
    fn to_canvas(&self, p: egui::Pos2) -> [f64; 2] {
        let k = f64::from(self.ppp);
        self.view
            .to_canvas([f64::from(p.x) * k, f64::from(p.y) * k])
    }

    // ---------------------------------------------------------------- strokes

    fn begin_stroke(
        &mut self,
        at: egui::Pos2,
        source: Source,
        eraser_end: bool,
        t0_us: u64,
        t0_s: f64,
    ) -> bool {
        let Some(doc) = self.doc() else { return false };
        let Some(id) = self.session.active_layer() else {
            self.status = "Select a layer to draw on".into();
            return false;
        };
        let Some(layer) = doc.layer(id) else {
            return false;
        };
        if layer.locked {
            self.status = format!("{} is locked", layer.name);
            return false;
        }
        let (preset, color) = if eraser_end || self.tool == ToolKind::Eraser {
            (self.eraser_preset.clone(), [0, 0, 0])
        } else {
            let c = self.color;
            (self.pen_preset.clone(), [c.r(), c.g(), c.b()])
        };
        let settings = match preset.settings(color) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("Brush: {e}");
                return false;
            }
        };
        let _ = at;
        match LiveStroke::new(Layer::clone(layer), (doc.width, doc.height), settings) {
            Ok(stroke) => {
                self.live = Some(Live {
                    stroke,
                    layer: id,
                    preset,
                    color,
                    source,
                    t0_us,
                    t0_s,
                    last: None,
                });
                true
            }
            Err(e) => {
                self.status = e.to_string();
                false
            }
        }
    }

    fn push_sample(
        &mut self,
        pos: egui::Pos2,
        pressure: Option<f32>,
        tilt: Option<[f32; 2]>,
        twist: Option<f32>,
        t_us: u64,
    ) {
        let c = self.to_canvas(pos);
        let Some(live) = self.live.as_mut() else {
            return;
        };
        if live.source == Source::Mouse && live.last == Some(c) {
            return;
        }
        live.last = Some(c);
        let t = t_us.saturating_sub(live.t0_us).min(u64::from(u32::MAX)) as u32;
        let raw = Sample {
            x: c[0],
            y: c[1],
            t_us: t,
            pressure,
            tilt,
            twist,
            eraser: false,
            barrel: [false; 2],
        };
        // Quantize now so the preview uses exactly the values the command will carry.
        if let Ok(q) = canonical(&[raw]) {
            live.stroke.push(&q);
        }
    }

    fn finish_stroke(&mut self) {
        let Some(live) = self.live.take() else { return };
        if live.stroke.samples().is_empty() {
            return;
        }
        let samples = match canonical(live.stroke.samples()) {
            Ok(s) => s,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        let cmd = live
            .preset
            .to_stroke(live.layer, live.color, Samples(samples));
        self.exec(Command::BrushStroke(cmd));
    }

    // ---------------------------------------------------------------- input

    /// Pen events from the tablet backend (queued by `raw_input_hook`).
    fn handle_pen(&mut self) {
        let evs = std::mem::take(&mut self.pen_events);
        for ev in evs {
            match ev {
                PenEvent::In { tool } => {
                    self.pen.near = true;
                    self.pen.eraser = tool == PenTool::Eraser;
                }
                PenEvent::Out => {
                    self.pen.near = false;
                    if self.live.as_ref().is_some_and(|l| l.source == Source::Pen) {
                        self.finish_stroke();
                    }
                }
                PenEvent::Pose(p) => {
                    self.pen.last = Some(p);
                    self.pressure = p.pressure;
                    let pos = egui::pos2(p.x as f32, p.y as f32);
                    self.cursor = self.canvas_rect.contains(pos).then(|| self.to_canvas(pos));
                    if self.live.as_ref().is_some_and(|l| l.source == Source::Pen) {
                        self.push_sample(pos, p.pressure, p.tilt, p.twist, p.time_us);
                    }
                }
                PenEvent::Down => {
                    let Some(p) = self.pen.last else { continue };
                    let pos = egui::pos2(p.x as f32, p.y as f32);
                    if self.live.is_none()
                        && self.dialog.is_none()
                        && !self.pan_drag
                        && !self.rotate_drag
                        && self.canvas_rect.contains(pos)
                        && self.begin_stroke(pos, Source::Pen, self.pen.eraser, p.time_us, 0.0)
                    {
                        self.push_sample(pos, p.pressure, p.tilt, p.twist, p.time_us);
                    }
                }
                PenEvent::Up => {
                    if self.live.as_ref().is_some_and(|l| l.source == Source::Pen) {
                        self.finish_stroke();
                    }
                }
                PenEvent::Button { .. } => {}
            }
        }
    }

    /// Mouse strokes (only while no pen is in proximity, so a pen delivered as the mouse
    /// isn't drawn twice).
    fn handle_mouse(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let (events, time) = ui.input(|i| (i.events.clone(), i.time));
        for ev in events {
            match ev {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    ..
                } if !self.pen.near
                    && self.live.is_none()
                    && !self.pan_drag
                    && !self.rotate_drag
                    && response.hovered()
                    && self.canvas_rect.contains(pos) =>
                {
                    if self.begin_stroke(pos, Source::Mouse, false, 0, time) {
                        self.push_sample(pos, None, None, None, 0);
                    }
                }
                egui::Event::PointerMoved(pos) => {
                    if !self.pen.near {
                        self.cursor = self.canvas_rect.contains(pos).then(|| self.to_canvas(pos));
                    }
                    if let Some(l) = &self.live
                        && l.source == Source::Mouse
                    {
                        let t = ((time - l.t0_s) * 1e6).max(0.0) as u64;
                        self.push_sample(pos, None, None, None, t);
                    }
                }
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                } if self
                    .live
                    .as_ref()
                    .is_some_and(|l| l.source == Source::Mouse) =>
                {
                    self.finish_stroke();
                }
                _ => {}
            }
        }
    }

    /// Pan (space or middle drag), rotate (shift+space drag), zoom (wheel).
    fn navigate(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let k = f64::from(self.ppp);
        let (space, shift, middle, primary, delta, scroll, hover) = ui.input(|i| {
            (
                i.key_down(egui::Key::Space),
                i.modifiers.shift,
                i.pointer.middle_down(),
                i.pointer.primary_down(),
                i.pointer.delta(),
                i.events
                    .iter()
                    .map(|e| match e {
                        egui::Event::MouseWheel { delta, .. } => delta.y,
                        _ => 0.0,
                    })
                    .sum::<f32>(),
                i.pointer.hover_pos(),
            )
        });
        let typing = ui.ctx().egui_wants_keyboard_input();
        let navigating = (space && !typing && primary) || middle;
        self.rotate_drag = navigating && shift && space;
        self.pan_drag = navigating && !self.rotate_drag;
        if self.live.is_some() {
            return;
        }
        if self.rotate_drag {
            self.view
                .rotate_by(f64::from(delta.x) * 0.5, self.canvas_center());
        } else if self.pan_drag {
            self.view
                .pan_by([f64::from(delta.x) * k, f64::from(delta.y) * k]);
        }
        if response.hovered() && scroll != 0.0 {
            let anchor = hover.map_or(self.canvas_center(), |p| {
                [f64::from(p.x) * k, f64::from(p.y) * k]
            });
            self.view
                .step_zoom(if scroll > 0.0 { 1 } else { -1 }, anchor);
        }
    }

    // ---------------------------------------------------------------- canvas

    pub(crate) fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        self.canvas_rect = rect;
        self.ppp = ui.ctx().pixels_per_point();
        if self.fit_pending && rect.width() > 1.0 {
            self.fit_pending = false;
            self.fit();
        }
        self.navigate(ui, &response);
        self.handle_pen();
        self.handle_mouse(ui, &response);

        let Some(doc) = self.doc() else { return };
        // While drawing, show the live preview in place of the layer.
        let shown: Arc<Document> = match &self.live {
            Some(l) => {
                let mut d = Document::clone(&doc);
                if let Ok(layer) = d.layer_mut(l.layer) {
                    *layer = l.stroke.preview().clone();
                }
                Arc::new(d)
            }
            None => doc.clone(),
        };
        let (uploads, reset) = self.display.update(&shown);
        // Both flags must be consumed (no short-circuit), or a stale reset would wipe the
        // tiles on a later frame.
        let forced = std::mem::take(&mut self.reset_gpu);
        let reset = reset || forced;
        if let Ok(mut p) = self.pending.lock() {
            p.push(uploads, reset);
        }
        let (m, t) = self.view.matrix();
        let uniform = ViewUniform {
            m: m.map(|v| v as f32),
            origin: [t[0] as f32, t[1] as f32],
            target: [1.0, 1.0],
            nearest: self.view.zoom >= 1.0,
        };
        if self.gpu_ready {
            ui.painter()
                .add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    CanvasCallback {
                        pending: self.pending.clone(),
                        view: uniform,
                    },
                ));
        }
        self.brush_cursor(ui, &doc);
        if self.live.is_some() || self.pan_drag || self.rotate_drag {
            ui.ctx().request_repaint();
        }
    }

    /// Brush outline at the pointer.
    fn brush_cursor(&self, ui: &egui::Ui, _doc: &Document) {
        let Some(c) = self.cursor else { return };
        if self.live.is_some()
            && self
                .live
                .as_ref()
                .is_some_and(|l| l.source == Source::Mouse)
        {
            return;
        }
        let preset = match self.tool {
            ToolKind::Pen if !self.pen.eraser => &self.pen_preset,
            _ => &self.eraser_preset,
        };
        let k = f64::from(self.ppp);
        let s = self.view.to_screen(c);
        let center = egui::pos2((s[0] / k) as f32, (s[1] / k) as f32);
        let r = (preset.size / 2.0 * self.view.zoom / k) as f32;
        let painter = ui.painter_at(self.canvas_rect);
        painter.circle_stroke(
            center,
            r.max(1.0),
            egui::Stroke::new(1.0, egui::Color32::from_gray(20)),
        );
        painter.circle_stroke(
            center,
            r.max(1.0) + 1.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(235)),
        );
    }

    pub(crate) fn status_line(&self) -> String {
        let pos = self
            .cursor
            .map(|c| format!("  ·  {:.0}, {:.0}", c[0], c[1]))
            .unwrap_or_default();
        let pressure = self
            .pressure
            .filter(|_| self.pen.near)
            .map(|p| format!("  ·  pressure {p:.2}"))
            .unwrap_or_default();
        format!(
            "{:.0} %  ·  {:.0}°{pos}{pressure}  ·  {}",
            self.view.zoom * 100.0,
            self.view.angle,
            self.tablet_status()
        )
    }
}

use brushye_command::core::Layer;

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let Ok(tablet) = &self.tablet else { return };
        tablet.set_scale_factor(f64::from(ctx.pixels_per_point()));
        let mut evs = Vec::new();
        tablet.poll(&mut evs);
        if tablet.delivery() == Delivery::Full {
            // The pen doesn't reach the app as a mouse: feed it into the UI too (D-015 2).
            let mut last = self.pen.last;
            for ev in &evs {
                match *ev {
                    PenEvent::Pose(p) => {
                        last = Some(p);
                        raw.events.push(egui::Event::PointerMoved(egui::pos2(
                            p.x as f32, p.y as f32,
                        )));
                    }
                    PenEvent::Down | PenEvent::Up => {
                        if let Some(p) = last {
                            raw.events.push(egui::Event::PointerButton {
                                pos: egui::pos2(p.x as f32, p.y as f32),
                                button: egui::PointerButton::Primary,
                                pressed: *ev == PenEvent::Down,
                                modifiers: egui::Modifiers::default(),
                            });
                        }
                    }
                    PenEvent::Out => raw.events.push(egui::Event::PointerGone),
                    _ => {}
                }
            }
        }
        self.pen_events.extend(evs);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::panels::frame(self, ui);
    }
}
