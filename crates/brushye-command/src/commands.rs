//! The command and query registry (M2 set). Each entry declares its name, version, class and
//! typed parameters; parsing, printing, JSON and the schema all derive from it (D-011).

use brushye_core::{BlendMode, LayerId, LayerKind, Rgba8};
use serde_json::{Value as Json, json};

use crate::params;
use crate::params::{Arg, JsonReader, Mode, ParamSpec, Reader};
use crate::syntax::{RawLine, Value};

/// How a command interacts with history (D-013, spec H-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Starts a new document and a fresh history.
    Session,
    /// Changes the document; one undo step (or merged into the previous one).
    Undoable,
    /// Application/view state: journaled, never undoable.
    AppState,
    /// `history.undo` / `history.redo`.
    History,
}

/// Paper colour or no paper: `#rrggbb[aa]` or the word `none`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paper {
    Color(Rgba8),
    None,
}

impl Arg for Paper {
    const KIND: &'static str = "colour or none";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Word(w) if w == "none" => Ok(Paper::None),
            other => Rgba8::from_value(other)
                .map(Paper::Color)
                .map_err(|_| format!("expected a colour or none, found {}", other.kind())),
        }
    }
    fn to_value(&self) -> Value {
        match self {
            Paper::Color(c) => c.to_value(),
            Paper::None => Value::Word("none".into()),
        }
    }
    fn to_json(&self) -> Json {
        match self {
            Paper::Color(c) => c.to_json(),
            Paper::None => Json::String("none".into()),
        }
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        match j.as_str() {
            Some("none") => Ok(Paper::None),
            _ => Rgba8::from_json(j).map(Paper::Color),
        }
    }
}

/// `top` or `bottom` of the layer stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Top,
    Bottom,
}

impl Place {
    pub fn name(self) -> &'static str {
        match self {
            Place::Top => "top",
            Place::Bottom => "bottom",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "top" => Some(Place::Top),
            "bottom" => Some(Place::Bottom),
            _ => None,
        }
    }
}

impl Arg for Place {
    const KIND: &'static str = "place (top|bottom)";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Word(w) => {
                Place::from_name(w).ok_or_else(|| format!("unknown place '{w}' (top|bottom)"))
            }
            other => Err(format!("expected top or bottom, found {}", other.kind())),
        }
    }
    fn to_value(&self) -> Value {
        Value::Word(self.name().into())
    }
    fn to_json(&self) -> Json {
        Json::String(self.name().into())
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        j.as_str()
            .and_then(Place::from_name)
            .ok_or_else(|| "expected \"top\" or \"bottom\"".into())
    }
}

params! {
    /// New document; starts a fresh history.
    pub struct DocNew {
        width: u32 = pos "Canvas width in pixels",
        height: u32 = pos "Canvas height in pixels",
        dpi: Option<u32> = opt "Resolution (default 72)",
        paper: Option<Paper> = opt "Paper colour, or none (default #ffffff)",
    }
}
params! {
    /// New raster layer with a caller-chosen id.
    pub struct LayerNew {
        layer: LayerId = pos "Id for the new layer (must be unused)",
        kind: Option<LayerKind> = opt "Layer kind (default raster)",
        name: Option<String> = opt "Name (default \"Layer <n>\")",
        above: Option<LayerId> = opt "Insert directly above this layer (default: on top)",
    }
}
params! {
    /// Delete a layer.
    pub struct LayerDelete {
        layer: LayerId = pos "Layer to delete",
    }
}
params! {
    /// Move a layer: directly above another layer, or to the top/bottom.
    pub struct LayerMove {
        layer: LayerId = pos "Layer to move",
        above: Option<LayerId> = opt "Place directly above this layer",
        to: Option<Place> = opt "top or bottom",
    }
}
params! {
    /// Set layer properties (undoable). Visibility is `layer.show`.
    pub struct LayerSet {
        layer: LayerId = pos "Target layer",
        name: Option<String> = opt "Name",
        opacity: Option<u8> = opt "Opacity in percent, 0..=100",
        blend: Option<BlendMode> = opt "Blending mode",
        locked: Option<bool> = opt "Lock layer",
        lock_alpha: Option<bool> = opt "Lock transparent pixels",
    }
}
params! {
    /// Show or hide a layer (journaled, not undoable, as in CSP — spec H-001).
    pub struct LayerShow {
        layer: LayerId = pos "Target layer",
        visible: bool = key "Visible",
    }
}
params! {
    /// Make a layer the active (selected) layer (application state).
    pub struct LayerSelect {
        layer: LayerId = pos "Layer to select",
    }
}
params! {
    /// Undo one step.
    pub struct HistoryUndo {}
}
params! {
    /// Redo one step.
    pub struct HistoryRedo {}
}

