//! Interface colours and metrics, measured from CSP 1.13.2 (spec U-009). Dark is the default;
//! Light is the alternative interface colour, as in CSP's preferences.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Interface {
    #[default]
    Dark,
    Light,
}

impl Interface {
    pub fn name(self) -> &'static str {
        match self {
            Interface::Dark => "dark",
            Interface::Light => "light",
        }
    }
    pub fn from_name(s: &str) -> Self {
        if s == "light" {
            Interface::Light
        } else {
            Interface::Dark
        }
    }
}

/// The measured colours (U-009 tables).
#[derive(Clone, Copy, Debug)]
pub struct Colors {
    /// Dock headers, palette title bars, tab strips, timeline dock.
    pub header: Color32,
    /// Menu bar, command bar, tabs, bottom bars, colour content, navigator controls, layer bars.
    pub bar: Color32,
    /// Lists and palette bodies.
    pub body: Color32,
    pub surround: Color32,
    pub status: Color32,
    pub splitter: Color32,
    pub row_separator: Color32,
    /// Active tab, toggles that are on, hovered menu row, active document tab.
    pub accent: Color32,
    /// Selected row, current tool, current preset.
    pub selection: Color32,
    pub name_plate: Color32,
    pub default_button: Color32,
    pub button: Color32,
    pub button_bevel: Color32,
    pub value_field: Color32,
    pub slider_fill: Color32,
    pub slider_top: Color32,
    pub slider_bottom: Color32,
    pub slider_base: Color32,
    pub scroll_track: Color32,
    pub scroll_thumb: Color32,
    pub tool_separator: Color32,
    pub icon: Color32,
    pub icon_dim: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    pub text_on_accent: Color32,
    pub checker_light: Color32,
    pub checker_dark: Color32,
}

pub const DARK: Colors = Colors {
    header: Color32::from_rgb(56, 56, 56),
    bar: Color32::from_rgb(78, 78, 78),
    body: Color32::from_rgb(63, 63, 63),
    surround: Color32::from_rgb(45, 45, 45),
    status: Color32::from_rgb(67, 67, 67),
    splitter: Color32::from_rgb(49, 49, 49),
    row_separator: Color32::from_rgb(34, 34, 34),
    accent: Color32::from_rgb(103, 113, 135),
    selection: Color32::from_rgb(96, 106, 128),
    name_plate: Color32::from_rgb(135, 143, 159),
    default_button: Color32::from_rgb(104, 121, 160),
    button: Color32::from_rgb(103, 103, 103),
    button_bevel: Color32::from_rgb(126, 126, 126),
    value_field: Color32::from_rgb(49, 49, 49),
    slider_fill: Color32::from_rgb(88, 88, 88),
    slider_top: Color32::from_rgb(111, 111, 111),
    slider_bottom: Color32::from_rgb(123, 123, 123),
    slider_base: Color32::from_rgb(99, 99, 99),
    scroll_track: Color32::from_rgb(90, 90, 90),
    scroll_thumb: Color32::from_rgb(67, 67, 67),
    tool_separator: Color32::from_rgb(114, 114, 114),
    icon: Color32::from_rgb(178, 178, 178),
    icon_dim: Color32::from_rgb(118, 118, 118),
    text: Color32::from_rgb(218, 218, 218),
    text_dim: Color32::from_rgb(146, 146, 146),
    text_disabled: Color32::from_rgb(150, 150, 150),
    text_on_accent: Color32::from_rgb(190, 192, 198),
    checker_light: Color32::from_rgb(255, 255, 255),
    checker_dark: Color32::from_rgb(204, 204, 204),
};

pub const LIGHT: Colors = Colors {
    header: Color32::from_rgb(224, 224, 224),
    bar: Color32::from_rgb(240, 240, 240),
    body: Color32::from_rgb(230, 230, 230),
    surround: Color32::from_rgb(199, 199, 199),
    status: Color32::from_rgb(239, 239, 239),
    splitter: Color32::from_rgb(184, 184, 184),
    row_separator: Color32::from_rgb(200, 200, 200),
    accent: Color32::from_rgb(184, 208, 229),
    selection: Color32::from_rgb(179, 203, 224),
    name_plate: Color32::from_rgb(205, 220, 235),
    default_button: Color32::from_rgb(104, 121, 160),
    button: Color32::from_rgb(255, 255, 255),
    button_bevel: Color32::from_rgb(210, 210, 210),
    value_field: Color32::from_rgb(255, 255, 255),
    slider_fill: Color32::from_rgb(200, 200, 200),
    slider_top: Color32::from_rgb(180, 180, 180),
    slider_bottom: Color32::from_rgb(170, 170, 170),
    slider_base: Color32::from_rgb(170, 170, 170),
    scroll_track: Color32::from_rgb(215, 215, 215),
    scroll_thumb: Color32::from_rgb(180, 180, 180),
    tool_separator: Color32::from_rgb(170, 170, 170),
    icon: Color32::from_rgb(40, 40, 40),
    icon_dim: Color32::from_rgb(150, 150, 150),
    text: Color32::from_rgb(30, 30, 30),
    text_dim: Color32::from_rgb(100, 100, 100),
    text_disabled: Color32::from_rgb(150, 150, 150),
    text_on_accent: Color32::from_rgb(20, 30, 45),
    checker_light: Color32::from_rgb(255, 255, 255),
    checker_dark: Color32::from_rgb(204, 204, 204),
};

