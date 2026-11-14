//! Left side: tool column (U-004), and the left dock's Sub Tool, Tool property and Brush size
//! palettes, plus the Sub Tool Detail window.

use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::brush::{self, Aa, BrushSettings, Curve, Gap, InkMode, TipAxis};
use brushye_command::core::input::Sample;
use brushye_command::core::{Layer, LayerId, LayerKind};
use eframe::egui::{self, Color32, ColorImage, Rect, Sense, Stroke, TextureHandle, Ui, pos2, vec2};

use super::UiState;
use super::theme::{Colors, m};
use super::tools::{self, SubTool, TOOLS, ToolId};
use super::widgets;
use crate::icons;

// ---------------------------------------------------------------- tool column

pub fn tool_column(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    widgets::fill(ui, r, c.body);
    let header = Rect::from_min_size(r.min, vec2(r.width(), m::DOCK_HEADER));
    widgets::fill(ui, header, c.header);
    // A small title bar ("≡" and the tool icon) above the tools (U-001/U-004).
    let title_r = Rect::from_min_size(
        pos2(r.left(), header.bottom()),
        vec2(r.width(), m::TITLE_BAR),
    );
    widgets::fill(ui, title_r, c.header);
    widgets::draw_icon(
        ui,
        pos2(title_r.left() + 7.0, title_r.center().y),
        icons::LIST,
        11.0,
        c.icon,
    );
    widgets::draw_icon(
        ui,
        pos2(title_r.left() + 26.0, title_r.center().y),
        tools::info(st.tool).icon,
        12.0,
        c.icon,
    );
    let mut y = title_r.bottom() + 15.0;
    for t in TOOLS {
        if t.separator_before {
            ui.painter().line_segment(
                [
                    pos2(r.left() + 6.0, y + 1.0),
                    pos2(r.right() - 6.0, y + 1.0),
                ],
                Stroke::new(1.0, c.tool_separator),
            );
            y += 3.0;
        }
        let br = Rect::from_min_size(pos2(r.center().x - 12.0, y), vec2(24.0, 22.0));
        let tip = format!("{} ({})", t.name, t.shortcut);
        let resp = widgets::area(ui, br, |ui| {
            widgets::icon_button(
                ui,
                c,
                t.icon,
                &tip,
                vec2(24.0, 22.0),
                t.available,
                st.tool == t.id,
            )
        });
        if resp.clicked() {
            st.tool = t.id;
        }
        y += 25.0;
        if y > r.bottom() - 70.0 {
            break;
        }
    }
    super::colour::chips(ui, c, &mut st.colour, pos2(r.left() + 6.0, y + 12.0), 18.0);
}

// ---------------------------------------------------------------- stroke previews

/// A preview of a preset: an S-shaped stroke with a pressure swell, white on transparent,
/// rendered by the real brush engine (shown in sub tool rows and the Tool property strip).
pub fn stroke_preview(preset: &BrushPreset, w: u32, h: u32) -> ColorImage {
    let mut p = preset.clone();
    p.ink = None; // erasers preview as ink
    p.size = p.size.min(f64::from(h) * 0.55).max(1.0);
    p.stabilization = None;
    let settings: BrushSettings = p.settings([255, 255, 255]).unwrap_or_default();
    let n = 48;
    let samples: Vec<Sample> = (0..=n)
        .map(|i| {
            let t = f64::from(i) / f64::from(n);
            Sample {
                x: 6.0 + t * (f64::from(w) - 12.0),
                y: f64::from(h) / 2.0 + (t * std::f64::consts::TAU).sin() * f64::from(h) * 0.18,
                t_us: i * 8000,
                pressure: Some((t * std::f64::consts::PI).sin().max(0.05) as f32),
                tilt: None,
                twist: None,
                eraser: false,
                barrel: [false; 2],
            }
        })
        .collect();
    let mut layer = Layer::new(LayerId(1), "preview", LayerKind::Raster);
    let _ = brush::stroke(&mut layer, (w, h), &settings, &samples);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            px.extend_from_slice(&layer.pixel(x, y));
        }
    }
    ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &px)
}

