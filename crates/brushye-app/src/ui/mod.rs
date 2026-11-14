//! The window layout, matching CSP 1.13.2's default workspace (specs U-001 … U-009): menu bar;
//! tool column; left dock (Sub Tool, Tool property, Brush size, colour group); canvas column
//! (command bar, document tabs, canvas, status strip, timeline dock); icon column; right dock
//! (Navigator group, Layer Property group, Layer group). Icons are Phosphor, not CSP's.

pub mod chrome;
pub mod colour;
pub mod dialogs;
pub mod left;
pub mod menus;
pub mod right;
pub mod theme;
pub mod tools;
pub mod widgets;

use std::collections::HashMap;

use eframe::egui::{self, Rect, TextureHandle, pos2, vec2};

use crate::app::App;
use theme::{Interface, m};
use tools::{ToolId, ToolSet};

/// Which palette groups are shown (Window menu).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Visible {
    pub command_bar: bool,
    pub tool: bool,
    pub left_dock: bool,
    pub right_dock: bool,
    pub icon_column: bool,
    pub scroll_bars: bool,
    /// Tab: hide all palettes.
    pub hide_all: bool,
}

impl Default for Visible {
    fn default() -> Self {
        Self {
            command_bar: true,
            tool: true,
            left_dock: true,
            right_dock: true,
            icon_column: true,
            scroll_bars: true,
            hide_all: false,
        }
    }
}

/// UI state that isn't part of the document.
pub struct UiState {
    pub interface: Interface,
    pub applied_interface: Option<Interface>,
    pub tool: ToolId,
    pub sets: HashMap<ToolId, ToolSet>,
    pub sets_dirty: bool,
    pub colour: colour::ColourState,
    pub previews: HashMap<String, TextureHandle>,
    pub detail_open: bool,
    pub dynamics_popup: Option<left::DynTarget>,
    pub pending_subtool_op: Option<left::SubToolOp>,
    pub import_brush: Option<String>,
    pub eyedropper_layer: bool,
    pub nav_tab: usize,
    pub layer_prop_tab: usize,
    pub layer_tab: usize,
    pub visible: Visible,
    pub thumbs: HashMap<u32, (Vec<usize>, TextureHandle)>,
    pub nav_texture: Option<(u64, TextureHandle)>,
    pub renaming: Option<(u32, String)>,
    pub about_open: bool,
    pub prefs_open: bool,
}

impl UiState {
    pub fn new(settings: &crate::settings::Settings) -> Self {
        let mut sets = HashMap::new();
        for t in tools::TOOLS {
            if let Some(d) = tools::default_set(t.id) {
                let saved = settings
                    .subtools
                    .get(t.id.key())
                    .and_then(|j| tools::from_json(t.id, j));
                sets.insert(t.id, saved.unwrap_or(d));
            }
        }
        Self {
            interface: Interface::from_name(&settings.interface),
            applied_interface: None,
            tool: ToolId::Pen,
            sets,
            sets_dirty: false,
            colour: colour::ColourState::default(),
            previews: HashMap::new(),
            detail_open: false,
            dynamics_popup: None,
            pending_subtool_op: None,
            import_brush: None,
            eyedropper_layer: false,
            nav_tab: 0,
            layer_prop_tab: 0,
            layer_tab: 0,
            visible: Visible::default(),
            thumbs: HashMap::new(),
            nav_texture: None,
            renaming: None,
            about_open: false,
            prefs_open: false,
        }
    }

    pub fn colors(&self) -> &'static theme::Colors {
        theme::colors(self.interface)
    }

    /// The current sub tool's preset, for brush tools.
    pub fn current_preset(
        &self,
        tool: ToolId,
    ) -> Option<&brushye_command::brush_preset::BrushPreset> {
        self.sets
            .get(&tool)
            .and_then(|s| s.current())
            .map(|s| &s.preset)
    }

    /// Workspace > Reset to Default: show everything in its default place.
    pub fn reset_layout(&mut self) {
        self.visible = Visible::default();
        self.nav_tab = 0;
        self.layer_prop_tab = 0;
        self.layer_tab = 0;
        self.colour.tab = 0;
    }
}

/// Split a column height between fixed palette groups (scaled down if the window is short) and
/// a last group that takes the rest (U-001: the Layer group takes the remaining height).
fn stack(top: f32, bottom: f32, fixed: &[f32], gap: f32) -> Vec<(f32, f32)> {
    let avail = (bottom - top).max(1.0);
    let want: f32 = fixed.iter().sum::<f32>() + gap * fixed.len() as f32;
    let k = if want > avail * 0.85 {
        avail * 0.85 / want
    } else {
        1.0
    };
    let mut out = Vec::new();
    let mut y = top;
    for h in fixed {
        out.push((y, y + h * k));
        y += h * k + gap;
    }
    out.push((y, bottom));
    out
}