params! {
    /// Document properties.
    pub struct QDocInfo {}
}
params! {
    /// All layers, bottom to top.
    pub struct QLayerList {}
}
params! {
    /// One layer's properties.
    pub struct QLayerGet {
        layer: LayerId = pos "Layer",
    }
}
params! {
    /// Deterministic content hash of the current document (hex).
    pub struct QDocHash {}
}
params! {
    /// Undo history: step labels and the current position.
    pub struct QHistory {}
}
params! {
    /// The command and query registry with parameter schemas.
    pub struct QCommands {}
}

/// Declares the command enum, the query enum and their registry tables.
macro_rules! registry {
    (
        commands { $( $C:ident ( $P:ident ) = $name:literal v $ver:literal $class:ident $doc:literal ),* $(,)? }
        queries { $( $Q:ident ( $QP:ident ) = $qname:literal $qdoc:literal ),* $(,)? }
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum Command { $( $C($P) ),* }

        #[derive(Clone, Debug, PartialEq)]
        pub enum Query { $( $Q($QP) ),* }

        /// Registry entry (schema) of a command or query.
        #[derive(Clone, Copy, Debug)]
        pub struct Spec {
            pub name: &'static str,
            pub version: u32,
            /// `None` for queries.
            pub class: Option<Class>,
            pub doc: &'static str,
            pub params: &'static [ParamSpec],
        }

        pub const COMMANDS: &[Spec] = &[
            $( Spec { name: $name, version: $ver, class: Some(Class::$class), doc: $doc, params: $P::PARAMS } ),*
        ];
        pub const QUERIES: &[Spec] = &[
            $( Spec { name: $qname, version: 1, class: None, doc: $qdoc, params: $QP::PARAMS } ),*
        ];

        impl Command {
            pub fn name(&self) -> &'static str {
                match self { $( Command::$C(_) => $name ),* }
            }
            pub fn version(&self) -> u32 {
                match self { $( Command::$C(_) => $ver ),* }
            }
            pub fn class(&self) -> Class {
                match self { $( Command::$C(_) => Class::$class ),* }
            }
            /// Typed command from a parsed line at the current version (see `migrate`).
            pub fn from_raw(raw: &RawLine) -> Result<Self, String> {
                let mut r = Reader::new(&raw.positional, &raw.named);
                let cmd = match raw.name.as_str() {
                    $( $name => Command::$C($P::read(&mut r)?), )*
                    other => return Err(format!("unknown command '{other}'")),
                };
                r.finish()?;
                Ok(cmd)
            }
            /// Parsed-line form (no sequence number).
            pub fn to_raw(&self) -> RawLine {
                let mut raw = RawLine { name: self.name().into(), version: self.version(), ..Default::default() };
                match self { $( Command::$C(p) => p.write(&mut raw.positional, &mut raw.named) ),* }
                raw
            }
            pub fn to_json(&self) -> Json {
                let args = match self { $( Command::$C(p) => p.write_json() ),* };
                json!({ "cmd": self.name(), "v": self.version(), "args": args })
            }
            pub fn from_json(j: &Json) -> Result<Self, String> {
                let name = j.get("cmd").and_then(Json::as_str).ok_or("missing \"cmd\"")?;
                let v = j.get("v").and_then(Json::as_u64).unwrap_or(1);
                let empty = serde_json::Map::new();
                let args = match j.get("args") { Some(Json::Object(m)) => m, None => &empty, Some(_) => return Err("\"args\" must be an object".into()) };
                let mut r = JsonReader::new(args);
                let cmd = match name {
                    $( $name => {
                        if v != $ver { return Err(format!("{name}: JSON form is version {v}, this Brushye has version {}", $ver)); }
                        Command::$C($P::read_json(&mut r)?)
                    } )*
                    other => return Err(format!("unknown command '{other}'")),
                };
                r.finish()?;
                Ok(cmd)
            }
        }

        impl Query {
            pub fn name(&self) -> &'static str {
                match self { $( Query::$Q(_) => $qname ),* }
            }
            pub fn from_raw(raw: &RawLine) -> Result<Self, String> {
                if raw.version != 1 {
                    return Err(format!("query {} has no version {}", raw.name, raw.version));
                }
                let mut r = Reader::new(&raw.positional, &raw.named);
                let q = match raw.name.as_str() {
                    $( $qname => Query::$Q($QP::read(&mut r)?), )*
                    other => return Err(format!("unknown query '{other}'")),
                };
                r.finish()?;
                Ok(q)
            }
            pub fn to_raw(&self) -> RawLine {
                let mut raw = RawLine { name: self.name().into(), version: 1, query: true, ..Default::default() };
                match self { $( Query::$Q(p) => p.write(&mut raw.positional, &mut raw.named) ),* }
                raw
            }
        }
    };
}

