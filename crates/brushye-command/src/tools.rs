//! Tools, sub tools and the drawing colour chips: application state, changed only by the
//! `tool.*`, `subtool.*` and `color.*` commands (DECISIONS D-018).
//!
//! The tool column lists CSP 1.13.2's 23 tools in its order (spec U-004). Brush-like tools have
//! sub tools: Brushye's own presets (never CSP's defaults), in group tabs, edited live and saved
//! per user. A sub tool is addressed by tool, group index and item index, all 0-based; left
//! out, each defaults to the current one.
//!
//! These commands never change the document, so they are not undoable (as in CSP, where sub
//! tool edits are not on the History palette) and not written to the document journal: a
//! stroke already records every setting and its colour.

use std::collections::BTreeMap;

use serde_json::{Map, Value as Json, json};

use crate::brush_preset::BrushPreset;
use crate::commands::Command;
use crate::params::Arg;
use crate::syntax::Value;

macro_rules! tools {
    ($( $v:ident = $key:literal ),* $(,)?) => {
        /// A tool of the tool column (U-004 order).
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Tool { $( $v ),* }

        impl Tool {
            /// All tools, top to bottom.
            pub const ALL: &'static [Tool] = &[ $( Tool::$v ),* ];
            pub fn name(self) -> &'static str {
                match self { $( Tool::$v => $key ),* }
            }
            pub fn from_name(s: &str) -> Option<Self> {
                match s { $( $key => Some(Tool::$v), )* _ => None }
            }
        }
    };
}

tools! {
    Zoom = "zoom",
    Hand = "hand",
    Operation = "operation",
    MoveLayer = "move_layer",
    Selection = "selection",
    AutoSelect = "auto_select",
    Eyedropper = "eyedropper",
    Pen = "pen",
    Pencil = "pencil",
    Brush = "brush",
    Airbrush = "airbrush",
    Decoration = "decoration",
    Eraser = "eraser",
    Blend = "blend",
    Liquify = "liquify",
    Fill = "fill",
    Gradient = "gradient",
    Figure = "figure",
    FrameBorder = "frame_border",
    Ruler = "ruler",
    Text = "text",
    Balloon = "balloon",
    CorrectLine = "correct_line",
}

impl Tool {
    /// Tools that paint with sub tools (brush presets).
    pub fn is_brush(self) -> bool {
        matches!(
            self,
            Tool::Pen | Tool::Pencil | Tool::Brush | Tool::Airbrush | Tool::Eraser
        )
    }
    /// The settings key (same as the name).
    pub fn key(self) -> &'static str {
        self.name()
    }
}

/// A drawing colour chip (U-013).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Chip {
    #[default]
    Main,
    Sub,
    Transparent,
}

impl Chip {
    pub fn name(self) -> &'static str {
        match self {
            Chip::Main => "main",
            Chip::Sub => "sub",
            Chip::Transparent => "transparent",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "main" => Some(Chip::Main),
            "sub" => Some(Chip::Sub),
            "transparent" => Some(Chip::Transparent),
            _ => None,
        }
    }
}

macro_rules! word_arg {
    ($t:ty, $kind:literal) => {
        impl Arg for $t {
            const KIND: &'static str = $kind;
            fn from_value(v: &Value) -> Result<Self, String> {
                match v {
                    Value::Word(w) => {
                        <$t>::from_name(w).ok_or_else(|| format!("unknown {} '{w}'", $kind))
                    }
                    other => Err(format!("expected a {}, found {}", $kind, other.kind())),
                }
            }
            fn to_value(&self) -> Value {
                Value::Word(self.name().to_owned())
            }
            fn to_json(&self) -> Json {
                Json::String(self.name().to_owned())
            }
            fn from_json(j: &Json) -> Result<Self, String> {
                let s = j
                    .as_str()
                    .ok_or(concat!("expected a ", $kind, " name in JSON"))?;
                <$t>::from_name(s).ok_or_else(|| format!("unknown {} '{s}'", $kind))
            }
        }
    };
}

word_arg!(Tool, "tool");
word_arg!(Chip, "colour chip (main|sub|transparent)");

