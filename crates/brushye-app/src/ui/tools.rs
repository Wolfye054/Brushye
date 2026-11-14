//! Tools and sub tools. The tool column lists CSP 1.13.2's 23 tools in its order (spec U-004),
//! with Phosphor icons. Brush-like tools have sub tools: Brushye's own presets (never CSP's
//! defaults), grouped per tool, edited live and saved per user.

use brushye_command::brush_preset::BrushPreset;
use serde_json::{Value as Json, json};

use crate::icons;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolId {
    Zoom,
    Hand,
    Operation,
    MoveLayer,
    Selection,
    AutoSelect,
    Eyedropper,
    Pen,
    Pencil,
    Brush,
    Airbrush,
    Decoration,
    Eraser,
    Blend,
    Liquify,
    Fill,
    Gradient,
    Figure,
    FrameBorder,
    Ruler,
    Text,
    Balloon,
    CorrectLine,
}

pub struct ToolInfo {
    pub id: ToolId,
    pub name: &'static str,
    pub icon: &'static str,
    pub shortcut: &'static str,
    /// Built in Brushye yet.
    pub available: bool,
    /// A separator line before this tool (U-004).
    pub separator_before: bool,
}

const fn t(
    id: ToolId,
    name: &'static str,
    icon: &'static str,
    shortcut: &'static str,
    available: bool,
    separator_before: bool,
) -> ToolInfo {
    ToolInfo {
        id,
        name,
        icon,
        shortcut,
        available,
        separator_before,
    }
}

/// The tool column, top to bottom (U-004 order and separators).
pub const TOOLS: &[ToolInfo] = &[
    t(
        ToolId::Zoom,
        "Zoom",
        icons::MAGNIFYING_GLASS,
        "/",
        true,
        false,
    ),
    t(ToolId::Hand, "Move (hand)", icons::HAND, "H", true, false),
    t(
        ToolId::Operation,
        "Operation",
        icons::CURSOR,
        "O",
        false,
        false,
    ),
    t(
        ToolId::MoveLayer,
        "Move layer",
        icons::ARROWS_OUT_CARDINAL,
        "K",
        false,
        false,
    ),
    t(
        ToolId::Selection,
        "Selection area",
        icons::SELECTION,
        "M",
        false,
        false,
    ),
    t(
        ToolId::AutoSelect,
        "Auto select",
        icons::MAGIC_WAND,
        "W",
        false,
        false,
    ),
    t(
        ToolId::Eyedropper,
        "Eyedropper",
        icons::EYEDROPPER,
        "I",
        true,
        false,
    ),
    t(ToolId::Pen, "Pen", icons::PEN_NIB, "P", true, true),
    t(
        ToolId::Pencil,
        "Pencil",
        icons::PENCIL_SIMPLE,
        "P",
        true,
        false,
    ),
    t(ToolId::Brush, "Brush", icons::PAINT_BRUSH, "B", true, false),
    t(
        ToolId::Airbrush,
        "Airbrush",
        icons::SPRAY_BOTTLE,
        "B",
        true,
        false,
    ),
    t(
        ToolId::Decoration,
        "Decoration",
        icons::SPARKLE,
        "B",
        false,
        false,
    ),
    t(ToolId::Eraser, "Eraser", icons::ERASER, "E", true, false),
    t(ToolId::Blend, "Blend", icons::DROP_HALF, "J", false, false),
    t(ToolId::Liquify, "Liquify", icons::WAVES, "J", false, false),
    t(ToolId::Fill, "Fill", icons::PAINT_BUCKET, "G", false, true),
    t(
        ToolId::Gradient,
        "Gradient",
        icons::GRADIENT,
        "G",
        false,
        false,
    ),
    t(ToolId::Figure, "Figure", icons::SHAPES, "U", false, false),
    t(
        ToolId::FrameBorder,
        "Frame border",
        icons::FRAME_CORNERS,
        "U",
        false,
        false,
    ),
    t(ToolId::Ruler, "Ruler", icons::RULER, "U", false, false),
    t(ToolId::Text, "Text", icons::TEXT_T, "T", false, false),
    t(
        ToolId::Balloon,
        "Balloon",
        icons::CHAT_CIRCLE,
        "T",
        false,
        false,
    ),
    t(
        ToolId::CorrectLine,
        "Correct line",
        icons::LINE_SEGMENTS,
        "Y",
        false,
        false,
    ),
];

pub fn info(id: ToolId) -> &'static ToolInfo {
    TOOLS.iter().find(|i| i.id == id).unwrap_or(&TOOLS[0])
}

impl ToolId {
    /// Tools that paint with sub tools (brush presets).
    pub fn is_brush(self) -> bool {
        matches!(
            self,
            ToolId::Pen | ToolId::Pencil | ToolId::Brush | ToolId::Airbrush | ToolId::Eraser
        )
    }