fn preview_texture(
    ui: &Ui,
    st: &mut UiState,
    preset: &BrushPreset,
    w: u32,
    h: u32,
) -> TextureHandle {
    let key = format!("{w}x{h} {}", preset.to_text());
    if let Some(t) = st.previews.get(&key) {
        return t.clone();
    }
    let tex = ui.ctx().load_texture(
        key.clone(),
        stroke_preview(preset, w, h),
        egui::TextureOptions::LINEAR,
    );
    if st.previews.len() > 256 {
        st.previews.clear();
    }
    st.previews.insert(key, tex.clone());
    tex
}

// ---------------------------------------------------------------- sub tool palette

pub fn sub_tool(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tool;
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    widgets::title(
        ui,
        c,
        title_r,
        icons::PAINT_BRUSH,
        &format!("Sub Tool: {}", tools::info(tool).name),
    );
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let Some(mut set) = st.sets.get(&tool).cloned() else {
        ui.painter().text(
            body.center(),
            egui::Align2::CENTER_CENTER,
            "No sub tools",
            egui::FontId::proportional(11.0),
            c.text_dim,
        );
        return;
    };
    // Group tabs, two per row (U-004).
    let mut y = body.top() + 5.0;
    let tab_w = (body.width() - 8.0) / 2.0;
    for (i, g) in set.groups.iter().enumerate() {
        let tr = Rect::from_min_size(
            pos2(
                body.left() + 3.0 + (i % 2) as f32 * (tab_w + 2.0),
                y + (i / 2) as f32 * 28.0,
            ),
            vec2(tab_w, 24.0),
        );
        let resp = ui.interact(tr, ui.scope_id().with(("group", i)), Sense::click());
        ui.painter()
            .rect_filled(tr, 2.0, if set.group == i { c.accent } else { c.bar });
        widgets::draw_icon(
            ui,
            pos2(tr.left() + 12.0, tr.center().y),
            g.icon,
            13.0,
            c.icon,
        );
        ui.painter().text(
            pos2(tr.left() + 24.0, tr.center().y),
            egui::Align2::LEFT_CENTER,
            &g.name,
            egui::FontId::proportional(11.0),
            c.text,
        );
        if resp.clicked() {
            set.group = i;
        }
    }
    y += set.groups.len().div_ceil(2) as f32 * 28.0 + 1.0;
    let list = Rect::from_min_max(
        pos2(body.left(), y),
        pos2(body.right(), body.bottom() - m::BOTTOM_BAR - 1.0),
    );
    let bottom = Rect::from_min_max(pos2(body.left(), list.bottom() + 1.0), body.max);
    let group = set.group;
    let items: Vec<SubTool> = set.groups[group].items.clone();
    let selected = set.groups[group].selected;
    widgets::area(ui, list, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("subtools")
            .show(ui, |ui| {
                for (i, item) in items.iter().enumerate() {
                    let w = ui.available_width();
                    let (rr, resp) =
                        ui.allocate_exact_size(vec2(w, m::SUB_TOOL_ROW), Sense::click());
                    if i == selected {
                        widgets::fill(ui, rr, c.selection);
                    }
                    let tex = preview_texture(ui, st, &item.preset, (w as u32).max(40), 30);
                    ui.painter().image(
                        tex.id(),
                        Rect::from_min_size(rr.min, vec2(w, 30.0)),
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    let font = egui::FontId::proportional(11.0);
                    let text_pos = pos2(rr.right() - 6.0, rr.bottom() - 9.0);
                    if i == selected {
                        let tw = ui.fonts_mut(|f| {
                            f.layout_no_wrap(item.name.clone(), font.clone(), c.text)
                                .size()
                                .x
                        });
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                pos2(text_pos.x - tw - 4.0, text_pos.y - 7.0),
                                pos2(rr.right() - 2.0, text_pos.y + 7.0),
                            ),
                            0.0,
                            c.name_plate,
                        );
                    }
                    ui.painter().text(
                        text_pos,
                        egui::Align2::RIGHT_CENTER,
                        &item.name,
                        font,
                        c.text,
                    );
                    ui.painter().line_segment(
                        [
                            pos2(rr.left(), rr.bottom() - 0.5),
                            pos2(rr.right(), rr.bottom() - 0.5),
                        ],
                        Stroke::new(1.0, c.row_separator),
                    );
                    if resp.clicked() {
                        set.groups[group].selected = i;
                    }
                    resp.context_menu(|ui| {
                        if ui.button("Duplicate sub tool").clicked() {
                            st.pending_subtool_op = Some(SubToolOp::Copy(i));
                            ui.close();
                        }
                        if ui.button("Delete sub tool").clicked() {
                            st.pending_subtool_op = Some(SubToolOp::Delete(i));
                            ui.close();
                        }
                    });
                }
            });
    });
    // Bottom bar: add, copy, delete (U-004).
    widgets::fill(ui, bottom, c.bar);
    widgets::area(ui, bottom, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if widgets::icon_button(
                ui,
                c,
                icons::TRASH,
                "Delete selected sub tool",
                vec2(20.0, 18.0),
                items.len() > 1,
                false,
            )
            .clicked()
            {
                st.pending_subtool_op = Some(SubToolOp::Delete(selected));
            }
            if widgets::icon_button(
                ui,
                c,
                icons::COPY,
                "Duplicate selected sub tool",
                vec2(20.0, 18.0),
                true,
                false,
            )
            .clicked()
            {
                st.pending_subtool_op = Some(SubToolOp::Copy(selected));
            }
            if widgets::icon_button(
                ui,
                c,
                icons::DOWNLOAD_SIMPLE,
                "Import a sub tool (.brshbrush)",
                vec2(20.0, 18.0),
                true,
                false,
            )
            .clicked()
            {
                st.import_brush = Some(String::new());
            }
        });
    });
    if let Some(op) = st.pending_subtool_op.take() {
        let g = &mut set.groups[group];
        match op {
            SubToolOp::Copy(i) => {
                if let Some(s) = g.items.get(i).cloned() {
                    let mut s2 = s;
                    s2.name = format!("{} copy", s2.name);
                    g.items.insert(i + 1, s2);
                    g.selected = i + 1;
                }
            }
            SubToolOp::Delete(i) => {
                if g.items.len() > 1 && i < g.items.len() {
                    g.items.remove(i);
                    g.selected = g.selected.min(g.items.len() - 1);
                }
            }
        }
    }
    if st.sets.get(&tool) != Some(&set) {
        st.sets.insert(tool, set);
        st.sets_dirty = true;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SubToolOp {
    Copy(usize),
    Delete(usize),
}

// ---------------------------------------------------------------- tool property

fn pct_value(v: Option<u8>, default: u8) -> f64 {
    f64::from(v.unwrap_or(default))
}

pub fn tool_property(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tool;
    let name = st
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.name.clone())
        .unwrap_or_else(|| tools::info(tool).name.to_owned());
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    widgets::title(
        ui,
        c,
        title_r,
        icons::WRENCH,
        &format!("Tool property: {name}"),
    );
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let bottom = Rect::from_min_max(pos2(body.left(), body.bottom() - m::BOTTOM_BAR), body.max);
    let Some(mut sub) = st.sets.get(&tool).and_then(|s| s.current()).cloned() else {
        tool_property_other(
            ui,
            c,
            st,
            Rect::from_min_max(body.min, pos2(body.right(), bottom.top())),
        );
        widgets::fill(ui, bottom, c.bar);
        return;
    };
    // Preview strip (44 px): name label, the brush's stroke, lock toggle (U-004).
    let strip = Rect::from_min_size(body.min, vec2(body.width(), 44.0));
    widgets::fill(ui, strip, c.bar);
    let tex = preview_texture(ui, st, &sub.preset, strip.width() as u32, 44);
    ui.painter().image(
        tex.id(),
        strip,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.painter().text(
        strip.min + vec2(3.0, 8.0),
        egui::Align2::LEFT_CENTER,
        &sub.name,
        egui::FontId::proportional(11.0),
        c.text,
    );
    widgets::area(
        ui,
        Rect::from_min_size(
            pos2(strip.right() - 20.0, strip.bottom() - 18.0),
            vec2(18.0, 16.0),
        ),
        |ui| {
            widgets::icon_button(
                ui,
                c,
                icons::LOCK_OPEN,
                "Lock sub tool settings",
                vec2(18.0, 16.0),
                false,
                false,
            );
        },
    );
    let rows = Rect::from_min_max(
        pos2(body.left(), strip.bottom() + 2.0),
        pos2(body.right(), bottom.top()),
    );
    let mut p = sub.preset.clone();
    widgets::area(ui, rows, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let (ch, dynr) = widgets::prop_slider(
            ui,
            c,
            "Brush Size",
            &mut p.size,
            0.5..=2000.0,
            true,
            1,
            Some(p.size_pressure.is_some()),
        );
        let _ = ch;
        if let Some(d) = dynr
            && d.clicked()
        {
            st.dynamics_popup = Some(DynTarget::Size);
        }
        let mut op = pct_value(p.opacity, 100);
        let (ch, dynr) = widgets::prop_slider(
            ui,
            c,
            "Opacity",
            &mut op,
            1.0..=100.0,
            false,
            0,
            Some(p.opacity_pressure.is_some()),
        );
        if ch {
            p.opacity = (op != 100.0).then_some(op as u8);
        }
        if let Some(d) = dynr
            && d.clicked()
        {
            st.dynamics_popup = Some(DynTarget::Opacity);
        }
        widgets::prop_row(ui, c, "Anti-aliasing", true, |ui| {
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
            if let Some(i) = widgets::segmented(ui, c, &cells, cur) {
                p.aa = (order[i] != Aa::default()).then_some(order[i]);
            }
        });
        let mut stab = f64::from(p.stabilization.unwrap_or(0));
        if widgets::prop_slider(
            ui,
            c,
            "Stabilization",
            &mut stab,
            0.0..=100.0,
            false,
            0,
            None,
        )
        .0
        {
            p.stabilization = (stab != 0.0).then_some(stab as u8);
        }
        widgets::prop_check(ui, c, "Adjust by speed", true, false, &mut false);
        widgets::prop_check(ui, c, "Vector magnet", false, false, &mut false);
    });
    widgets::fill(ui, bottom, c.bar);
    widgets::area(ui, bottom, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            if widgets::icon_button(
                ui,
                c,
                icons::WRENCH,
                "Show Sub Tool Detail and edit all settings",
                vec2(20.0, 18.0),
                true,
                st.detail_open,
            )
            .clicked()
            {
                st.detail_open = !st.detail_open;
            }
            if widgets::icon_button(
                ui,
                c,
                icons::ARROW_COUNTER_CLOCKWISE,
                "Reset sub tool to its default settings",
                vec2(20.0, 18.0),
                true,
                false,
            )
            .clicked()
                && let Some(d) = tools::default_set(tool).and_then(|d| {
                    d.groups
                        .iter()
                        .flat_map(|g| g.items.iter())
                        .find(|s| s.name == sub.name)
                        .cloned()
                })
            {
                p = d.preset;
            }
        });
    });
    if p != sub.preset {
        sub.preset = p;
        if let Some(s) = st.sets.get_mut(&tool).and_then(|s| s.current_mut()) {
            *s = sub;
            st.sets_dirty = true;
        }
    }
}