pub fn frame(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    if app.ui.applied_interface != Some(app.ui.interface) {
        theme::apply(&ctx, app.ui.interface);
        app.ui.applied_interface = Some(app.ui.interface);
    }
    dialogs::window_title(app, &ctx);
    menus::shortcuts(app, &ctx);
    dialogs::close_request(app, &ctx);
    let c = app.ui.colors();

    // Menu bar across the full width.
    egui::Panel::top("menu")
        .exact_size(m::MENU_BAR)
        .frame(
            egui::Frame::NONE
                .fill(c.bar)
                .inner_margin(egui::Margin::symmetric(6, 2)),
        )
        .show(ui, |ui| menus::menu_bar(app, ui));

    let full = ui.available_rect_before_wrap();
    let vis = app.ui.visible;
    let show = |on: bool| on && !vis.hide_all;
    let mut x0 = full.left();
    let mut x1 = full.right();
    let tool_r = show(vis.tool).then(|| {
        let r = Rect::from_min_max(
            pos2(x0, full.top()),
            pos2(x0 + m::TOOL_COLUMN, full.bottom()),
        );
        x0 = r.right() + m::SPLITTER;
        r
    });
    let left_r = show(vis.left_dock).then(|| {
        let r = Rect::from_min_max(pos2(x0, full.top()), pos2(x0 + m::LEFT_DOCK, full.bottom()));
        x0 = r.right() + m::SPLITTER;
        r
    });
    let right_r = show(vis.right_dock).then(|| {
        let r = Rect::from_min_max(
            pos2(x1 - m::RIGHT_DOCK, full.top()),
            pos2(x1, full.bottom()),
        );
        x1 = r.left() - m::SPLITTER;
        r
    });
    let icon_r = show(vis.icon_column).then(|| {
        let r = Rect::from_min_max(
            pos2(x1 - m::ICON_COLUMN, full.top()),
            pos2(x1, full.bottom()),
        );
        x1 = r.left() - m::SPLITTER;
        r
    });
    widgets::fill(ui, full, c.splitter);

    if let Some(r) = tool_r {
        left::tool_column(ui, c, &mut app.ui, r);
    }
    if let Some(r) = left_r {
        let header = Rect::from_min_size(r.min, vec2(r.width(), m::DOCK_HEADER));
        widgets::fill(ui, header, c.header);
        let parts = stack(
            header.bottom(),
            r.bottom(),
            &[m::SUB_TOOL_H, m::TOOL_PROPERTY_H, m::BRUSH_SIZE_H],
            3.0,
        );
        // The colour group keeps its height; Brush size gives way when the window is short.
        let colour_top = (r.bottom() - m::COLOUR_H).max(parts[2].0 + 60.0);
        let rect = |a: f32, b: f32| Rect::from_min_max(pos2(r.left(), a), pos2(r.right(), b));
        left::sub_tool(ui, c, &mut app.ui, rect(parts[0].0, parts[0].1));
        left::tool_property(ui, c, &mut app.ui, rect(parts[1].0, parts[1].1));
        left::brush_size(ui, c, &mut app.ui, rect(parts[2].0, colour_top - 3.0));
        colour::palette(ui, c, &mut app.ui.colour, rect(colour_top, r.bottom()));
    }
    if let Some(r) = right_r {
        let header = Rect::from_min_size(r.min, vec2(r.width(), m::DOCK_HEADER));
        widgets::fill(ui, header, c.header);
        let parts = stack(
            header.bottom(),
            r.bottom(),
            &[m::NAVIGATOR_H, m::LAYER_PROPERTY_H],
            3.0,
        );
        let rect = |a: f32, b: f32| Rect::from_min_max(pos2(r.left(), a), pos2(r.right(), b));
        right::navigator_group(app, ui, rect(parts[0].0, parts[0].1));
        right::layer_property_group(app, ui, rect(parts[1].0, parts[1].1));
        right::layer_group(app, ui, rect(parts[2].0, parts[2].1));
    }
    if let Some(r) = icon_r {
        chrome::icon_column(ui, c, r);
    }

    // Canvas column: command bar, document tabs, canvas, status strip, timeline dock.
    let col = Rect::from_min_max(pos2(x0, full.top()), pos2(x1, full.bottom()));
    let mut y = col.top();
    if show(vis.command_bar) {
        let r = Rect::from_min_size(pos2(col.left(), y), vec2(col.width(), m::COMMAND_BAR));
        chrome::command_bar(app, ui, r);
        y = r.bottom() + 1.0;
    }
    let tabs = Rect::from_min_size(pos2(col.left(), y), vec2(col.width(), m::DOC_TABS));
    chrome::doc_tabs(app, ui, tabs);
    let timeline = Rect::from_min_max(pos2(col.left(), col.bottom() - m::TIMELINE), col.max);
    let status = Rect::from_min_max(
        pos2(col.left(), timeline.top() - m::STATUS),
        pos2(col.right(), timeline.top()),
    );
    let sb = if vis.scroll_bars { 12.0 } else { 0.0 };
    let canvas = Rect::from_min_max(
        pos2(col.left(), tabs.bottom() + 1.0),
        pos2(col.right() - sb, status.top()),
    );
    widgets::area(ui, canvas, |ui| app.canvas_ui(ui));
    if vis.scroll_bars {
        chrome::v_scroll(
            app,
            ui,
            Rect::from_min_max(
                pos2(canvas.right(), canvas.top()),
                pos2(col.right(), canvas.bottom()),
            ),
        );
    }
    chrome::status_strip(app, ui, status);
    chrome::timeline_dock(ui, c, timeline);

    left::dynamics_popup(&ctx, &mut app.ui);
    left::detail_window(&ctx, &mut app.ui);
    dialogs::dialogs(app, &ctx);
    app.save_subtools_if_idle(&ctx);
}