    pub fn key(self) -> &'static str {
        match self {
            ToolId::Pen => "pen",
            ToolId::Pencil => "pencil",
            ToolId::Brush => "brush",
            ToolId::Airbrush => "airbrush",
            ToolId::Eraser => "eraser",
            _ => "other",
        }
    }
}

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
    pub fn new(tool: ToolId, name: &str, preset: BrushPreset) -> Self {
        Self {
            name: name.into(),
            preset,
            shown: super::rows::default_shown(tool),
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
    pub icon: &'static str,
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

fn preset(text: &str) -> BrushPreset {
    // Defaults are written here and validated by tests; fall back to a plain round pen.
    BrushPreset::from_text(text).unwrap_or(BrushPreset {
        size: 1.0, // no default uses 1 px: the test catches a fallback
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
        size_pressure: None,
        size_curve: None,
        opacity_pressure: None,
        opacity_curve: None,
        density_pressure: None,
        density_curve: None,
    })
}

fn group(tool: ToolId, name: &str, icon: &'static str, items: &[(&str, &str)]) -> SubToolGroup {
    SubToolGroup {
        name: name.into(),
        icon,
        items: items
            .iter()
            .map(|(n, p)| SubTool::new(tool, n, preset(p)))
            .collect(),
        selected: 0,
    }
}

/// The built-in default of a sub tool, found by name.
pub fn builtin_default(tool: ToolId, name: &str) -> Option<BrushPreset> {
    default_set(tool)?
        .groups
        .into_iter()
        .flat_map(|g| g.items)
        .find(|s| s.name == name)
        .map(|s| s.preset)
}

/// Brushye's own default sub tools (original names and settings).
pub fn default_set(tool: ToolId) -> Option<ToolSet> {
    let groups = match tool {
        ToolId::Pen => vec![
            group(
                tool,
                "Pen",
                icons::PEN_NIB,
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
                icons::HIGHLIGHTER,
                &[
                    ("Marker", "size=14.0"),
                    ("Fine liner", "size=3.0 aa=weak"),
                    ("Chisel marker", "size=18.0 thickness=35 angle=45.0"),
                ],
            ),
        ],
        ToolId::Pencil => vec![group(
            tool,
            "Pencil",
            icons::PENCIL_SIMPLE,
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
        ToolId::Brush => vec![group(
            tool,
            "Brush",
            icons::PAINT_BRUSH,
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
        ToolId::Airbrush => vec![group(
            tool,
            "Airbrush",
            icons::SPRAY_BOTTLE,
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
        ToolId::Eraser => vec![group(
            tool,
            "Eraser",
            icons::ERASER,
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

/// Brush size palette presets (U-004).
pub const BRUSH_SIZES: &[f64] = &[
    0.7, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 15.0, 17.0, 20.0, 25.0,
    30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 100.0, 120.0, 150.0, 170.0, 200.0, 250.0, 300.0, 400.0,
    500.0, 600.0, 700.0, 800.0, 1000.0, 1200.0, 1500.0, 1700.0, 2000.0,
];

// ---------------------------------------------------------------- persistence

pub fn to_json(set: &ToolSet) -> Json {
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

/// Restore a saved set onto the defaults (group icons come from the defaults).
pub fn from_json(tool: ToolId, j: &Json) -> Option<ToolSet> {
    let defaults = default_set(tool)?;
    let groups = j["groups"].as_array()?;
    let mut out = Vec::new();
    for (i, g) in groups.iter().enumerate() {
        let items: Vec<SubTool> = g["items"]
            .as_array()?
            .iter()
            .filter_map(|s| {
                let preset = BrushPreset::from_json(s["settings"].as_object()?).ok()?;
                let name = s["name"].as_str()?;
                let mut st = SubTool::new(tool, name, preset);
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
        let icon = defaults.groups.get(i).map_or(info(tool).icon, |d| d.icon);
        out.push(SubToolGroup {
            name: g["name"].as_str()?.to_owned(),
            icon,
            selected: (g["selected"].as_u64()? as usize).min(items.len().saturating_sub(1)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_round_trip() {
        for t in TOOLS {
            let Some(set) = default_set(t.id) else {
                assert!(!t.id.is_brush(), "{:?}", t.id);
                continue;
            };
            for g in &set.groups {
                for s in &g.items {
                    assert!(s.preset.validate().is_ok(), "{}", s.name);
                    // A broken default would silently fall back to size 1.
                    assert!(s.preset.size != 1.0, "{}", s.name);
                }
            }
            assert_eq!(from_json(t.id, &to_json(&set)), Some(set));
        }
        assert_eq!(TOOLS.len(), 23);
        assert_eq!(BRUSH_SIZES.len(), 40);
    }
}