/// Tool property for tools without sub tools.
fn tool_property_other(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    widgets::area(ui, r, |ui| {
        ui.add_space(6.0);
        match st.tool {
            ToolId::Zoom => {
                ui.label("  Click to zoom in, Alt+click to zoom out.");
            }
            ToolId::Hand => {
                ui.label("  Drag to scroll the canvas.");
            }
            ToolId::Eyedropper => {
                widgets::prop_row(ui, c, "Acquire from", true, |ui| {
                    ui.selectable_value(&mut st.eyedropper_layer, false, "Canvas");
                    ui.selectable_value(&mut st.eyedropper_layer, true, "Layer");
                });
            }
            _ => {
                ui.colored_label(c.text_disabled, "  Not available yet");
            }
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynTarget {
    Size,
    Opacity,
    Density,
}

/// The dynamics pop-up: pen pressure on/off, minimum value and the setting's pressure curve.
pub fn dynamics_popup(ctx: &egui::Context, st: &mut UiState) {
    let Some(target) = st.dynamics_popup else {
        return;
    };
    let tool = st.tool;
    let Some(mut p) = st
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.preset.clone())
    else {
        st.dynamics_popup = None;
        return;
    };
    let (label, on, curve) = match target {
        DynTarget::Size => ("Brush size", &mut p.size_pressure, &mut p.size_curve),
        DynTarget::Opacity => ("Opacity", &mut p.opacity_pressure, &mut p.opacity_curve),
        DynTarget::Density => (
            "Brush density",
            &mut p.density_pressure,
            &mut p.density_curve,
        ),
    };
    let mut open = true;
    egui::Window::new(format!("{label} dynamics"))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            let mut enabled = on.is_some();
            if ui.checkbox(&mut enabled, "Pen pressure").changed() {
                *on = enabled.then_some(0);
                if !enabled {
                    *curve = None;
                }
            }
            if let Some(minimum) = on {
                ui.add(
                    egui::Slider::new(minimum, 0..=100)
                        .text("Minimum value")
                        .suffix(" %"),
                );
                let mut pts: Vec<[f64; 2]> = curve
                    .as_ref()
                    .map(|c| c.points().to_vec())
                    .unwrap_or_default();
                if crate::curve_editor::curve_editor(
                    ui,
                    ui.scope_id().with("dyn-curve"),
                    &mut pts,
                    180.0,
                ) {
                    *curve = Curve::new(pts.clone()).ok().filter(|c| !c.is_linear());
                }
            }
        });
    if !open {
        st.dynamics_popup = None;
    }
    if let Some(s) = st.sets.get_mut(&tool).and_then(|s| s.current_mut())
        && s.preset != p
    {
        s.preset = p;
        st.sets_dirty = true;
    }
}

// ---------------------------------------------------------------- brush size palette

pub fn brush_size(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tool;
    let name = st
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.name.clone())
        .unwrap_or_default();
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    widgets::title(
        ui,
        c,
        title_r,
        icons::CIRCLES_THREE,
        &format!("Brush size: {name}"),
    );
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let Some(cur) = st
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.preset.size)
    else {
        return;
    };
    let mut pick = None;
    widgets::area(ui, body.shrink2(vec2(2.0, 4.0)), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("brush-sizes")
            .show(ui, |ui| {
                let [cw, ch] = m::BRUSH_CELL;
                let cols = 6;
                let rows = tools::BRUSH_SIZES.len().div_ceil(cols);
                let (area, _) = ui
                    .allocate_exact_size(vec2(cw * cols as f32, ch * rows as f32), Sense::hover());
                for (i, &size) in tools::BRUSH_SIZES.iter().enumerate() {
                    let cell = Rect::from_min_size(
                        area.min + vec2((i % cols) as f32 * cw, (i / cols) as f32 * ch),
                        vec2(cw - 1.0, ch - 1.0),
                    );
                    let resp = ui.interact(cell, ui.scope_id().with(("size", i)), Sense::click());
                    if (cur - size).abs() < 1e-9 {
                        widgets::fill(ui, cell, c.selection);
                    } else if resp.hovered() {
                        widgets::fill(ui, cell, c.bar);
                    }
                    let dot = (size as f32 / 2.0).clamp(0.6, 11.0);
                    ui.painter().circle_filled(
                        pos2(cell.center().x, cell.top() + 14.0),
                        dot,
                        Color32::WHITE,
                    );
                    let label = if size.fract() == 0.0 {
                        format!("{size:.0}")
                    } else {
                        format!("{size}")
                    };
                    ui.painter().text(
                        pos2(cell.center().x, cell.bottom() - 7.0),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0),
                        c.text,
                    );
                    ui.painter().line_segment(
                        [cell.right_top(), cell.right_bottom()],
                        Stroke::new(1.0, c.row_separator),
                    );
                    ui.painter().line_segment(
                        [cell.left_bottom(), cell.right_bottom()],
                        Stroke::new(1.0, c.row_separator),
                    );
                    if resp.clicked() {
                        pick = Some(size);
                    }
                }
            });
    });
    if let Some(size) = pick
        && let Some(s) = st.sets.get_mut(&tool).and_then(|s| s.current_mut())
    {
        s.preset.size = size;
        st.sets_dirty = true;
    }
}

