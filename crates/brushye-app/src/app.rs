//! The app state and frame loop. Every change to the document is a command executed by the
//! session (D-011); the UI only reads document versions. Strokes are drawn with a live preview
//! (`LiveStroke`) and committed as one `brush.stroke` command when the pen lifts.

use std::path::PathBuf;
use std::sync::Arc;

use brushye_command::Session;
use brushye_command::brush_preset::BrushPreset;
use brushye_command::commands::{self, Command};
use brushye_command::core::brush::{InkMode, LiveStroke};
use brushye_command::core::input::{Sample, canonical};
use brushye_command::core::{Document, Layer, LayerId, Rgba8};
use brushye_command::params::Samples;
use brushye_format::brsh::{self, SaveOptions, TileCache};
use brushye_tablet::{Delivery, PenEvent, PenPose, Tablet, Tool as PenTool};
use eframe::egui;

use crate::canvas_gpu::{CanvasCallback, CanvasGpu, Pending, ViewUniform};
use crate::display::Display;
use crate::ui::tools::ToolId;
use crate::ui::{UiState, colour::Chip};
use crate::view::View;

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
    SaveDuplicate {
        path: String,
    },
    ExportPng {
        path: String,
    },
    /// Global pen pressure curve (CSP: File > Pen Pressure Settings).
    PenPressure {
        points: Vec<[f64; 2]>,
    },
    /// Discard unsaved changes, then run the action.
    Confirm(Box<Action>),
}

/// Actions that may need a "discard changes?" confirmation.
#[derive(Clone, Debug)]
pub enum Action {
    New,
    Open,
    OpenPath(String),
    Close,
    Revert,
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
    pub(crate) settings: crate::settings::Settings,
    pub(crate) ui: UiState,
}

