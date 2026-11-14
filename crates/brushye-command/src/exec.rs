//! The executor: the only place session state changes (D-010/D-011), with snapshot history
//! (D-013) and the session journal.
//!
//! - History is a list of immutable document versions and a position; undo/redo move the
//!   position. An undoable command clones the current version (sharing unchanged layers and
//!   tiles), edits it, and pushes it, discarding any redo branch.
//! - Consecutive changes of the same kind merge into one step (CSP, spec H-001). Live, the
//!   executor decides; the journal records the decision as `@n+`, and replay applies exactly
//!   the recorded grouping, never re-deciding.
//! - Layer visibility is journaled but not undoable (CSP): it is applied to every version in
//!   history, so undo/redo never change it.
//! - A command that fails changes nothing and is not journaled.

use std::sync::Arc;

use brushye_core::brush::BrushSettings;
use brushye_core::document::DocError;
use brushye_core::{Document, Layer, LayerId, LayerKind, Rgba8};
use serde_json::{Value as Json, json};

use crate::commands::{self, Class, Command, Paper, Place, Query};
use crate::params::{Arg, value_text};
use crate::syntax::{Line, RawLine, parse_line, print_line};

/// Journal format version (docs/command-format.md).
pub const JOURNAL_FORMAT: u32 = 1;
/// Default undo limit (CSP's maximum "Undo levels", spec H-001).
pub const DEFAULT_UNDO_LIMIT: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecError(pub String);

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ExecError {}

impl From<DocError> for ExecError {
    fn from(e: DocError) -> Self {
        ExecError(e.to_string())
    }
}

impl From<String> for ExecError {
    fn from(e: String) -> Self {
        ExecError(e)
    }
}

/// An error at a line of a script or journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineError {
    pub line: usize,
    pub msg: String,
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for LineError {}

#[derive(Clone, Debug)]
struct Step {
    label: &'static str,
    doc: Arc<Document>,
}

/// What can still merge into the current undo step.
#[derive(Clone, Debug, PartialEq, Eq)]
struct MergeKey {
    cmd: &'static str,
    layer: LayerId,
    keys: Vec<&'static str>,
}

fn merge_key(cmd: &Command) -> Option<MergeKey> {
    match cmd {
        Command::LayerSet(p) => {
            let mut keys = Vec::new();
            if p.name.is_some() {
                keys.push("name");
            }
            if p.opacity.is_some() {
                keys.push("opacity");
            }
            if p.blend.is_some() {
                keys.push("blend");
            }
            if p.locked.is_some() {
                keys.push("locked");
            }
            if p.lock_alpha.is_some() {
                keys.push("lock_alpha");
            }
            Some(MergeKey {
                cmd: "layer.set",
                layer: p.layer,
                keys,
            })
        }
        _ => None,
    }
}

