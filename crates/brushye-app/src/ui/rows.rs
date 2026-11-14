//! Brush setting rows, shared by the Sub Tool Detail window and the Tool property palette
//! (spec U-010). Each row has a category, a nesting level, a help text and a control type.
//! Settings Brushye doesn't have yet are listed and disabled, never hidden (U-010).

use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::brush::{Aa, Gap, InkMode, TipAxis};
use eframe::egui::{self, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

use super::theme::Colors;
use super::widgets;
use crate::icons;

/// Categories of the Sub Tool Detail list, in CSP's order: (name, available, description).
pub const CATEGORIES: [(&str, bool, &str); 19] = [
    (
        "Brush Size",
        true,
        "Size of the brush tip, and how pen pressure changes it.",
    ),
    (
        "Ink",
        true,
        "Opacity and how the ink combines with the layer.",
    ),
    (
        "Color Jitter",
        false,
        "Random variation of the drawing colour.",
    ),
    ("Anti-aliasing", true, "How soft the edge of the line is."),
    (
        "Brush shape",
        false,
        "Preset brush shapes (tip, spray, stroke and texture together).",
    ),
    (
        "  Brush tip",
        true,
        "Shape of the tip: hardness, thickness, angle and density.",
    ),
    (
        "Spraying effect",
        false,
        "Scatters particles around the stroke.",
    ),
    ("Stroke", true, "Spacing of the dabs along the stroke."),
    ("Texture", false, "Paper texture applied to the ink."),
    ("2 - Brush shape", false, "A second brush (dual brush)."),
    ("  2 - Brush tip", false, "Tip of the second brush."),
    (
        "2 - Spraying effect",
        false,
        "Spraying for the second brush.",
    ),
    ("2 - Stroke", false, "Spacing for the second brush."),
    ("2 - Texture", false, "Texture for the second brush."),
    (
        "Watercolor edge",
        false,
        "Darkens the edges of the stroke like watercolour.",
    ),
    ("Erase", false, "Vector erasing options."),
    ("Correction", true, "Stabilization and line correction."),
    (
        "Starting and ending",
        false,
        "Tapering at the start and end of strokes.",
    ),
    (
        "Anti-overflow",
        false,
        "Keeps the ink inside reference lines.",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Row {
    BrushSize,
    SizeOnScreen,
    AtLeast1px,
    Opacity,
    BlendMode,
    ColorMixing,
    BlurIntensity,
    AntiAliasing,
    TipShape,
    Hardness,
    Thickness,
    Direction,
    Angle,
    FlipH,
    FlipV,
    Density,
    DensityByGap,
    Gap,
    GapValue,
    ContinuousSpray,
    CorrectVelocity,
    Ribbon,
    BlendDarken,
    SharpAngles,
    Stabilization,
    AdjustBySpeed,
    PostCorrection,
    Taper,
    Snapping,
    VectorMagnet,
}

pub struct RowInfo {
    pub row: Row,
    pub key: &'static str,
    pub label: &'static str,
    pub category: usize,
    pub nested: u8,
    pub enabled: bool,
    pub help: &'static str,
}

const fn r(
    row: Row,
    key: &'static str,
    label: &'static str,
    category: usize,
    nested: u8,
    enabled: bool,
    help: &'static str,
) -> RowInfo {
    RowInfo {
        row,
        key,
        label,
        category,
        nested,
        enabled,
        help,
    }
}

/// All rows, in page order.
pub const ROWS: &[RowInfo] = &[
    r(
        Row::BrushSize,
        "size",
        "Brush Size",
        0,
        0,
        true,
        "Diameter of the brush in pixels. The dynamics button links it to pen pressure.",
    ),
    r(
        Row::SizeOnScreen,
        "size_on_screen",
        "Specify by size on screen",
        0,
        1,
        false,
        "Keep the size constant on screen whatever the zoom.",
    ),
    r(
        Row::AtLeast1px,
        "at_least_1px",
        "At least 1 pixel",
        0,
        1,
        false,
        "Never draw thinner than one pixel.",
    ),
    r(
        Row::Opacity,
        "opacity",
        "Opacity",
        1,
        0,
        true,
        "Highest opacity of the stroke. Overlapping dabs never exceed it.",
    ),
    r(
        Row::BlendMode,
        "blend",
        "Blending mode",
        1,
        0,
        true,
        "How the ink combines with the pixels already on the layer.",
    ),
    r(
        Row::ColorMixing,
        "color_mixing",
        "Color mixing",
        1,
        0,
        false,
        "Mix the ink with the colours on the canvas.",
    ),
    r(
        Row::BlurIntensity,
        "blur",
        "Intensity of blur",
        1,
        0,
        false,
        "Strength of blurring for blend tools.",
    ),
    r(
        Row::AntiAliasing,
        "aa",
        "Anti-aliasing",
        3,
        0,
        true,
        "Softness of the line's edge: none, weak, middle or strong.",
    ),
    r(
        Row::TipShape,
        "tip_shape",
        "Tip shape",
        5,
        0,
        true,
        "A round tip, or an image (material) tip.",
    ),
    r(
        Row::Hardness,
        "hardness",
        "Hardness",
        5,
        0,
        true,
        "How sharp the edge of a round tip is, in five steps.",
    ),
    r(
        Row::Thickness,
        "thickness",
        "Thickness",
        5,
        0,
        true,
        "Flattens the tip into an ellipse.",
    ),
    r(
        Row::Direction,
        "direction",
        "Direction",
        5,
        1,
        true,
        "Which axis thickness shortens.",
    ),
    r(
        Row::Angle,
        "angle",
        "Angle",
        5,
        0,
        true,
        "Rotation of the tip. Its dynamics can make it follow the line.",
    ),
    r(
        Row::FlipH,
        "flip_h",
        "Flip horizontal",
        5,
        0,
        false,
        "Mirror the tip horizontally.",
    ),
    r(
        Row::FlipV,
        "flip_v",
        "Flip vertical",
        5,
        0,
        false,
        "Mirror the tip vertically.",
    ),
    r(
        Row::Density,
        "density",
        "Brush density",
        5,
        0,
        true,
        "Opacity of each dab. Dabs build up along the stroke.",
    ),
    r(
        Row::DensityByGap,
        "density_by_gap",
        "Adjust brush density by gap",
        5,
        1,
        true,
        "Lower the density when dabs are close together, so the gap doesn't change the look.",
    ),
    r(
        Row::Gap,
        "gap",
        "Gap",
        7,
        0,
        true,
        "Distance between dabs: fixed, wide, normal or narrow.",
    ),
    r(
        Row::GapValue,
        "gap_value",
        "Gap value",
        7,
        1,
        true,
        "The fixed gap, in percent of the brush size.",
    ),
    r(
        Row::ContinuousSpray,
        "continuous_spray",
        "Continuous spraying",
        7,
        0,
        false,
        "Keep placing dabs while the pen rests.",
    ),
    r(
        Row::CorrectVelocity,
        "correct_velocity",
        "Correct velocity input",
        7,
        0,
        false,
        "Compensate for uneven input speed.",
    ),
    r(
        Row::Ribbon,
        "ribbon",
        "Ribbon",
        7,
        0,
        false,
        "Stretch the tip along the stroke.",
    ),
    r(
        Row::BlendDarken,
        "blend_darken",
        "Blend brush tips with Darken",
        7,
        0,
        false,
        "Combine overlapping dabs with Darken.",
    ),
    r(
        Row::SharpAngles,
        "sharp_angles",
        "Sharp angles",
        16,
        0,
        false,
        "Keep corners sharp.",
    ),
    r(
        Row::Stabilization,
        "stabilization",
        "Stabilization",
        16,
        0,
        true,
        "Smooths shaky lines; higher values follow the pen more slowly.",
    ),
    r(
        Row::AdjustBySpeed,
        "adjust_by_speed",
        "Adjust by speed",
        16,
        1,
        false,
        "Change stabilization with drawing speed.",
    ),
    r(
        Row::PostCorrection,
        "post_correction",
        "Post correction",
        16,
        0,
        false,
        "Smooth the line after it's drawn.",
    ),
    r(
        Row::Taper,
        "taper",
        "Taper",
        16,
        0,
        false,
        "Taper both ends of the line.",
    ),
    r(
        Row::Snapping,
        "snapping",
        "Enable snapping",
        16,
        0,
        false,
        "Snap to rulers.",
    ),
    r(
        Row::VectorMagnet,
        "vector_magnet",
        "Vector magnet",
        16,
        0,
        false,
        "Join vector lines that end close together.",
    ),
];

pub fn info(row: Row) -> &'static RowInfo {
    ROWS.iter().find(|i| i.row == row).unwrap_or(&ROWS[0])
}

pub fn by_key(key: &str) -> Option<&'static RowInfo> {
    ROWS.iter().find(|i| i.key == key)
}

/// Rows shown in Tool property by default (eye on), per tool kind.
pub fn default_shown(tool: super::tools::ToolId) -> Vec<String> {
    use super::tools::ToolId as T;
    let keys: &[&str] = match tool {
        T::Brush | T::Airbrush => &["size", "opacity", "hardness", "density", "stabilization"],
        T::Eraser => &["size", "aa", "hardness"],
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

/// Which dynamics a row's button opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynRow {
    Size,
    Opacity,
    Density,
    Angle,
}

/// What happened while drawing a row.
#[derive(Default)]
pub struct RowOut {
    pub changed: bool,
    pub dynamics: Option<DynRow>,
    pub hovered: bool,
}

fn pct(v: Option<u8>, d: u8) -> f64 {
    f64::from(v.unwrap_or(d))
}

/// A two-option toggle (a wide button pair, the selected half in the accent, U-010).
fn toggle2(ui: &mut Ui, c: &Colors, a: &str, b: &str, first: bool, enabled: bool) -> Option<bool> {
    let w = (ui.available_width() / 2.0).min(70.0);
    let mut out = None;
    for (i, label) in [a, b].iter().enumerate() {
        let on = (i == 0) == first;
        let (r, resp) = ui.allocate_exact_size(
            vec2(w, 19.0),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        ui.painter()
            .rect_filled(r, 0.0, if on { c.accent } else { c.button });
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(11.0),
            if enabled { c.text } else { c.text_disabled },
        );
        if resp.clicked() {
            out = Some(i == 0);
        }
    }
    out
}

/// A 5-step bar (Hardness etc., U-010): returns the clicked step 1..=5.
fn steps5(ui: &mut Ui, c: &Colors, value: u8, enabled: bool) -> Option<u8> {
    let w = (ui.available_width() / 5.0).min(22.0);
    let mut out = None;
    for i in 1..=5u8 {
        let (r, resp) = ui.allocate_exact_size(
            vec2(w - 1.0, 14.0),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let fill = if i <= value {
            if enabled { c.accent } else { c.slider_fill }
        } else {
            c.button
        };
        ui.painter().rect_filled(r, 0.0, fill);
        if resp.clicked() {
            out = Some(i);
        }
        ui.add_space(1.0);
    }
    out
}

/// Draw one row for `preset`.
pub fn draw(ui: &mut Ui, c: &Colors, row: Row, p: &mut BrushPreset) -> RowOut {
    let i = info(row);
    let mut out = RowOut::default();
    let top = ui.cursor().top();
    let indent = f32::from(i.nested) * 12.0;
    if indent > 0.0 {
        // A bracket line joining nested rows to their parent (U-010).
        let x = ui.cursor().left() + 6.0;
        ui.painter().line_segment(
            [pos2(x, top - 6.0), pos2(x, top + 13.0)],
            Stroke::new(1.0, c.splitter),
        );
        ui.painter().line_segment(
            [pos2(x, top + 13.0), pos2(x + 6.0, top + 13.0)],
            Stroke::new(1.0, c.splitter),
        );
    }
    let resp = ui
        .horizontal(|ui| {
            ui.add_space(indent);
            ui.vertical(|ui| draw_control(ui, c, i, p, &mut out))
                .response
        })
        .response;
    out.hovered = resp.hovered() || ui.rect_contains_pointer(resp.rect);
    out
}

fn draw_control(ui: &mut Ui, c: &Colors, i: &RowInfo, p: &mut BrushPreset, out: &mut RowOut) {
    let en = i.enabled;
    match i.row {
        Row::BrushSize => {
            let (ch, d) = widgets::prop_slider(
                ui,
                c,
                i.label,
                &mut p.size,
                0.5..=2000.0,
                true,
                1,
                Some(p.size_pressure.is_some()),
            );
            out.changed |= ch;
            if d.is_some_and(|d| d.clicked()) {
                out.dynamics = Some(DynRow::Size);
            }
        }
        Row::Opacity => {
            let mut v = pct(p.opacity, 100);
            let (ch, d) = widgets::prop_slider(
                ui,
                c,
                i.label,
                &mut v,
                1.0..=100.0,
                false,
                0,
                Some(p.opacity_pressure.is_some()),
            );
            if ch {
                p.opacity = (v != 100.0).then_some(v as u8);
                out.changed = true;
            }
            if d.is_some_and(|d| d.clicked()) {
                out.dynamics = Some(DynRow::Opacity);
            }
        }
        Row::Density => {
            let mut v = pct(p.density, 100);
            let (ch, d) = widgets::prop_slider(
                ui,
                c,
                i.label,
                &mut v,
                1.0..=100.0,
                false,
                0,
                Some(p.density_pressure.is_some()),
            );
            if ch {
                p.density = (v != 100.0).then_some(v as u8);
                out.changed = true;
            }
            if d.is_some_and(|d| d.clicked()) {
                out.dynamics = Some(DynRow::Density);
            }
        }
        Row::Thickness => {
            let mut v = pct(p.thickness, 100);
            if widgets::prop_slider(ui, c, i.label, &mut v, 1.0..=100.0, false, 0, None).0 {
                p.thickness = (v != 100.0).then_some(v as u8);
                out.changed = true;
            }
        }
        Row::Angle => {
            let mut v = p.angle.unwrap_or(0.0);
            let (ch, d) = widgets::prop_slider(
                ui,
                c,
                i.label,
                &mut v,
                0.0..=360.0,
                false,
                0,
                Some(p.angle_line == Some(true)),
            );
            if ch {
                p.angle = (v != 0.0).then_some(v);
                out.changed = true;
            }
            if d.is_some_and(|d| d.clicked()) {
                out.dynamics = Some(DynRow::Angle);
            }
        }
        Row::Stabilization => {
            let mut v = f64::from(p.stabilization.unwrap_or(0));
            if widgets::prop_slider(ui, c, i.label, &mut v, 0.0..=100.0, false, 0, None).0 {
                p.stabilization = (v != 0.0).then_some(v as u8);
                out.changed = true;
            }
        }
        Row::GapValue => {
            let fixed = matches!(p.gap, Some(Gap::Fixed(_)));
            let mut v = match p.gap {
                Some(Gap::Fixed(v)) => v,
                _ => 10.0,
            };
            ui.add_enabled_ui(fixed, |ui| {
                if widgets::prop_slider(ui, c, i.label, &mut v, 1.0..=1000.0, true, 1, None).0
                    && fixed
                {
                    p.gap = Some(Gap::Fixed(v));
                    out.changed = true;
                }
            });
        }
        Row::AntiAliasing => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                let cells = [
                    (icons::CIRCLE, "None"),
                    (icons::CIRCLE_DASHED, "Weak"),
                    (icons::CIRCLE_HALF, "Middle"),
                    (icons::CIRCLE_NOTCH, "Strong"),
                ];
                let order = [Aa::None, Aa::Weak, Aa::Middle, Aa::Strong];
                let cur = order
                    .iter()
                    .position(|&a| a == p.aa.unwrap_or_default())
                    .unwrap_or(2);
                if let Some(k) = widgets::segmented(ui, c, &cells, cur) {
                    p.aa = (order[k] != Aa::default()).then_some(order[k]);
                    out.changed = true;
                }
            });
        }
        Row::Gap => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                // CSP's cell order: fixed, wide, normal, narrow (B-002).
                let cells = [
                    (icons::RULER, "Fixed"),
                    (icons::DOTS_THREE, "Wide"),
                    (icons::DOTS_SIX, "Normal"),
                    (icons::DOTS_NINE, "Narrow"),
                ];
                let g = p.gap.unwrap_or_default();
                let cur = match g {
                    Gap::Fixed(_) => 0,
                    Gap::Wide => 1,
                    Gap::Normal => 2,
                    Gap::Narrow => 3,
                };
                if let Some(k) = widgets::segmented(ui, c, &cells, cur) {
                    let ng = [Gap::Fixed(10.0), Gap::Wide, Gap::Normal, Gap::Narrow][k];
                    if k != cur {
                        p.gap = (ng != Gap::Normal).then_some(ng);
                        out.changed = true;
                    }
                }
            });
        }
        Row::BlendMode => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                let ink = p.ink.unwrap_or_default();
                egui::ComboBox::from_id_salt(("ink", ui.next_auto_id()))
                    .width(ui.available_width() - 6.0)
                    .selected_text(ink.name())
                    .show_ui(ui, |ui| {
                        for &m in InkMode::ALL {
                            if ui.selectable_label(ink == m, m.name()).clicked() {
                                p.ink = (m != InkMode::Normal).then_some(m);
                                out.changed = true;
                            }
                        }
                    });
            });
        }
        Row::TipShape => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                toggle2(ui, c, "Circle", "Material", true, en);
            });
        }
        Row::Direction => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let horizontal = p.direction == Some(TipAxis::Horizontal);
                if let Some(first) = toggle2(ui, c, "Horizontal", "Vertical", horizontal, en) {
                    p.direction = first.then_some(TipAxis::Horizontal);
                    out.changed = true;
                }
            });
        }
        Row::Hardness => {
            widgets::prop_row(ui, c, i.label, en, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if let Some(v) = steps5(ui, c, p.hardness.unwrap_or(5), en) {
                    p.hardness = (v != 5).then_some(v);
                    out.changed = true;
                }
            });
        }
        Row::DensityByGap => {
            let mut v = p.density_by_gap.unwrap_or(false);
            if widgets::prop_check(ui, c, i.label, false, en, &mut v) {
                p.density_by_gap = v.then_some(true);
                out.changed = true;
            }
        }
        Row::CorrectVelocity => {
            // Brushye always behaves as CSP's default (on); B-002 found no effect off.
            widgets::prop_check(ui, c, i.label, false, false, &mut true);
        }
        Row::PostCorrection | Row::Taper | Row::VectorMagnet => {
            widgets::prop_check(ui, c, i.label, false, false, &mut false);
        }
        Row::AdjustBySpeed => {
            widgets::prop_check(ui, c, i.label, true, false, &mut false);
        }
        Row::SizeOnScreen
        | Row::AtLeast1px
        | Row::ColorMixing
        | Row::BlurIntensity
        | Row::FlipH
        | Row::FlipV
        | Row::ContinuousSpray
        | Row::Ribbon
        | Row::BlendDarken
        | Row::SharpAngles
        | Row::Snapping => {
            widgets::prop_check(ui, c, i.label, false, false, &mut false);
        }
    }
}

/// A small eye toggle and a ▸ button at the left of a Sub Tool Detail row (U-010).
pub fn eye_and_menu(ui: &mut Ui, c: &Colors, shown: bool, enabled: bool) -> (Response, Response) {
    let (er, eye) = ui.allocate_exact_size(
        vec2(14.0, 22.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if shown {
        ui.painter().rect_filled(er, 0.0, c.accent);
    }
    widgets::draw_icon(
        ui,
        er.center(),
        icons::EYE,
        10.0,
        if enabled { c.icon } else { c.icon_dim },
    );
    let (mr, menu) = ui.allocate_exact_size(vec2(14.0, 22.0), Sense::click());
    widgets::draw_icon(ui, mr.center(), icons::CARET_RIGHT, 9.0, c.icon);
    let eye = eye.on_hover_text("Show in Tool property");
    let _: Rect = mr;
    (eye, menu)
}