/// Apply the user's application settings to a session as commands (so they're journaled).
fn apply_settings(s: &mut Session, settings: &crate::settings::Settings) {
    if settings.pressure_curve.is_empty() {
        return;
    }
    if let Ok(points) = brushye_command::core::brush::Curve::new(settings.pressure_curve.clone()) {
        let _ = s.execute(Command::InputPressureCurve(commands::InputPressureCurve {
            points,
        }));
    }
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

fn set_cmd(id: LayerId) -> commands::LayerSet {
    commands::LayerSet {
        layer: id,
        name: None,
        opacity: None,
        blend: None,
        locked: None,
        lock_alpha: None,
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        let mut adapter = String::new();
        let gpu_ready = if let Some(rs) = cc.wgpu_render_state.as_ref() {
            let info = rs.adapter.get_info();
            adapter = format!("{} ({:?})", info.name, info.backend);
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
        let settings = crate::settings::Settings::load();
        let mut session = new_session(1920, 1080, 72, true).unwrap_or_else(|_| Session::new());
        apply_settings(&mut session, &settings);
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
            ui: UiState::new(&settings),
            settings,
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

    // ---------------------------------------------------------------- state queries

    pub(crate) fn dirty(&self) -> bool {
        self.session.journal_len() != self.saved_len
    }

    pub(crate) fn doc_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// CSP's caption shape: "<name>* (<w> x <h>px <dpi>dpi <zoom>%) - <app>".
    pub(crate) fn title(&self) -> String {
        let d = self.doc();
        format!(
            "{}{} ({} x {}px {}dpi {:.1}%) - Brushye",
            self.doc_name(),
            if self.dirty() { "*" } else { "" },
            d.as_ref().map_or(0, |d| d.width),
            d.as_ref().map_or(0, |d| d.height),
            d.as_ref().map_or(0, |d| d.dpi),
            self.view.zoom * 100.0
        )
    }

    pub(crate) fn doc(&self) -> Option<Arc<Document>> {
        self.session.document()
    }

    pub(crate) fn history(&self) -> (Vec<String>, usize) {
        let j = self
            .session
            .query(&commands::Query::History(commands::QHistory {}))
            .unwrap_or_default();
        let steps = j["steps"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        (steps, j["position"].as_u64().unwrap_or(0) as usize)
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.live.is_none() && self.history().1 > 0
    }

    pub(crate) fn can_redo(&self) -> bool {
        let (steps, pos) = self.history();
        self.live.is_none() && pos + 1 < steps.len()
    }

    fn active_layer(&self) -> Option<Layer> {
        let d = self.doc()?;
        let id = self.session.active_layer()?;
        d.layer(id).map(|l| Layer::clone(l))
    }

    pub(crate) fn can_edit_layer(&self) -> bool {
        self.active_layer().is_some_and(|l| !l.locked)
    }

    pub(crate) fn active_flags(&self) -> (bool, bool, bool) {
        self.active_layer().map_or((false, false, true), |l| {
            (l.locked, l.lock_alpha, l.visible)
        })
    }

    pub(crate) fn is_drawing(&self) -> bool {
        self.live.is_some()
    }

    pub(crate) fn display_generation(&self) -> u64 {
        self.display.generation
    }

    pub(crate) fn display_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        self.display.pixel(x, y)
    }

    pub(crate) fn cursor_canvas(&self) -> Option<[f64; 2]> {
        self.cursor
    }

    pub(crate) fn tablet_status(&self) -> String {
        match &self.tablet {
            Ok(t) => format!("tablet: {} ({:?})", t.backend(), t.delivery()),
            Err(e) => format!("tablet: {e}"),
        }
    }

    // ---------------------------------------------------------------- commands

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

    /// History palette: go to step `i` by undoing/redoing.
    pub(crate) fn goto_history(&mut self, i: usize) {
        let (_, pos) = self.history();
        for _ in i..pos {
            self.undo();
        }
        for _ in pos..i {
            self.redo();
        }
    }

    pub(crate) fn new_raster_layer(&mut self) {
        let id = self.doc().map_or(1, |d| {
            d.layers().iter().map(|l| l.id.0).max().unwrap_or(0) + 1
        });
        self.exec(Command::LayerNew(commands::LayerNew {
            layer: LayerId(id),
            kind: None,
            name: Some(format!("Layer {id}")),
            above: self.session.active_layer(),
        }));
    }

    pub(crate) fn delete_layer(&mut self) {
        if let Some(id) = self.session.active_layer() {
            self.exec(Command::LayerDelete(commands::LayerDelete { layer: id }));
        }
    }

    /// Edit > Delete with no selection: clear the layer.
    pub(crate) fn clear_layer(&mut self) {
        if let Some(id) = self.session.active_layer() {
            self.exec(Command::LayerFill(commands::LayerFill {
                layer: id,
                color: Rgba8([0, 0, 0, 0]),
            }));
        }
    }

    /// Edit > Fill with no selection: fill the layer with the drawing colour.
    pub(crate) fn fill_layer(&mut self) {
        let Some(id) = self.session.active_layer() else {
            return;
        };
        let color = match self.ui.colour.drawing() {
            Some(c) => Rgba8([c[0], c[1], c[2], 255]),
            None => Rgba8([0, 0, 0, 0]),
        };
        self.exec(Command::LayerFill(commands::LayerFill { layer: id, color }));
    }

    pub(crate) fn toggle_lock(&mut self, alpha: bool) {
        if let Some(l) = self.active_layer() {
            let mut s = set_cmd(l.id);
            if alpha {
                s.lock_alpha = Some(!l.lock_alpha);
            } else {
                s.locked = Some(!l.locked);
            }
            self.exec(Command::LayerSet(s));
        }
    }

    pub(crate) fn toggle_visible(&mut self) {
        if let Some(l) = self.active_layer() {
            self.exec(Command::LayerShow(commands::LayerShow {
                layer: l.id,
                visible: !l.visible,
            }));
        }
    }

    pub(crate) fn start_rename(&mut self) {
        if let Some(l) = self.active_layer() {
            self.ui.renaming = Some((l.id.0, l.name.clone()));
        }
    }

    /// Layer order: 0 top, 1 up, 2 down, 3 bottom.
    pub(crate) fn move_active(&mut self, which: u8) {
        let (Some(d), Some(id)) = (self.doc(), self.session.active_layer()) else {
            return;
        };
        let ids: Vec<LayerId> = d.layers().iter().map(|l| l.id).collect();
        let Some(i) = ids.iter().position(|&x| x == id) else {
            return;
        };
        let cmd = match which {
            0 => commands::LayerMove {
                layer: id,
                above: None,
                to: Some(commands::Place::Top),
            },
            1 if i + 1 < ids.len() => commands::LayerMove {
                layer: id,
                above: Some(ids[i + 1]),
                to: None,
            },
            2 if i >= 2 => commands::LayerMove {
                layer: id,
                above: Some(ids[i - 2]),
                to: None,
            },
            2 | 3 if i > 0 => commands::LayerMove {
                layer: id,
                above: None,
                to: Some(commands::Place::Bottom),
            },
            _ => return,
        };
        self.exec(Command::LayerMove(cmd));
    }

    pub(crate) fn select_neighbour(&mut self, above: bool) {
        let (Some(d), Some(id)) = (self.doc(), self.session.active_layer()) else {
            return;
        };
        let ids: Vec<LayerId> = d.layers().iter().map(|l| l.id).collect();
        let Some(i) = ids.iter().position(|&x| x == id) else {
            return;
        };
        let j = if above { i + 1 } else { i.wrapping_sub(1) };
        if let Some(&n) = ids.get(j) {
            self.exec(Command::LayerSelect(commands::LayerSelect { layer: n }));
        }
    }

    pub(crate) fn scale_brush(&mut self, k: f64) {
        let tool = self.ui.tool;
        if let Some(s) = self.ui.sets.get_mut(&tool).and_then(|s| s.current_mut()) {
            s.preset.size = (s.preset.size * k).clamp(0.5, 2000.0);
            self.ui.sets_dirty = true;
        }
    }

    /// Set the global pen pressure curve: a command for this session, and the saved setting
    /// for later ones.
    pub(crate) fn set_pressure_curve(&mut self, points: Vec<[f64; 2]>) {
        let curve = match brushye_command::core::brush::Curve::new(points.clone()) {
            Ok(c) => c,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        if self.exec(Command::InputPressureCurve(commands::InputPressureCurve {
            points: curve,
        })) {
            self.settings.pressure_curve = points;
            self.save_settings();
        }
    }

    pub(crate) fn set_interface(&mut self, i: crate::ui::theme::Interface) {
        self.ui.interface = i;
        self.settings.interface = i.name().into();
        self.save_settings();
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.settings.save() {
            self.status = format!("Couldn't save settings: {e}");
        }
    }

    /// Save edited sub tools when the pointer is up (they're per-user settings, like CSP's).
    pub(crate) fn save_subtools_if_idle(&mut self, ctx: &egui::Context) {
        if !self.ui.sets_dirty || self.live.is_some() || ctx.input(|i| i.pointer.any_down()) {
            return;
        }
        self.ui.sets_dirty = false;
        for (tool, set) in &self.ui.sets {
            self.settings
                .subtools
                .insert(tool.key().into(), crate::ui::tools::to_json(set));
        }
        self.save_settings();
    }

    pub(crate) fn import_sub_tool(&mut self, path: &str) {
        let result = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| brushye_format::brush_file::load(&b, false).map_err(|e| e.to_string()))
            .and_then(|(b, _)| BrushPreset::from_json(&b.settings).map(|p| (b.name, p)));
        match result {
            Ok((name, preset)) => {
                let tool = self.ui.tool;
                if let Some(set) = self.ui.sets.get_mut(&tool) {
                    let g = set.group;
                    let group = &mut set.groups[g];
                    group.items.push(crate::ui::tools::SubTool {
                        name: name.clone(),
                        preset,
                    });
                    group.selected = group.items.len() - 1;
                    self.ui.sets_dirty = true;
                    self.status = format!("Imported “{name}”");
                } else {
                    self.status = "Choose a brush tool first".into();
                }
            }
            Err(e) => self.status = e,
        }
    }

    // ---------------------------------------------------------------- files

    fn replace_session(&mut self, mut s: Session, path: Option<PathBuf>) {
        apply_settings(&mut s, &self.settings);
        self.live = None;
        self.saved_len = s.journal_len();
        self.session = s;
        self.path = path;
        self.cache = TileCache::default();
        self.fit_pending = true;
        self.reset_gpu = true;
        self.display = Display::default();
        self.ui.thumbs.clear();
        self.ui.nav_texture = None;
    }

    pub(crate) fn new_document(&mut self, w: u32, h: u32, dpi: u32, paper: bool) {
        match new_session(w, h, dpi, paper) {
            Ok(s) => {
                self.replace_session(s, None);
                self.status = format!("New {w}×{h} canvas");
            }
            Err(e) => self.status = e,
        }
    }

    fn remember_recent(&mut self, path: &str) {
        let p = std::path::Path::new(path)
            .canonicalize()
            .map_or_else(|_| path.to_owned(), |p| p.to_string_lossy().into_owned());
        self.settings.recent.retain(|r| *r != p);
        self.settings.recent.insert(0, p);
        self.settings.recent.truncate(10);
        self.save_settings();
    }

    pub(crate) fn open_path(&mut self, path: &str) {
        match brsh::load_file(std::path::Path::new(path)) {
            Ok(l) => {
                let warnings = l.warnings.len();
                let s = Session::from_document(l.doc, l.active);
                self.replace_session(s, Some(PathBuf::from(path)));
                self.remember_recent(path);
                self.status = if warnings > 0 {
                    format!("Opened {path} ({warnings} warnings: unknown content ignored)")
                } else {
                    format!("Opened {path}")
                };
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    fn write_brsh(&mut self, path: &std::path::Path) -> Result<(), String> {
        let doc = self.doc().ok_or("no canvas")?;
        let journal = self.session.journal_text();
        let opts = SaveOptions {
            active: self.session.active_layer(),
            journal: self.embed_journal.then_some(journal.as_str()),
        };
        brsh::save_file(path, &doc, &opts, &mut self.cache).map_err(|e| e.to_string())?;
        // The session journal next to the file (D-014 4).
        brsh::write_atomic(&path.with_extension("bcmd"), journal.as_bytes())
            .map_err(|e| e.to_string())
    }

    /// Save to `path` (crash-safe).
    pub(crate) fn save_to(&mut self, path: PathBuf) {
        match self.write_brsh(&path) {
            Ok(()) => {
                self.status = format!(
                    "Saved {} ({} tiles compressed)",
                    path.display(),
                    self.cache.last_compressed
                );
                self.saved_len = self.session.journal_len();
                self.remember_recent(&path.to_string_lossy());
                self.path = Some(path);
            }
            Err(e) => self.status = e,
        }
    }

    pub(crate) fn save_duplicate(&mut self, path: PathBuf) {
        self.status = match self.write_brsh(&path) {
            Ok(()) => format!("Saved a duplicate as {}", path.display()),
            Err(e) => e,
        };
    }

    pub(crate) fn save(&mut self) {
        match self.path.clone() {
            Some(p) => self.save_to(p),
            None => self.dialog = Some(self.save_as_dialog()),
        }
    }

    pub(crate) fn suggest_path(&self, ext: &str) -> String {
        let base = self
            .path
            .clone()
            .unwrap_or_else(|| PathBuf::from("untitled.brsh"));
        match ext {
            "copy.brsh" => base
                .with_file_name(format!("{} copy.brsh", self.doc_name()))
                .to_string_lossy()
                .into_owned(),
            e => base.with_extension(e).to_string_lossy().into_owned(),
        }
    }

    pub(crate) fn save_as_dialog(&self) -> Dialog {
        Dialog::SaveAs {
            path: self.suggest_path("brsh"),
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
        let discards = !matches!(a, Action::Open);
        if self.dirty() && discards {
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
                let path = self
                    .path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.dialog = Some(Dialog::Open { path });
            }
            Action::OpenPath(p) => self.open_path(&p),
            Action::Close => self.new_document(1920, 1080, 72, true),
            Action::Revert => {
                if let Some(p) = self.path.clone() {
                    self.open_path(&p.to_string_lossy());
                }
            }
            Action::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    // ---------------------------------------------------------------- view

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
    pub(crate) fn canvas_px(&self) -> ([f64; 2], [f64; 2]) {
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

    fn screen_px(&self, p: egui::Pos2) -> [f64; 2] {
        let k = f64::from(self.ppp);
        [f64::from(p.x) * k, f64::from(p.y) * k]
    }

    // ---------------------------------------------------------------- strokes and tools

    /// The pen's tool: the eraser end uses the Eraser tool's current sub tool (CSP).
    fn stroke_tool(&self, eraser_end: bool) -> ToolId {
        if eraser_end {
            ToolId::Eraser
        } else {
            self.ui.tool
        }
    }

    fn begin_stroke(&mut self, source: Source, eraser_end: bool, t0_us: u64, t0_s: f64) -> bool {
        let tool = self.stroke_tool(eraser_end);
        if !tool.is_brush() {
            return false;
        }
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
        let Some(mut preset) = self.ui.current_preset(tool).cloned() else {
            return false;
        };
        // The transparent colour paints as an eraser (CSP).
        let color = match (tool, self.ui.colour.drawing()) {
            (ToolId::Eraser, _) => [0, 0, 0],
            (_, Some(c)) => c,
            (_, None) => {
                preset.ink = Some(InkMode::Erase);
                [0, 0, 0]
            }
        };
        let settings = match preset.settings_with(color, self.session.pressure_curve()) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("Brush: {e}");
                return false;
            }
        };
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
        if self.exec(Command::BrushStroke(cmd)) && live.preset.ink != Some(InkMode::Erase) {
            self.ui.colour.used();
        }
    }

    /// A press on the canvas with a non-brush tool (Zoom, Hand, Eyedropper).
    fn tool_press(&mut self, pos: egui::Pos2, alt: bool) -> bool {
        match self.ui.tool {
            ToolId::Zoom => {
                let at = self.screen_px(pos);
                self.view.step_zoom(if alt { -1 } else { 1 }, at);
                true
            }
            ToolId::Eyedropper => {
                let c = self.to_canvas(pos);
                if c[0] >= 0.0 && c[1] >= 0.0 {
                    let (x, y) = (c[0] as u32, c[1] as u32);
                    let px = if self.ui.eyedropper_layer {
                        self.active_layer()
                            .filter(|_| self.doc().is_some_and(|d| x < d.width && y < d.height))
                            .map(|l| l.pixel(x, y))
                    } else {
                        self.display.pixel(x, y)
                    };
                    match px {
                        Some(p) if p[3] > 0 => self.ui.colour.set([p[0], p[1], p[2]]),
                        Some(_) => self.ui.colour.select(Chip::Transparent),
                        None => {}
                    }
                }
                true
            }
            ToolId::Hand => true,
            _ => false,
        }
    }

    // ---------------------------------------------------------------- input

    /// Pen events from the tablet backend (queued by `raw_input_hook`).
    fn handle_pen(&mut self, alt: bool) {
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
                    if self.live.is_some()
                        || self.dialog.is_some()
                        || self.pan_drag
                        || self.rotate_drag
                        || !self.canvas_rect.contains(pos)
                    {
                        continue;
                    }
                    if !self.pen.eraser && self.tool_press(pos, alt) {
                        continue;
                    }
                    if self.begin_stroke(Source::Pen, self.pen.eraser, p.time_us, 0.0) {
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

    /// Mouse input (only while no pen is in proximity, so a pen delivered as the mouse isn't
    /// handled twice).
    fn handle_mouse(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let (events, time, alt) = ui.input(|i| (i.events.clone(), i.time, i.modifiers.alt));
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
                    if !self.tool_press(pos, alt)
                        && self.begin_stroke(Source::Mouse, false, 0, time)
                    {
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

    /// Pan (space or middle drag, or the Hand tool), rotate (shift+space drag), zoom (wheel).
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
        let hand = self.ui.tool == ToolId::Hand && primary && response.is_pointer_button_down_on();
        let navigating = (space && !typing && primary) || middle || hand;
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
        let c = self.ui.colors();
        ui.painter().rect_filled(rect, 0.0, c.surround);
        self.canvas_rect = rect;
        self.ppp = ui.ctx().pixels_per_point();
        if self.fit_pending && rect.width() > 1.0 {
            self.fit_pending = false;
            self.fit();
        }
        let alt = ui.input(|i| i.modifiers.alt);
        self.navigate(ui, &response);
        self.handle_pen(alt);
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
        self.tool_cursor(ui, &response);
        if self.live.is_some() || self.pan_drag || self.rotate_drag {
            ui.ctx().request_repaint();
        }
    }

    /// Brush outline at the pointer for brush tools; a tool cursor otherwise.
    fn tool_cursor(&self, ui: &egui::Ui, response: &egui::Response) {
        match self.ui.tool {
            ToolId::Hand => {
                response.clone().on_hover_cursor(if self.pan_drag {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::Grab
                });
                return;
            }
            ToolId::Zoom => {
                response.clone().on_hover_cursor(egui::CursorIcon::ZoomIn);
                return;
            }
            ToolId::Eyedropper => {
                response
                    .clone()
                    .on_hover_cursor(egui::CursorIcon::Crosshair);
                return;
            }
            _ => {}
        }
        let Some(c) = self.cursor else { return };
        if self
            .live
            .as_ref()
            .is_some_and(|l| l.source == Source::Mouse)
        {
            return;
        }
        let Some(preset) = self
            .ui
            .current_preset(self.stroke_tool(self.pen.eraser && self.pen.near))
        else {
            return;
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
}

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
        crate::ui::frame(self, ui);
    }
}