/// One editing session: current document, undo history, active layer and journal.
#[derive(Clone, Debug)]
pub struct Session {
    steps: Vec<Step>,
    pos: usize,
    active: Option<LayerId>,
    journal: Vec<RawLine>,
    undo_limit: usize,
    merge_open: Option<MergeKey>,
    /// Content hash of the document this session was opened from (`None`: started by
    /// `doc.new`). Recorded in the journal header so replay can check it gets the same base.
    base: Option<u64>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            pos: 0,
            active: None,
            journal: Vec::new(),
            undo_limit: DEFAULT_UNDO_LIMIT,
            merge_open: None,
            base: None,
        }
    }

    /// A session that starts from an existing document (e.g. one opened from a `.brsh` file).
    /// History starts at that document; the journal header records its content hash.
    pub fn from_document(doc: Document, active: Option<LayerId>) -> Self {
        let base = doc.content_hash();
        let mut s = Self::new();
        s.steps = vec![Step {
            label: "open",
            doc: Arc::new(doc),
        }];
        s.active = active;
        s.base = Some(base);
        s.fix_active();
        s
    }

    /// Number of journaled commands (cheap; for "unsaved changes" tracking).
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Content hash of the base document, if the session was opened from one.
    pub fn base_hash(&self) -> Option<u64> {
        self.base
    }

    /// Undo limit in steps, 1..=200 like CSP (values are clamped).
    pub fn set_undo_limit(&mut self, n: usize) {
        self.undo_limit = n.clamp(1, DEFAULT_UNDO_LIMIT);
        self.enforce_limit();
    }

    /// The current document version (immutable; share it freely).
    pub fn document(&self) -> Option<Arc<Document>> {
        self.steps.get(self.pos).map(|s| s.doc.clone())
    }

    pub fn active_layer(&self) -> Option<LayerId> {
        self.active
    }

    fn doc(&self) -> Result<&Arc<Document>, ExecError> {
        self.steps
            .get(self.pos)
            .map(|s| &s.doc)
            .ok_or_else(|| ExecError("no document (run doc.new first)".into()))
    }

    /// Execute a command live: the executor decides merging. On success it is journaled.
    pub fn execute(&mut self, cmd: Command) -> Result<(), ExecError> {
        let merge = cmd.class() == Class::Undoable
            && self.merge_open.is_some()
            && self.merge_open == merge_key(&cmd)
            && self.pos + 1 == self.steps.len();
        self.apply(&cmd, merge)?;
        let mut raw = cmd.to_raw();
        raw.seq = Some(self.journal.len() as u64 + 1);
        raw.merge = merge;
        self.journal.push(raw);
        Ok(())
    }

    fn apply(&mut self, cmd: &Command, merge: bool) -> Result<(), ExecError> {
        match cmd.class() {
            Class::Session => self.apply_session(cmd),
            Class::Undoable => self.apply_undoable(cmd, merge),
            Class::AppState => self.apply_app_state(cmd),
            Class::History => {
                match cmd {
                    Command::HistoryUndo(_) => self.pos = self.pos.saturating_sub(1),
                    Command::HistoryRedo(_) => {
                        if self.pos + 1 < self.steps.len() {
                            self.pos += 1;
                        }
                    }
                    _ => {
                        return Err(ExecError(format!(
                            "{} is not a history command",
                            cmd.name()
                        )));
                    }
                }
                self.merge_open = None;
                self.fix_active();
                Ok(())
            }
        }
    }

    fn apply_session(&mut self, cmd: &Command) -> Result<(), ExecError> {
        let Command::DocNew(p) = cmd else {
            return Err(ExecError(format!(
                "{} is not a session command",
                cmd.name()
            )));
        };
        let paper = match p.paper {
            None => Some(Rgba8::WHITE),
            Some(Paper::None) => None,
            Some(Paper::Color(c)) => Some(c),
        };
        let doc = Document::new(p.width, p.height, p.dpi.unwrap_or(72), paper)?;
        self.steps = vec![Step {
            label: "doc.new",
            doc: Arc::new(doc),
        }];
        self.pos = 0;
        self.active = None;
        self.merge_open = None;
        Ok(())
    }

    fn apply_undoable(&mut self, cmd: &Command, merge: bool) -> Result<(), ExecError> {
        let mut d = Document::clone(self.doc()?);
        edit(&mut d, cmd)?;
        let doc = Arc::new(d);
        if merge {
            if self.pos == 0 {
                return Err(ExecError("nothing to merge into".into()));
            }
            self.steps.truncate(self.pos + 1);
            if let Some(s) = self.steps.get_mut(self.pos) {
                s.doc = doc;
            }
        } else {
            self.steps.truncate(self.pos + 1);
            self.steps.push(Step {
                label: cmd.name(),
                doc,
            });
            self.pos += 1;
            self.enforce_limit();
        }
        self.merge_open = merge_key(cmd);
        if let Command::LayerNew(p) = cmd {
            self.active = Some(p.layer);
        }
        self.fix_active();
        Ok(())
    }

    fn apply_app_state(&mut self, cmd: &Command) -> Result<(), ExecError> {
        match cmd {
            Command::LayerShow(p) => {
                self.doc()?
                    .layer(p.layer)
                    .ok_or(DocError::NoSuchLayer(p.layer))?;
                // Not undoable: apply to every version that has this layer.
                for s in &mut self.steps {
                    if s.doc.layer(p.layer).is_some_and(|l| l.visible != p.visible) {
                        let mut d = Document::clone(&s.doc);
                        d.layer_mut(p.layer)?.visible = p.visible;
                        s.doc = Arc::new(d);
                    }
                }
                Ok(())
            }
            Command::LayerSelect(p) => {
                self.doc()?
                    .layer(p.layer)
                    .ok_or(DocError::NoSuchLayer(p.layer))?;
                self.active = Some(p.layer);
                Ok(())
            }
            _ => Err(ExecError(format!(
                "{} is not an app-state command",
                cmd.name()
            ))),
        }
    }

    fn enforce_limit(&mut self) {
        while self.steps.len() > self.undo_limit + 1 {
            self.steps.remove(0);
            self.pos = self.pos.saturating_sub(1);
        }
    }

    /// The active layer must exist in the current version (it may not after undo/redo).
    fn fix_active(&mut self) {
        if let (Some(a), Some(d)) = (self.active, self.steps.get(self.pos))
            && d.doc.layer(a).is_none()
        {
            self.active = None;
        }
    }

    /// Answer a query (never journaled, never changes state).
    pub fn query(&self, q: &Query) -> Result<Json, ExecError> {
        Ok(match q {
            Query::Commands(_) => commands::registry_json(),
            Query::History(_) => json!({
                "position": self.pos,
                "steps": self.steps.iter().map(|s| s.label).collect::<Vec<_>>(),
                "can_undo": self.pos > 0,
                "can_redo": self.pos + 1 < self.steps.len(),
                "limit": self.undo_limit,
            }),
            Query::DocInfo(_) => {
                let d = self.doc()?;
                json!({
                    "width": d.width, "height": d.height, "dpi": d.dpi,
                    "paper": d.paper.map(|p| p.to_json()),
                    "layers": d.layers().len(),
                    "active": self.active.map(|a| a.to_string()),
                })
            }
            Query::LayerList(_) => {
                Json::Array(self.doc()?.layers().iter().map(|l| layer_json(l)).collect())
            }
            Query::LayerGet(p) => layer_json(
                self.doc()?
                    .layer(p.layer)
                    .ok_or(DocError::NoSuchLayer(p.layer))?,
            ),
            Query::DocHash(_) => Json::String(format!("{:016x}", self.doc()?.content_hash())),
            Query::Pixel(p) => {
                let d = self.doc()?;
                in_canvas(d, p.x, p.y)?;
                // Renders the whole document; fine for queries, the app renders incrementally.
                let img = brushye_core::render::flatten(d);
                json!(img.pixel(p.x, p.y))
            }
            Query::LayerPixel(p) => {
                let d = self.doc()?;
                in_canvas(d, p.x, p.y)?;
                json!(
                    d.layer(p.layer)
                        .ok_or(DocError::NoSuchLayer(p.layer))?
                        .pixel(p.x, p.y)
                )
            }
        })
    }

    /// Run one text line live (script mode): a command, a query (returns `Some`), a comment or
    /// a blank line. Journal lines (`@n …`) are rejected here; use [`Session::replay`].
    pub fn run_line(&mut self, text: &str) -> Result<Option<Json>, ExecError> {
        match parse_line(text).map_err(|e| ExecError(e.to_string()))? {
            Line::Blank | Line::Comment(_) => Ok(None),
            Line::Item(raw) if raw.query => {
                let q = Query::from_raw(&raw)?;
                self.query(&q).map(Some)
            }
            Line::Item(raw) => {
                if raw.seq.is_some() {
                    return Err(ExecError(
                        "journal lines (@n) can only be replayed, not run as a script".into(),
                    ));
                }
                let raw = commands::migrate(raw)?;
                self.execute(Command::from_raw(&raw)?)?;
                Ok(None)
            }
        }
    }

    /// Run a whole script; stops at the first error. Query results are returned in order.
    pub fn run_script(&mut self, text: &str) -> Result<Vec<Json>, LineError> {
        let mut out = Vec::new();
        for (i, l) in text.lines().enumerate() {
            match self.run_line(l) {
                Ok(Some(j)) => out.push(j),
                Ok(None) => {}
                Err(e) => {
                    return Err(LineError {
                        line: i + 1,
                        msg: e.0,
                    });
                }
            }
        }
        Ok(out)
    }

    /// The journal so far, as text (header + one line per executed command).
    pub fn journal_text(&self) -> String {
        let mut s = format!(
            "#brushye-journal v{JOURNAL_FORMAT} app={}",
            env!("CARGO_PKG_VERSION")
        );
        if let Some(b) = self.base {
            s.push_str(&format!(" base={b:016x}"));
        }
        s.push('\n');
        for raw in &self.journal {
            s.push_str(&print_line(raw));
            s.push('\n');
        }
        s
    }

    /// Rebuild a session from a journal: the header is required, sequence numbers must run
    /// 1, 2, 3 …, old command versions are migrated, and recorded merges are applied as written.
    pub fn replay(text: &str) -> Result<Session, LineError> {
        Self::replay_from(Session::new(), text)
    }

    /// Replay a journal recorded on top of an opened document (`base=` in its header). The
    /// document must be the same one: its content hash is checked.
    pub fn replay_on(
        base: Document,
        active: Option<LayerId>,
        text: &str,
    ) -> Result<Session, LineError> {
        Self::replay_from(Session::from_document(base, active), text)
    }

    fn replay_from(mut s: Session, text: &str) -> Result<Session, LineError> {
        let mut lines = text.lines().enumerate();
        let err = |line: usize, msg: String| LineError { line, msg };
        match lines.next() {
            Some((_, h)) if h.starts_with("#brushye-journal ") => {
                let mut words = h.split_whitespace().skip(1);
                let v = words.next().unwrap_or("");
                if v != format!("v{JOURNAL_FORMAT}") {
                    return Err(err(
                        1,
                        format!(
                            "unsupported journal format '{v}' (this Brushye reads v{JOURNAL_FORMAT})"
                        ),
                    ));
                }
                let base = words
                    .find_map(|w| w.strip_prefix("base="))
                    .map(|b| {
                        u64::from_str_radix(b, 16)
                            .map_err(|_| err(1, format!("bad base hash '{b}'")))
                    })
                    .transpose()?;
                match (base, s.base) {
                    (None, None) => {}
                    (Some(_), None) => {
                        return Err(err(
                            1,
                            "this journal continues a saved document (base= in the header); replay it on that document".into(),
                        ));
                    }
                    (None, Some(_)) => {
                        return Err(err(
                            1,
                            "this journal starts with doc.new; replay it without a base document"
                                .into(),
                        ));
                    }
                    (Some(want), Some(have)) if want != have => {
                        return Err(err(
                            1,
                            format!(
                                "base document differs: journal expects {want:016x}, got {have:016x}"
                            ),
                        ));
                    }
                    _ => {}
                }
            }
            _ => return Err(err(1, "missing '#brushye-journal v1 …' header".into())),
        }
        for (i, l) in lines {
            let n = i + 1;
            let raw = match parse_line(l).map_err(|e| err(n, e.to_string()))? {
                Line::Blank | Line::Comment(_) => continue,
                Line::Item(r) => r,
            };
            if raw.query {
                return Err(err(n, "queries don't belong in a journal".into()));
            }
            let expected = s.journal.len() as u64 + 1;
            if raw.seq != Some(expected) {
                return Err(err(n, format!("expected sequence number @{expected}")));
            }
            let merge = raw.merge;
            let raw = commands::migrate(raw).map_err(|e| err(n, e))?;
            let cmd = Command::from_raw(&raw).map_err(|e| err(n, e))?;
            if merge && cmd.class() != Class::Undoable {
                return Err(err(n, format!("{} can't join an undo step", cmd.name())));
            }
            s.apply(&cmd, merge).map_err(|e| err(n, e.0))?;
            let mut rec = cmd.to_raw();
            rec.seq = Some(expected);
            rec.merge = merge;
            s.journal.push(rec);
        }
        Ok(s)
    }
}