/// Metrics (U-009), in points at 100 % UI scale.
pub mod m {
    pub const MENU_BAR: f32 = 24.0;
    pub const COMMAND_BAR: f32 = 24.0;
    pub const DOCK_HEADER: f32 = 21.0;
    pub const TITLE_BAR: f32 = 17.0;
    pub const BOTTOM_BAR: f32 = 20.0;
    pub const TOOL_COLUMN: f32 = 41.0;
    pub const LEFT_DOCK: f32 = 208.0;
    pub const RIGHT_DOCK: f32 = 227.0;
    pub const ICON_COLUMN: f32 = 40.0;
    pub const DOC_TABS: f32 = 19.0;
    pub const STATUS: f32 = 13.0;
    pub const TIMELINE: f32 = 38.0;
    pub const SPLITTER: f32 = 2.0;
    pub const SUB_TOOL_ROW: f32 = 31.0;
    pub const LAYER_ROW: f32 = 41.0;
    pub const BRUSH_CELL: [f32; 2] = [32.0, 40.0];
    pub const FONT: f32 = 12.0;
    /// Palette group heights at 1080 px (U-001).
    pub const SUB_TOOL_H: f32 = 289.0;
    pub const TOOL_PROPERTY_H: f32 = 243.0;
    pub const BRUSH_SIZE_H: f32 = 242.0;
    pub const COLOUR_H: f32 = 226.0;
    pub const NAVIGATOR_H: f32 = 239.0;
    pub const LAYER_PROPERTY_H: f32 = 239.0;
}

pub fn colors(i: Interface) -> &'static Colors {
    match i {
        Interface::Dark => &DARK,
        Interface::Light => &LIGHT,
    }
}

/// Apply the interface colours and metrics to egui's style.
pub fn apply(ctx: &egui::Context, i: Interface) {
    let c = colors(i);
    let mut v = match i {
        Interface::Dark => egui::Visuals::dark(),
        Interface::Light => egui::Visuals::light(),
    };
    let square = CornerRadius::ZERO;
    v.override_text_color = Some(c.text);
    v.panel_fill = c.body;
    v.window_fill = c.bar;
    v.extreme_bg_color = c.value_field;
    v.faint_bg_color = c.body;
    v.code_bg_color = c.value_field;
    v.window_corner_radius = square;
    v.menu_corner_radius = square;
    v.window_stroke = Stroke::new(1.0, c.splitter);
    v.window_shadow = egui::Shadow::NONE;
    v.popup_shadow = egui::Shadow::NONE;
    v.selection.bg_fill = c.selection;
    v.selection.stroke = Stroke::new(1.0, c.text);
    v.hyperlink_color = c.default_button;
    for (w, fill) in [
        (&mut v.widgets.noninteractive, c.bar),
        (&mut v.widgets.inactive, c.button),
        (&mut v.widgets.hovered, c.accent),
        (&mut v.widgets.active, c.selection),
        (&mut v.widgets.open, c.accent),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.corner_radius = square;
        w.bg_stroke = Stroke::NONE;
        w.fg_stroke = Stroke::new(1.0, c.text);
        w.expansion = 0.0;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.splitter);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text);
    v.widgets.inactive.weak_bg_fill = c.bar;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.text_styles = [
            (
                TextStyle::Small,
                FontId::new(10.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Body,
                FontId::new(m::FONT, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(m::FONT, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(13.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(m::FONT, FontFamily::Monospace),
            ),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(4.0, 2.0);
        s.spacing.button_padding = egui::vec2(4.0, 1.0);
        s.spacing.interact_size = egui::vec2(20.0, 19.0);
        s.spacing.menu_margin = egui::Margin::same(2);
        s.spacing.window_margin = egui::Margin::same(6);
        s.spacing.slider_width = 100.0;
        s.spacing.scroll.bar_width = 9.0;
        s.spacing.scroll.floating = false;
    });
}