/// One sub tool: a named brush preset, plus how it shows in the palettes (U-010, U-011).
#[derive(Clone, Debug, PartialEq)]
pub struct SubTool {
    pub name: String,
    pub preset: BrushPreset,
    /// Row keys shown in Tool property (the Sub Tool Detail eye buttons).
    pub shown: Vec<String>,
    /// Locked: edits work but aren't kept; the sub tool reverts to `base` when deselected.
    pub base: Option<BrushPreset>,
    /// "Save as default": what Reset to default restores (else the built-in default).
    pub user_default: Option<BrushPreset>,
}

impl SubTool {
    pub fn new(tool: Tool, name: &str, preset: BrushPreset) -> Self {
        Self {
            name: name.into(),
            preset,
            shown: default_shown(tool),
            base: None,
            user_default: None,
        }
    }

    pub fn locked(&self) -> bool {
        self.base.is_some()
    }
}

/// A group tab of sub tools (U-004: e.g. two groups for the Pen tool).
#[derive(Clone, Debug, PartialEq)]
pub struct SubToolGroup {
    pub name: String,
    pub items: Vec<SubTool>,
    pub selected: usize,
}

/// The sub tools of one tool.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolSet {
    pub groups: Vec<SubToolGroup>,
    pub group: usize,
}

impl ToolSet {
    pub fn current(&self) -> Option<&SubTool> {
        let g = self.groups.get(self.group)?;
        g.items.get(g.selected)
    }
    pub fn current_mut(&mut self) -> Option<&mut SubTool> {
        let g = self.groups.get_mut(self.group)?;
        g.items.get_mut(g.selected)
    }
}

/// Tool property rows shown by default (keys of the Sub Tool Detail rows).
pub fn default_shown(tool: Tool) -> Vec<String> {
    let keys: &[&str] = match tool {
        Tool::Brush | Tool::Airbrush => {
            &["size", "opacity", "hardness", "density", "stabilization"]
        }
        Tool::Eraser => &["size", "aa", "hardness"],
        _ => &[
            "size",
            "opacity",
            "aa",
            "stabilization",
            "adjust_by_speed",
            "vector_magnet",
        ],
    };
    keys.iter().map(|k| (*k).to_owned()).collect()
}

fn preset(text: &str) -> BrushPreset {
    // Defaults are written here and validated by tests; a broken one falls back to 1 px, which
    // no default uses, so the test catches it.
    BrushPreset::from_text(text)
        .or_else(|_| BrushPreset::from_text("size=1"))
        .unwrap_or_else(|e| unreachable!("{e}"))
}

fn group(tool: Tool, name: &str, items: &[(&str, &str)]) -> SubToolGroup {
    SubToolGroup {
        name: name.into(),
        items: items
            .iter()
            .map(|(n, p)| SubTool::new(tool, n, preset(p)))
            .collect(),
        selected: 0,
    }
}

/// Brushye's own default sub tools (original names and settings).
pub fn default_set(tool: Tool) -> Option<ToolSet> {
    let groups = match tool {
        Tool::Pen => vec![
            group(
                tool,
                "Pen",
                &[
                    ("Round pen", "size=10.0 size_pressure=10"),
                    ("Smooth ink", "size=10.0 size_pressure=5 stabilization=6"),
                    ("Crisp pen", "size=6.0 aa=weak size_pressure=30"),
                    (
                        "Pressure ink",
                        "size=12.0 size_pressure=0 size_curve=[0.5,0.3]",
                    ),
                ],
            ),
            group(
                tool,
                "Marker",
                &[
                    ("Marker", "size=14.0"),
                    ("Fine liner", "size=3.0 aa=weak"),
                    ("Chisel marker", "size=18.0 thickness=35 angle=45.0"),
                ],
            ),
        ],
        Tool::Pencil => vec![group(
            tool,
            "Pencil",
            &[
                (
                    "Pencil",
                    "size=6.0 hardness=4 density=80 density_pressure=10",
                ),
                (
                    "Soft lead",
                    "size=10.0 hardness=3 opacity=80 density_pressure=0",
                ),
                (
                    "Sketch",
                    "size=4.0 hardness=4 density_pressure=20 size_pressure=60",
                ),
            ],
        )],
        Tool::Brush => vec![group(
            tool,
            "Brush",
            &[
                (
                    "Soft round",
                    "size=40.0 hardness=2 opacity=90 density_pressure=10",
                ),
                ("Hard round", "size=30.0 size_pressure=40"),
                (
                    "Flat brush",
                    "size=36.0 thickness=30 angle_line=true size_pressure=50",
                ),
                (
                    "Glaze",
                    "size=50.0 hardness=2 density=30 density_pressure=0",
                ),
            ],
        )],
        Tool::Airbrush => vec![group(
            tool,
            "Airbrush",
            &[
                (
                    "Soft airbrush",
                    "size=120.0 hardness=1 density=20 density_pressure=0",
                ),
                (
                    "Fine airbrush",
                    "size=30.0 hardness=1 density=30 density_pressure=0",
                ),
            ],
        )],
        Tool::Eraser => vec![group(
            tool,
            "Eraser",
            &[
                ("Hard eraser", "size=30.0 ink=erase"),
                ("Soft eraser", "size=60.0 hardness=2 ink=erase"),
                ("Fine eraser", "size=6.0 ink=erase aa=weak"),
            ],
        )],
        _ => return None,
    };
    Some(ToolSet { groups, group: 0 })
}