registry! {
    commands {
        DocNew(DocNew) = "doc.new" v 1 Session "New document; starts a fresh history",
        LayerNew(LayerNew) = "layer.new" v 1 Undoable "New layer with a caller-chosen id",
        LayerDelete(LayerDelete) = "layer.delete" v 1 Undoable "Delete a layer",
        LayerMove(LayerMove) = "layer.move" v 1 Undoable "Move a layer above another layer, or to the top or bottom",
        LayerSet(LayerSet) = "layer.set" v 1 Undoable "Set layer properties (name, opacity, blend, locks)",
        LayerShow(LayerShow) = "layer.show" v 1 AppState "Show or hide a layer (not undoable, as in CSP)",
        LayerSelect(LayerSelect) = "layer.select" v 1 AppState "Make a layer the active layer",
        HistoryUndo(HistoryUndo) = "history.undo" v 1 History "Undo one step",
        HistoryRedo(HistoryRedo) = "history.redo" v 1 History "Redo one step",
    }
    queries {
        DocInfo(QDocInfo) = "doc.info" "Document size, dpi, paper and active layer",
        LayerList(QLayerList) = "layer.list" "All layers, bottom to top",
        LayerGet(QLayerGet) = "layer.get" "One layer's properties",
        DocHash(QDocHash) = "doc.hash" "Deterministic content hash of the document (hex)",
        History(QHistory) = "history" "Undo history: step labels and current position",
        Commands(QCommands) = "commands" "Command and query registry with parameter schemas",
    }
}

/// Registry as JSON (for `?commands` and the MCP server).
pub fn registry_json() -> Json {
    let spec = |s: &Spec| {
        json!({
            "name": s.name,
            "version": s.version,
            "class": s.class.map(|c| format!("{c:?}").to_lowercase()),
            "params": s.params.iter().map(|p| json!({
                "name": p.name, "kind": p.kind, "doc": p.doc,
                "mode": match p.mode { Mode::Pos => "positional", Mode::Key => "required", Mode::Opt => "optional" },
            })).collect::<Vec<_>>(),
        })
    };
    json!({
        "commands": COMMANDS.iter().map(spec).collect::<Vec<_>>(),
        "queries": QUERIES.iter().map(spec).collect::<Vec<_>>(),
    })
}

/// A migration step: rewrites a line from `from` to `from + 1` for one command.
pub type Migration = (&'static str, u32, fn(RawLine) -> Result<RawLine, String>);

/// All migrations (none yet: every command is at version 1).
pub const MIGRATIONS: &[Migration] = &[];

/// Current version of a registered command.
pub fn current_version(name: &str) -> Option<u32> {
    COMMANDS.iter().find(|s| s.name == name).map(|s| s.version)
}

/// Bring a parsed line to its command's current version (D-011).
pub fn migrate(raw: RawLine) -> Result<RawLine, String> {
    migrate_with(MIGRATIONS, current_version, raw)
}

/// Migration engine, parameterised for tests.
pub fn migrate_with(
    table: &[Migration],
    current: fn(&str) -> Option<u32>,
    mut raw: RawLine,
) -> Result<RawLine, String> {
    let now = current(&raw.name).ok_or_else(|| format!("unknown command '{}'", raw.name))?;
    if raw.version > now {
        return Err(format!(
            "{} version {} is newer than this Brushye supports (version {now})",
            raw.name, raw.version
        ));
    }
    while raw.version < now {
        let (name, from) = (raw.name.clone(), raw.version);
        let step = table
            .iter()
            .find(|m| m.0 == name && m.1 == from)
            .ok_or_else(|| format!("no migration for {name} from version {from}"))?;
        raw = (step.2)(raw)?;
        raw.version = from + 1;
    }
    Ok(raw)
}