fn in_canvas(d: &Document, x: u32, y: u32) -> Result<(), ExecError> {
    if x < d.width && y < d.height {
        Ok(())
    } else {
        Err(ExecError(format!(
            "({x}, {y}) is outside the {}×{} canvas",
            d.width, d.height
        )))
    }
}

/// Write pixels honouring the layer's locks (lock transparent pixels keeps alpha, spec L-006).
fn write_pixels(
    d: &mut Document,
    id: LayerId,
    x: i64,
    y: i64,
    w: u32,
    h: u32,
    rgba: &[u8],
) -> Result<(), ExecError> {
    let bounds = (d.width, d.height);
    let l = d.layer_mut(id)?;
    if l.locked {
        return Err(DocError::LayerLocked(id).into());
    }
    if l.lock_alpha {
        let mut kept = rgba.to_vec();
        for row in 0..h {
            for col in 0..w {
                let (cx, cy) = (x + i64::from(col), y + i64::from(row));
                let i = ((row * w + col) * 4) as usize;
                let old =
                    if cx >= 0 && cy >= 0 && cx < i64::from(bounds.0) && cy < i64::from(bounds.1) {
                        l.pixel(cx as u32, cy as u32)
                    } else {
                        [0; 4]
                    };
                // Keep alpha; fully transparent pixels stay exactly as they were.
                if old[3] == 0 {
                    kept[i..i + 4].copy_from_slice(&old);
                } else {
                    kept[i + 3] = old[3];
                }
            }
        }
        l.put_pixels(bounds, x, y, w, h, &kept);
    } else {
        l.put_pixels(bounds, x, y, w, h, rgba);
    }
    Ok(())
}

