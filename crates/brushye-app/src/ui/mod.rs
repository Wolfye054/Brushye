//! The window layout, matching CSP 1.13.2's default workspace (specs U-001 … U-009): menu bar;
//! tool column; left dock (Sub Tool, Tool property, Brush size, colour group); canvas column
//! (command bar, document tabs, canvas, status strip, timeline dock); icon column; right dock
//! (Navigator group, Layer Property group, Layer group). Icons are Phosphor, not CSP's.

pub mod chrome;
pub mod colour;
pub mod dialogs;
pub mod journal;
pub mod left;
pub mod menus;
pub mod right;
pub mod rows;
pub mod theme;
pub mod tool_cmds;
pub mod tools;
pub mod widgets;

use std::collections::HashMap;

use eframe::egui::{self, Rect, TextureHandle, pos2, vec2};

use crate::app::App;
use theme::{Interface, m};
use tools::ToolId;

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

/// Palette display preferences from the palettes' ≡ menus (U-011, U-012), saved per user.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prefs {
    pub subtool_text: bool,
    pub show_group_names: bool,
    pub subtool_bar: bool,
    pub drag_needs_ctrl: bool,
    pub show_category: bool,
    pub show_preview: bool,
    pub tool_property_bar: bool,
    pub size_slider: bool,
    pub navigator_bar: bool,
    pub layer_property_bar: bool,
    pub layer_bar: bool,
    pub hls: bool,
}

impl Default for Prefs {
    /// CSP's defaults (U-011, U-012).
    fn default() -> Self {
        Self {
            subtool_text: false,
            show_group_names: true,
            subtool_bar: true,
            drag_needs_ctrl: false,
            show_category: false,
            show_preview: true,
            tool_property_bar: true,
            size_slider: false,
            navigator_bar: true,
            layer_property_bar: true,
            layer_bar: true,
            hls: false,
        }
    }
}

impl Prefs {
    const KEYS: [&'static str; 12] = [
        "subtool_text",
        "show_group_names",
        "subtool_bar",
        "drag_needs_ctrl",
        "show_category",
        "show_preview",
        "tool_property_bar",
        "size_slider",
        "navigator_bar",
        "layer_property_bar",
        "layer_bar",
        "hls",
    ];
    fn fields(&mut self) -> [&mut bool; 12] {
        [
            &mut self.subtool_text,
            &mut self.show_group_names,
            &mut self.subtool_bar,
            &mut self.drag_needs_ctrl,
            &mut self.show_category,
            &mut self.show_preview,
            &mut self.tool_property_bar,
            &mut self.size_slider,
            &mut self.navigator_bar,
            &mut self.layer_property_bar,
            &mut self.layer_bar,
            &mut self.hls,
        ]
    }
    pub fn to_json(mut self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        for (k, v) in Self::KEYS.iter().zip(self.fields()) {
            m.insert((*k).into(), serde_json::Value::Bool(*v));
        }
        serde_json::Value::Object(m)
    }
    pub fn from_json(j: &serde_json::Value) -> Self {
        let mut p = Self::default();
        for (k, v) in Self::KEYS.iter().zip(p.fields()) {
            if let Some(b) = j.get(*k).and_then(serde_json::Value::as_bool) {
                *v = b;
            }
        }
        p
    }
}

/// UI state that isn't part of the document.
pub struct UiState {
    pub prefs: Prefs,
    /// Palettes hidden from their ≡ menu ("sub_tool", "tool_property", …).
    pub hidden: std::collections::HashSet<&'static str>,
    pub brush_sizes: Vec<f64>,
    pub detail_category: usize,
    pub detail_hover: Option<rows::Row>,
    pub subtool_dialog: Option<left::SubToolDialog>,
    pub interface: Interface,
    pub applied_interface: Option<Interface>,
    /// A mirror of the session's tools, sub tools and colour chips (D-018), refreshed when
    /// they change. The UI reads it and changes it only by queuing commands in `cmds`.
    pub tools: tools::ToolState,
    /// Commands queued by the UI this frame; the app runs them after the UI pass.
    pub cmds: Vec<brushye_command::commands::Command>,
    /// Palette preferences or brush size presets changed: save the settings.
    pub prefs_dirty: bool,
    pub colour: colour::ColourState,
    pub previews: HashMap<String, TextureHandle>,
    pub detail_open: bool,
    pub dynamics_popup: Option<rows::DynRow>,
    /// Where the dynamics button was clicked; the pop-up opens next to it (U-010).
    pub dynamics_anchor: egui::Pos2,
    pub pending_subtool_op: Option<left::SubToolOp>,
    pub import_brush: Option<String>,
    /// After a .sut import: each sub tool's name and what was approximated or left out.
    pub sut_report: Option<Vec<(String, Vec<String>)>>,
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
    /// Window > Journal (Brushye), M9.
    pub journal: journal::JournalWindow,
    /// Layer palette drag: where the dragged layer would land if released now (U-015).
    pub layer_drop: Option<(brushye_command::core::LayerId, crate::app::MoveTo)>,
}

