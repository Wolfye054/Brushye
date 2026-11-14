//! Tools and sub tools. The tool column lists CSP 1.13.2's 23 tools in its order (spec U-004),
//! with Phosphor icons. Brush-like tools have sub tools: Brushye's own presets (never CSP's
//! defaults), grouped per tool, edited live and saved per user.

pub use brushye_command::tools::{SubTool, Tool as ToolId, ToolSet, ToolState};

use crate::icons;

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

/// The icon of a sub tool group tab: the built-in groups have their own, others the tool's.
pub fn group_icon(tool: ToolId, group: usize) -> &'static str {
    match (tool, group) {
        (ToolId::Pen, 1) => icons::HIGHLIGHTER,
        _ => info(tool).icon,
    }
}

/// Brush size palette presets (U-004).
pub const BRUSH_SIZES: &[f64] = &[
    0.7, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0, 12.0, 15.0, 17.0, 20.0, 25.0,
    30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 100.0, 120.0, 150.0, 170.0, 200.0, 250.0, 300.0, 400.0,
    500.0, 600.0, 700.0, 800.0, 1000.0, 1200.0, 1500.0, 1700.0, 2000.0,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_table_matches_the_command_layer() {
        assert_eq!(TOOLS.len(), 23);
        assert!(TOOLS.iter().map(|t| t.id).eq(ToolId::ALL.iter().copied()));
        assert_eq!(BRUSH_SIZES.len(), 40);
    }
}