fn layer_json(l: &Layer) -> Json {
    json!({
        "id": l.id.to_string(),
        "name": l.name,
        "kind": l.kind.name(),
        "visible": l.visible,
        "opacity": l.opacity,
        "blend": l.blend.name(),
        "locked": l.locked,
        "lock_alpha": l.lock_alpha,
        "tiles": l.tiles.len(),
    })
}

/// Apply an undoable command to a document copy.
fn edit(d: &mut Document, cmd: &Command) -> Result<(), ExecError> {
    match cmd {
        Command::LayerNew(p) => {
            let kind = p.kind.unwrap_or(LayerKind::Raster);
            let name = p
                .name
                .clone()
                .unwrap_or_else(|| format!("Layer {}", p.layer.0));
            d.add_layer(Layer::new(p.layer, name, kind), p.above)?;
        }
        Command::LayerDelete(p) => {
            d.remove_layer(p.layer)?;
        }
        Command::LayerMove(p) => match (p.above, p.to) {
            (Some(a), None) => d.move_layer(p.layer, Some(a))?,
            (None, Some(Place::Bottom)) => d.move_layer(p.layer, None)?,
            (None, Some(Place::Top)) => {
                let top = d.layers().last().map(|l| l.id);
                d.move_layer(p.layer, top)?;
            }
            _ => {
                return Err(ExecError(
                    "layer.move needs exactly one of above=… or to=…".into(),
                ));
            }
        },
        Command::LayerSet(p) => {
            let only_unlock = p.locked == Some(false)
                && p.name.is_none()
                && p.opacity.is_none()
                && p.blend.is_none()
                && p.lock_alpha.is_none();
            if p.name.is_none()
                && p.opacity.is_none()
                && p.blend.is_none()
                && p.locked.is_none()
                && p.lock_alpha.is_none()
            {
                return Err(ExecError("layer.set needs at least one property".into()));
            }
            if let Some(o) = p.opacity
                && o > 100
            {
                return Err(DocError::OpacityOutOfRange(u32::from(o)).into());
            }
            let l = d.layer_mut(p.layer)?;
            if l.locked && !only_unlock {
                return Err(DocError::LayerLocked(p.layer).into());
            }
            if let Some(v) = &p.name {
                l.name.clone_from(v);
            }
            if let Some(v) = p.opacity {
                l.opacity = v;
            }
            if let Some(v) = p.blend {
                l.blend = v;
            }
            if let Some(v) = p.locked {
                l.locked = v;
            }
            if let Some(v) = p.lock_alpha {
                l.lock_alpha = v;
            }
        }
        Command::LayerFill(p) => {
            let (w, h) = (d.width, d.height);
            let block: Vec<u8> = p
                .color
                .0
                .iter()
                .copied()
                .cycle()
                .take(w as usize * h as usize * 4)
                .collect();
            write_pixels(d, p.layer, 0, 0, w, h, &block)?;
        }
        Command::BrushStroke(p) => {
            let settings = brush_settings(p)?;
            let bounds = (d.width, d.height);
            let l = d.layer_mut(p.layer)?;
            if l.locked {
                return Err(DocError::LayerLocked(p.layer).into());
            }
            brushye_core::brush::stroke(l, bounds, &settings, &p.samples.0)
                .map_err(|e| ExecError(e.to_string()))?;
        }
        Command::LayerPixels(p) => {
            let need = u64::from(p.w) * u64::from(p.h) * 4;
            if p.data.0.len() as u64 != need {
                return Err(ExecError(format!(
                    "layer.pixels: data has {} bytes, w*h*4 = {need}",
                    p.data.0.len()
                )));
            }
            write_pixels(d, p.layer, p.x, p.y, p.w, p.h, &p.data.0)?;
        }
        _ => {
            return Err(ExecError(format!(
                "{} is not an undoable command",
                cmd.name()
            )));
        }
    }
    Ok(())
}

/// Brush settings of a `brush.stroke` command (validated by the engine).
fn brush_settings(p: &commands::BrushStroke) -> Result<BrushSettings, ExecError> {
    if p.color.0[3] != 255 {
        return Err(ExecError(
            "brush.stroke: color must be opaque (#rrggbb); use opacity=… for transparency".into(),
        ));
    }
    crate::brush_preset::BrushPreset::from_stroke(p)
        .settings([p.color.0[0], p.color.0[1], p.color.0[2]])
        .map_err(|e| ExecError(format!("brush.stroke: {e}")))
}

/// Text of a single parameter value (for error messages and tools).
pub fn arg_text<T: Arg>(v: &T) -> String {
    value_text(&v.to_value())
}