/// The built-in default of a sub tool, found by name.
pub fn builtin_default(tool: Tool, name: &str) -> Option<BrushPreset> {
    default_set(tool)?
        .groups
        .into_iter()
        .flat_map(|g| g.items)
        .find(|s| s.name == name)
        .map(|s| s.preset)
}

// ---------------------------------------------------------------- persistence

/// JSON form of one tool's sub tools (the user settings file).
pub fn set_to_json(set: &ToolSet) -> Json {
    json!({
        "group": set.group,
        "groups": set.groups.iter().map(|g| json!({
            "name": g.name,
            "selected": g.selected,
            "items": g.items.iter().map(|s| json!({
                "name": s.name,
                // A locked sub tool saves its locked settings, not the edits (U-010).
                "settings": s.base.as_ref().unwrap_or(&s.preset).to_json(),
                "shown": s.shown,
                "locked": s.locked(),
                "default": s.user_default.as_ref().map(|p| p.to_json()),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// Read one tool's saved sub tools; `None` if the JSON doesn't describe a usable set.
pub fn set_from_json(tool: Tool, j: &Json) -> Option<ToolSet> {
    default_set(tool)?;
    let mut out = Vec::new();
    for g in j["groups"].as_array()? {
        let items: Vec<SubTool> = g["items"]
            .as_array()?
            .iter()
            .filter_map(|s| {
                let preset = BrushPreset::from_json(s["settings"].as_object()?).ok()?;
                let mut st = SubTool::new(tool, s["name"].as_str()?, preset);
                if let Some(a) = s["shown"].as_array() {
                    st.shown = a
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect();
                }
                if s["locked"].as_bool() == Some(true) {
                    st.base = Some(st.preset.clone());
                }
                st.user_default = s["default"]
                    .as_object()
                    .and_then(|d| BrushPreset::from_json(d).ok());
                Some(st)
            })
            .collect();
        if items.is_empty() {
            continue;
        }
        out.push(SubToolGroup {
            name: g["name"].as_str()?.to_owned(),
            selected: (g["selected"].as_u64()? as usize).min(items.len() - 1),
            items,
        });
    }
    if out.is_empty() {
        return None;
    }
    Some(ToolSet {
        group: (j["group"].as_u64()? as usize).min(out.len() - 1),
        groups: out,
    })
}

// ---------------------------------------------------------------- state

/// Tools, sub tools and colour chips of one Brushye instance.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolState {
    pub tool: Tool,
    pub sets: BTreeMap<Tool, ToolSet>,
    pub main: [u8; 3],
    pub sub: [u8; 3],
    pub chip: Chip,
    /// Bumped by every successful change (for UIs that mirror this state).
    pub generation: u64,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            tool: Tool::Pen,
            sets: Tool::ALL
                .iter()
                .filter_map(|t| default_set(*t).map(|s| (*t, s)))
                .collect(),
            main: [32, 32, 40],
            sub: [255, 255, 255],
            chip: Chip::Main,
            generation: 0,
        }
    }
}

/// The address of one sub tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub tool: Tool,
    pub group: usize,
    pub item: usize,
}

fn idx(v: Option<u32>) -> Option<usize> {
    v.map(|v| v as usize)
}

impl ToolState {
    /// Defaults with the user's saved sub tools (settings JSON keyed by tool name) on top.
    pub fn with_saved(saved: &Map<String, Json>) -> Self {
        let mut s = Self::default();
        for (tool, set) in &mut s.sets {
            if let Some(v) = saved.get(tool.key()).and_then(|j| set_from_json(*tool, j)) {
                *set = v;
            }
        }
        s
    }

    /// The sub tools as settings JSON keyed by tool name.
    pub fn saved(&self) -> Map<String, Json> {
        self.sets
            .iter()
            .map(|(t, s)| (t.key().to_owned(), set_to_json(s)))
            .collect()
    }

    pub fn set(&self, tool: Tool) -> Option<&ToolSet> {
        self.sets.get(&tool)
    }

    /// The current sub tool of a tool.
    pub fn current(&self, tool: Tool) -> Option<&SubTool> {
        self.sets.get(&tool).and_then(ToolSet::current)
    }

    /// The colour of the active chip; `None` for transparent.
    pub fn color(&self) -> Option<[u8; 3]> {
        match self.chip {
            Chip::Main => Some(self.main),
            Chip::Sub => Some(self.sub),
            Chip::Transparent => None,
        }
    }

    /// Resolve an address; left-out parts are the current ones.
    pub fn target(
        &self,
        tool: Option<Tool>,
        group: Option<u32>,
        item: Option<u32>,
    ) -> Result<Target, String> {
        let tool = tool.unwrap_or(self.tool);
        let set = self
            .sets
            .get(&tool)
            .ok_or_else(|| format!("the {} tool has no sub tools", tool.name()))?;
        let group = idx(group).unwrap_or(set.group);
        let g = set
            .groups
            .get(group)
            .ok_or_else(|| format!("{} has no group {group}", tool.name()))?;
        let item = idx(item).unwrap_or(g.selected);
        if item >= g.items.len() {
            return Err(format!(
                "{} group {group} has no sub tool {item}",
                tool.name()
            ));
        }
        Ok(Target { tool, group, item })
    }

    fn group_mut(&mut self, t: Target) -> &mut SubToolGroup {
        &mut self
            .sets
            .get_mut(&t.tool)
            .unwrap_or_else(|| unreachable!("target checked"))
            .groups[t.group]
    }

    fn sub_mut(&mut self, t: Target) -> &mut SubTool {
        &mut self.group_mut(t).items[t.item]
    }

    pub fn get(&self, t: Target) -> &SubTool {
        &self.sets[&t.tool].groups[t.group].items[t.item]
    }

    /// Make `t` the selected sub tool; a locked sub tool that is left reverts (U-010).
    fn select(&mut self, t: Target) {
        let set = self
            .sets
            .get_mut(&t.tool)
            .unwrap_or_else(|| unreachable!("target checked"));
        if (set.group, set.groups[set.group].selected) != (t.group, t.item)
            && let Some(s) = set.current_mut()
            && let Some(base) = s.base.clone()
        {
            s.preset = base;
        }
        set.group = t.group;
        set.groups[t.group].selected = t.item;
    }

    /// Apply a tool command. Fails without changing anything.
    pub fn apply(&mut self, cmd: &Command) -> Result<(), String> {
        let mut next = self.clone();
        next.apply_in_place(cmd)?;
        next.generation = self.generation + 1;
        *self = next;
        Ok(())
    }

    fn apply_in_place(&mut self, cmd: &Command) -> Result<(), String> {
        match cmd {
            Command::ToolSelect(p) => self.tool = p.tool,
            Command::SubtoolSelect(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                self.tool = t.tool;
                self.select(t);
            }
            Command::SubtoolSet(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let s = self.sub_mut(t);
                if let Some(n) = &p.name {
                    let n = n.trim();
                    if n.is_empty() {
                        return Err("a sub tool name can't be empty".into());
                    }
                    s.name = n.to_owned();
                }
                let mut m = s.preset.to_json();
                for k in p.clear.as_deref().unwrap_or("").split(',') {
                    let k = k.trim();
                    if k.is_empty() {
                        continue;
                    }
                    if k == "size" || !BrushPreset::PARAMS.iter().any(|q| q.name == k) {
                        return Err(format!("can't clear '{k}'"));
                    }
                    m.remove(k);
                }
                for (k, v) in p.settings_json() {
                    m.insert(k, v);
                }
                s.preset = BrushPreset::from_json(&m).map_err(|e| format!("subtool.set: {e}"))?;
            }
            Command::SubtoolAdd(p) => {
                let tool = p.tool.unwrap_or(self.tool);
                let first = default_set(tool)
                    .and_then(|d| d.groups.into_iter().next())
                    .and_then(|g| g.items.into_iter().next())
                    .ok_or_else(|| format!("the {} tool has no sub tools", tool.name()))?;
                let preset = match &p.settings {
                    Some(text) => BrushPreset::from_text(text)?,
                    None => first.preset,
                };
                let name = p.name.trim();
                if name.is_empty() {
                    return Err("a sub tool name can't be empty".into());
                }
                let set = &self.sets[&tool];
                let group = idx(p.group).unwrap_or(set.group);
                if group >= set.groups.len() {
                    return Err(format!("{} has no group {group}", tool.name()));
                }
                let t = Target {
                    tool,
                    group,
                    item: set.groups[group].items.len(),
                };
                self.group_mut(t)
                    .items
                    .push(SubTool::new(tool, name, preset));
                self.tool = tool;
                self.select(t);
            }
            Command::SubtoolCopy(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let mut s = self.get(t).clone();
                s.base = None;
                s.name = match &p.name {
                    Some(n) if !n.trim().is_empty() => n.trim().to_owned(),
                    Some(_) => return Err("a sub tool name can't be empty".into()),
                    None => format!("{} 2", s.name),
                };
                let to = Target {
                    item: t.item + 1,
                    ..t
                };
                // Selecting first lets a locked source revert; the copy keeps the edits.
                self.select(t);
                self.group_mut(t).items.insert(to.item, s);
                self.group_mut(t).selected = to.item;
                self.tool = t.tool;
            }
            Command::SubtoolDelete(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let g = self.group_mut(t);
                if g.items.len() < 2 {
                    return Err("a group keeps at least one sub tool".into());
                }
                g.items.remove(t.item);
                if g.selected > t.item || g.selected >= g.items.len() {
                    g.selected = g.selected.saturating_sub(1);
                }
            }
            Command::SubtoolMove(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let g = self.group_mut(t);
                let to = p.to as usize;
                if to >= g.items.len() {
                    return Err(format!("position {to} is past the end of the group"));
                }
                let selected = g.selected == t.item;
                let s = g.items.remove(t.item);
                g.items.insert(to, s);
                if selected {
                    g.selected = to;
                } else if t.item < g.selected && to >= g.selected {
                    g.selected -= 1;
                } else if t.item > g.selected && to <= g.selected {
                    g.selected += 1;
                }
            }
            Command::SubtoolLock(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let s = self.sub_mut(t);
                match (p.locked, s.locked()) {
                    (true, false) => s.base = Some(s.preset.clone()),
                    (false, true) => s.base = None,
                    _ => {}
                }
            }
            Command::SubtoolReset(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let s = self.get(t);
                let d = s
                    .user_default
                    .clone()
                    .or_else(|| builtin_default(t.tool, &s.name))
                    .ok_or_else(|| {
                        format!(
                            "“{}” has no default (register one with subtool.save_default)",
                            s.name
                        )
                    })?;
                let s = self.sub_mut(t);
                if s.locked() {
                    s.base = Some(d.clone());
                }
                s.preset = d;
            }
            Command::SubtoolSaveDefault(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                let s = self.sub_mut(t);
                s.user_default = Some(s.preset.clone());
            }
            Command::SubtoolShowRow(p) => {
                let t = self.target(p.tool, p.group, p.item)?;
                if p.row.is_empty() || !p.row.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
                    return Err(format!("'{}' is not a row key", p.row));
                }
                let s = self.sub_mut(t);
                let has = s.shown.contains(&p.row);
                if p.shown && !has {
                    s.shown.push(p.row.clone());
                } else if !p.shown {
                    s.shown.retain(|k| *k != p.row);
                }
            }
            Command::SubtoolAddDefaults(p) => {
                let tool = p.tool.unwrap_or(self.tool);
                let d = default_set(tool)
                    .ok_or_else(|| format!("the {} tool has no sub tools", tool.name()))?;
                let set = self
                    .sets
                    .get_mut(&tool)
                    .ok_or_else(|| format!("the {} tool has no sub tools", tool.name()))?;
                // Add back the built-in sub tools missing from their group (and lost groups).
                for (i, dg) in d.groups.into_iter().enumerate() {
                    match set.groups.get_mut(i) {
                        Some(g) => {
                            for s in dg.items {
                                if !g.items.iter().any(|x| x.name == s.name) {
                                    g.items.push(s);
                                }
                            }
                        }
                        None => set.groups.push(dg),
                    }
                }
            }
            Command::SubtoolGroupSet(p) => {
                let tool = p.tool.unwrap_or(self.tool);
                let set = self
                    .sets
                    .get_mut(&tool)
                    .ok_or_else(|| format!("the {} tool has no sub tools", tool.name()))?;
                let g = set
                    .groups
                    .get_mut(p.group as usize)
                    .ok_or_else(|| format!("{} has no group {}", tool.name(), p.group))?;
                let n = p.name.trim();
                if n.is_empty() {
                    return Err("a group name can't be empty".into());
                }
                g.name = n.to_owned();
            }
            Command::ColorSet(p) => {
                if p.color.0[3] != 255 {
                    return Err("color.set: the colour must be opaque (#rrggbb)".into());
                }
                let rgb = [p.color.0[0], p.color.0[1], p.color.0[2]];
                match p.chip {
                    Chip::Main => self.main = rgb,
                    Chip::Sub => self.sub = rgb,
                    Chip::Transparent => {
                        return Err("the transparent chip has no colour".into());
                    }
                }
            }
            Command::ColorActive(p) => self.chip = p.chip,
            other => return Err(format!("{} is not a tool command", other.name())),
        }
        Ok(())
    }

    // ------------------------------------------------------------ queries

    /// `?tools`: the current tool, colours and every tool's groups and sub tool names.
    pub fn tools_json(&self) -> Json {
        json!({
            "tool": self.tool.name(),
            "tools": Tool::ALL.iter().map(|t| match self.sets.get(t) {
                Some(set) => json!({
                    "tool": t.name(),
                    "group": set.group,
                    "groups": set.groups.iter().map(|g| json!({
                        "name": g.name,
                        "selected": g.selected,
                        "items": g.items.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                }),
                None => json!({ "tool": t.name() }),
            }).collect::<Vec<_>>(),
        })
    }

    /// `?subtool`: one sub tool in full.
    pub fn subtool_json(&self, t: Target) -> Json {
        let s = self.get(t);
        json!({
            "tool": t.tool.name(),
            "group": t.group,
            "item": t.item,
            "name": s.name,
            "settings": s.preset.to_text(),
            "shown": s.shown,
            "locked": s.locked(),
            "user_default": s.user_default.as_ref().map(BrushPreset::to_text),
        })
    }

    /// `?color`: the chips.
    pub fn color_json(&self) -> Json {
        let hex = |c: [u8; 3]| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
        json!({ "main": hex(self.main), "sub": hex(self.sub), "active": self.chip.name() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;

    #[test]
    fn defaults_are_valid_and_round_trip() {
        for t in Tool::ALL {
            let Some(set) = default_set(*t) else {
                assert!(!t.is_brush(), "{t:?}");
                continue;
            };
            for g in &set.groups {
                for s in &g.items {
                    assert!(s.preset.validate().is_ok(), "{}", s.name);
                    assert!(s.preset.size != 1.0, "{}", s.name);
                }
            }
            assert_eq!(set_from_json(*t, &set_to_json(&set)), Some(set));
        }
        assert_eq!(Tool::ALL.len(), 23);
        let s = ToolState::default();
        assert_eq!(ToolState::with_saved(&s.saved()), s);
    }

    fn run(s: &mut Session, script: &str) {
        if let Err(e) = s.run_script(script) {
            unreachable!("{e}");
        }
    }

    fn q(s: &mut Session, line: &str) -> Json {
        match s.run_line(line) {
            Ok(Some(j)) => j,
            other => unreachable!("{other:?}"),
        }
    }

    #[test]
    fn sub_tool_commands() {
        let mut s = Session::new();
        run(
            &mut s,
            "tool.select brush\nsubtool.select item=1\nsubtool.set size=12.5 hardness=3 name=\"Mine\"",
        );
        let j = q(&mut s, "?subtool");
        assert_eq!(j["name"], "Mine");
        assert_eq!(j["tool"], "brush");
        assert_eq!(j["settings"], "size=12.5 hardness=3 size_pressure=40");
        run(&mut s, "subtool.set clear=\"size_pressure,hardness\"");
        assert_eq!(q(&mut s, "?subtool")["settings"], "size=12.5");
        // Copy goes after the source and is selected; move and delete keep the selection.
        run(
            &mut s,
            "subtool.copy name=\"Mine copy\"\nsubtool.move item=2 to=0",
        );
        let j = q(&mut s, "?tools");
        let brush = &j["tools"][9];
        assert_eq!(brush["tool"], "brush");
        assert_eq!(
            brush["groups"][0]["items"],
            json!(["Mine copy", "Soft round", "Mine", "Flat brush", "Glaze"])
        );
        assert_eq!(brush["groups"][0]["selected"], 0);
        run(&mut s, "subtool.delete item=1");
        assert_eq!(q(&mut s, "?subtool")["name"], "Mine copy");
        // Reset: a renamed sub tool has no built-in default until one is saved.
        assert!(s.run_line("subtool.reset item=1").is_err());
        run(
            &mut s,
            "subtool.save_default\nsubtool.set size=99\nsubtool.reset",
        );
        assert_eq!(q(&mut s, "?subtool")["settings"], "size=12.5");
        // Bad commands change nothing.
        let before = s.tools().clone();
        assert!(s.run_line("subtool.set size=5 hardness=9").is_err());
        assert!(s.run_line("subtool.set clear=\"size\"").is_err());
        assert!(s.run_line("subtool.select group=4").is_err());
        assert!(s.run_line("tool.select zoom\nsubtool.set size=3").is_err());
        assert_eq!(s.tools(), &before);
    }

    #[test]
    fn locked_sub_tool_reverts_when_left() {
        let mut s = Session::new();
        run(
            &mut s,
            "subtool.lock locked=true\nsubtool.set size=40\nsubtool.select item=1",
        );
        assert_eq!(
            q(&mut s, "?subtool item=0")["settings"],
            "size=10.0 size_pressure=10"
        );
        assert_eq!(q(&mut s, "?subtool item=0")["locked"], true);
        // Re-selecting the current one keeps the edits.
        run(
            &mut s,
            "subtool.select item=0\nsubtool.set size=40\nsubtool.select item=0",
        );
        assert_eq!(
            q(&mut s, "?subtool")["settings"],
            "size=40.0 size_pressure=10"
        );
    }

    #[test]
    fn add_defaults_rows_groups_and_colours() {
        let mut s = Session::new();
        run(
            &mut s,
            "subtool.delete item=0\nsubtool.add_defaults\nsubtool.show_row row=\"hardness\" shown=true\nsubtool.show_row row=\"size\" shown=false\nsubtool.group_set 1 name=\"Markers\"\nsubtool.add name=\"Wide\" settings=\"size=50 hardness=2\"",
        );
        let j = q(&mut s, "?tools");
        assert_eq!(
            j["tools"][7]["groups"][0]["items"],
            json!([
                "Smooth ink",
                "Crisp pen",
                "Pressure ink",
                "Round pen",
                "Wide"
            ])
        );
        assert_eq!(j["tools"][7]["groups"][1]["name"], "Markers");
        let j = q(&mut s, "?subtool item=0");
        assert_eq!(
            j["shown"],
            json!([
                "opacity",
                "aa",
                "stabilization",
                "adjust_by_speed",
                "vector_magnet",
                "hardness"
            ])
        );
        assert_eq!(q(&mut s, "?subtool")["settings"], "size=50.0 hardness=2");
        run(&mut s, "color.set sub color=#ff0000\ncolor.active sub");
        assert_eq!(
            q(&mut s, "?color"),
            json!({"main": "#202028", "sub": "#ff0000", "active": "sub"})
        );
        assert!(s.run_line("color.set transparent color=#ff0000").is_err());
        assert!(s.run_line("color.set main color=#ff000080").is_err());
    }

    #[test]
    fn tool_commands_stay_out_of_the_journal() {
        let mut s = Session::new();
        run(
            &mut s,
            "doc.new 8 8\ntool.select eraser\nsubtool.set size=3\ncolor.set main color=#102030",
        );
        assert_eq!(s.journal_len(), 1);
        assert!(Session::replay(&s.journal_text()).is_ok());
        assert_eq!(s.tools().tool, Tool::Eraser);
        // doc.new keeps the tools, like the pressure curve.
        run(&mut s, "doc.new 4 4");
        assert_eq!(s.tools().main, [16, 32, 48]);
    }
}