impl UiState {
    pub fn new(settings: &crate::settings::Settings) -> Self {
        let ui = serde_json::Value::Object(settings.ui.clone());
        let brush_sizes = ui["brush_sizes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(serde_json::Value::as_f64)
                    .filter(|v| *v > 0.0)
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| tools::BRUSH_SIZES.to_vec());
        Self {
            prefs: Prefs::from_json(&ui["prefs"]),
            hidden: Default::default(),
            brush_sizes,
            detail_category: 0,
            detail_hover: None,
            subtool_dialog: None,
            interface: Interface::from_name(&settings.interface),
            applied_interface: None,
            tools: tools::ToolState::with_saved(&settings.subtools),
            cmds: Vec::new(),
            prefs_dirty: false,
            colour: colour::ColourState::default(),
            previews: HashMap::new(),
            detail_open: false,
            dynamics_popup: None,
            dynamics_anchor: egui::Pos2::ZERO,
            pending_subtool_op: None,
            import_brush: None,
            sut_report: None,
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
            journal: Default::default(),
            layer_drop: None,
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
        self.tools.current(tool).map(|s| &s.preset)
    }

    /// Queue a command (run by the app after the UI pass).
    pub fn cmd(&mut self, c: brushye_command::commands::Command) {
        self.cmds.push(c);
    }

    /// Queue selecting a tool.
    pub fn select_tool(&mut self, tool: ToolId) {
        self.cmd(brushye_command::commands::Command::ToolSelect(
            brushye_command::commands::ToolSelect { tool },
        ));
    }

    /// The UI preferences saved with the user settings.
    pub fn ui_json(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut m = serde_json::Map::new();
        m.insert("prefs".into(), self.prefs.to_json());
        m.insert("brush_sizes".into(), serde_json::json!(self.brush_sizes));
        m
    }

    /// Workspace > Reset to Default: show everything in its default place.
    pub fn reset_layout(&mut self) {
        self.visible = Visible::default();
        self.hidden.clear();
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
        // Visible palettes in CSP's order; the colour group keeps its height and Brush size takes
        // what's left (a hidden palette's space goes to the others).
        let groups: Vec<(&str, f32)> = [
            ("sub_tool", m::SUB_TOOL_H),
            ("tool_property", m::TOOL_PROPERTY_H),
            ("brush_size", m::BRUSH_SIZE_H),
        ]
        .into_iter()
        .filter(|(k, _)| !app.ui.hidden.contains(k))
        .collect();
        let colour_shown = !app.ui.hidden.contains("colour");
        let fixed: Vec<f32> = groups
            .iter()
            .take(groups.len().saturating_sub(1))
            .map(|g| g.1)
            .collect();
        let bottom = if colour_shown {
            (r.bottom() - m::COLOUR_H).max(header.bottom() + 120.0)
        } else {
            r.bottom()
        };
        let parts = stack(
            header.bottom(),
            bottom - if colour_shown { 3.0 } else { 0.0 },
            &fixed,
            3.0,
        );
        let rect = |a: f32, b: f32| Rect::from_min_max(pos2(r.left(), a), pos2(r.right(), b));
        for ((key, _), (a, b)) in groups.iter().zip(parts) {
            match *key {
                "sub_tool" => left::sub_tool(ui, c, &mut app.ui, rect(a, b)),
                "tool_property" => left::tool_property(ui, c, &mut app.ui, rect(a, b)),
                _ => left::brush_size(ui, c, &mut app.ui, rect(a, b)),
            }
        }
        if colour_shown {
            colour::palette(ui, c, &mut app.ui, rect(bottom, r.bottom()));
        }
    }
    if let Some(r) = right_r {
        let header = Rect::from_min_size(r.min, vec2(r.width(), m::DOCK_HEADER));
        widgets::fill(ui, header, c.header);
        let groups: Vec<(&str, f32)> = [
            ("navigator", m::NAVIGATOR_H),
            ("layer_property", m::LAYER_PROPERTY_H),
            ("layer", 0.0),
        ]
        .into_iter()
        .filter(|(k, _)| !app.ui.hidden.contains(k))
        .collect();
        let fixed: Vec<f32> = groups
            .iter()
            .take(groups.len().saturating_sub(1))
            .map(|g| g.1)
            .collect();
        let parts = stack(header.bottom(), r.bottom(), &fixed, 3.0);
        let rect = |a: f32, b: f32| Rect::from_min_max(pos2(r.left(), a), pos2(r.right(), b));
        for ((key, _), (a, b)) in groups.iter().zip(parts) {
            match *key {
                "navigator" => right::navigator_group(app, ui, rect(a, b)),
                "layer_property" => right::layer_property_group(app, ui, rect(a, b)),
                _ => right::layer_group(app, ui, rect(a, b)),
            }
        }
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
    if let Some(s) = left::sub_tool_dialogs(&ctx, &mut app.ui) {
        app.status = s;
    }
    left::detail_window(&ctx, &mut app.ui);
    dialogs::dialogs(app, &ctx);
    app.run_ui_commands();
    app.save_subtools_if_idle(&ctx);
}