// ---------------------------------------------------------------- sub tool detail

/// Every setting of the current sub tool, by category (CSP's Sub Tool Detail palette).
pub fn detail_window(ctx: &egui::Context, st: &mut UiState) {
    if !st.detail_open {
        return;
    }
    let tool = st.tool;
    let Some(mut sub) = st.sets.get(&tool).and_then(|s| s.current()).cloned() else {
        return;
    };
    let mut open = true;
    egui::Window::new(format!("Sub Tool Detail: {}", sub.name))
        .open(&mut open)
        .default_size([360.0, 420.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut sub.name);
            });
            let p = &mut sub.preset;
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new("Brush size")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Slider::new(&mut p.size, 0.5..=2000.0)
                                .logarithmic(true)
                                .suffix(" px"),
                        );
                        pressure_ui(ui, "Pen pressure", &mut p.size_pressure, &mut p.size_curve);
                    });
                egui::CollapsingHeader::new("Ink")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut op = p.opacity.unwrap_or(100);
                        if ui
                            .add(
                                egui::Slider::new(&mut op, 1..=100)
                                    .text("Opacity")
                                    .suffix(" %"),
                            )
                            .changed()
                        {
                            p.opacity = (op != 100).then_some(op);
                        }
                        pressure_ui(
                            ui,
                            "Opacity: pen pressure",
                            &mut p.opacity_pressure,
                            &mut p.opacity_curve,
                        );
                        let ink = p.ink.unwrap_or_default();
                        egui::ComboBox::from_label("Blending mode")
                            .selected_text(ink.name())
                            .show_ui(ui, |ui| {
                                for &m in InkMode::ALL {
                                    if ui.selectable_label(ink == m, m.name()).clicked() {
                                        p.ink = (m != InkMode::Normal).then_some(m);
                                    }
                                }
                            });
                    });
                egui::CollapsingHeader::new("Anti-aliasing").show(ui, |ui| {
                    let aa = p.aa.unwrap_or_default();
                    for a in [Aa::None, Aa::Weak, Aa::Middle, Aa::Strong] {
                        if ui.radio(aa == a, a.name()).clicked() {
                            p.aa = (a != Aa::default()).then_some(a);
                        }
                    }
                });
                egui::CollapsingHeader::new("Brush tip")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut h = p.hardness.unwrap_or(5);
                        if ui
                            .add(egui::Slider::new(&mut h, 1..=5).text("Hardness"))
                            .changed()
                        {
                            p.hardness = (h != 5).then_some(h);
                        }
                        let mut th = p.thickness.unwrap_or(100);
                        if ui
                            .add(
                                egui::Slider::new(&mut th, 1..=100)
                                    .text("Thickness")
                                    .suffix(" %"),
                            )
                            .changed()
                        {
                            p.thickness = (th != 100).then_some(th);
                        }
                        let dir = p.direction.unwrap_or_default();
                        egui::ComboBox::from_label("Direction")
                            .selected_text(dir.name())
                            .show_ui(ui, |ui| {
                                for d in [TipAxis::Vertical, TipAxis::Horizontal] {
                                    if ui.selectable_label(dir == d, d.name()).clicked() {
                                        p.direction = (d != TipAxis::default()).then_some(d);
                                    }
                                }
                            });
                        let mut a = p.angle.unwrap_or(0.0);
                        if ui
                            .add(
                                egui::Slider::new(&mut a, 0.0..=360.0)
                                    .text("Angle")
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            p.angle = (a != 0.0).then_some(a);
                        }
                        let mut line = p.angle_line.unwrap_or(false);
                        if ui.checkbox(&mut line, "Angle: direction of line").changed() {
                            p.angle_line = line.then_some(true);
                        }
                        let mut den = p.density.unwrap_or(100);
                        if ui
                            .add(
                                egui::Slider::new(&mut den, 1..=100)
                                    .text("Brush density")
                                    .suffix(" %"),
                            )
                            .changed()
                        {
                            p.density = (den != 100).then_some(den);
                        }
                        pressure_ui(
                            ui,
                            "Density: pen pressure",
                            &mut p.density_pressure,
                            &mut p.density_curve,
                        );
                        let mut dbg = p.density_by_gap.unwrap_or(false);
                        if ui
                            .checkbox(&mut dbg, "Adjust brush density by gap")
                            .changed()
                        {
                            p.density_by_gap = dbg.then_some(true);
                        }
                    });
                egui::CollapsingHeader::new("Stroke")
                    .default_open(true)
                    .show(ui, |ui| {
                        let gap = p.gap.unwrap_or_default();
                        ui.horizontal(|ui| {
                            ui.label("Gap");
                            for (g, n) in [
                                (Gap::Fixed(10.0), "Fixed"),
                                (Gap::Wide, "Wide"),
                                (Gap::Normal, "Normal"),
                                (Gap::Narrow, "Narrow"),
                            ] {
                                let sel =
                                    std::mem::discriminant(&gap) == std::mem::discriminant(&g);
                                if ui.selectable_label(sel, n).clicked() && !sel {
                                    p.gap = (g != Gap::Normal).then_some(g);
                                }
                            }
                        });
                        if let Some(Gap::Fixed(v)) = &mut p.gap {
                            ui.add(
                                egui::Slider::new(v, 1.0..=1000.0)
                                    .text("Fixed gap")
                                    .suffix(" %"),
                            );
                        }
                    });
                egui::CollapsingHeader::new("Correction").show(ui, |ui| {
                    let mut s = p.stabilization.unwrap_or(0);
                    if ui
                        .add(egui::Slider::new(&mut s, 0..=100).text("Stabilization"))
                        .changed()
                    {
                        p.stabilization = (s != 0).then_some(s);
                    }
                });
            });
            if let Err(e) = p.validate() {
                ui.colored_label(Color32::LIGHT_RED, e);
            }
        });
    st.detail_open = open;
    if let Some(s) = st.sets.get_mut(&tool).and_then(|s| s.current_mut())
        && *s != sub
        && sub.preset.validate().is_ok()
    {
        *s = sub;
        st.sets_dirty = true;
    }
}

fn pressure_ui(ui: &mut Ui, label: &str, on: &mut Option<u8>, curve: &mut Option<Curve>) {
    let mut enabled = on.is_some();
    if ui.checkbox(&mut enabled, label).changed() {
        *on = enabled.then_some(0);
        if !enabled {
            *curve = None;
        }
    }
    if let Some(minimum) = on {
        ui.add(
            egui::Slider::new(minimum, 0..=100)
                .text("Minimum")
                .suffix(" %"),
        );
        let mut pts: Vec<[f64; 2]> = curve
            .as_ref()
            .map(|c| c.points().to_vec())
            .unwrap_or_default();
        if crate::curve_editor::curve_editor(ui, ui.scope_id().with(label), &mut pts, 140.0) {
            *curve = Curve::new(pts).ok().filter(|c| !c.is_linear());
        }
    }
}
