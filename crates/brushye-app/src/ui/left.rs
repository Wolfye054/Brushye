//! Left side: tool column (U-004), and the left dock's Sub Tool, Tool property and Brush size
//! palettes, plus the Sub Tool Detail window.

use brushye_command::brush_preset::BrushPreset;
use brushye_command::core::brush::{self, BrushSettings, Curve};
use brushye_command::core::input::Sample;
use brushye_command::core::{Layer, LayerId, LayerKind};
use eframe::egui::{self, Color32, ColorImage, Rect, Sense, Stroke, TextureHandle, Ui, pos2, vec2};

use super::theme::{Colors, m};
use super::tools::{self, SubTool, TOOLS, ToolId};
use super::widgets;
use super::{UiState, rows, tool_cmds};
use crate::icons;
use brushye_command::tools::Target;

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
        tools::info(st.tools.tool).icon,
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
                st.tools.tool == t.id,
            )
        });
        if resp.clicked() {
            st.select_tool(t.id);
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

/// Dialogs of the Sub Tool palette (U-011).
#[derive(Clone, Debug, PartialEq)]
pub enum SubToolDialog {
    /// Sub tool group settings: rename group `group`.
    Group { group: usize, name: String },
    /// Duplicate the selected sub tool under a new name.
    Duplicate { name: String },
    /// Sub tool settings: rename the selected sub tool.
    Settings { name: String },
    /// Create custom sub tool: a new sub tool from the tool's plain default.
    Create { name: String },
    /// Export the selected sub tool as a `.brshbrush` file.
    Export { path: String },
}

/// Queue the commands that turn the current sub tool of `tool` into `new`.
fn queue_changes(st: &mut UiState, tool: ToolId, new: &SubTool) {
    if let Ok(t) = st.tools.target(Some(tool), None, None) {
        let old = st.tools.get(t).clone();
        for cmd in tool_cmds::changes(t, &old, new) {
            st.cmd(cmd);
        }
    }
}

/// The sub tool menu shared by the palette's ≡ button and a row's right-click (U-011).
fn sub_tool_menu(ui: &mut Ui, st: &mut UiState, palette: bool) {
    let tool = st.tools.tool;
    let Some(set) = st.tools.sets.get(&tool) else {
        return;
    };
    let cur = set.current().cloned();
    let group = set.group;
    let count = set.groups[group].items.len();
    let item = |ui: &mut Ui, label: &str, enabled: bool| {
        let r = ui.add_enabled(enabled, egui::Button::new(label));
        if r.clicked() {
            ui.close();
        }
        r.clicked()
    };
    if palette && item(ui, "Sub tool group settings…", true) {
        let name = st.tools.sets[&tool].groups[group].name.clone();
        st.subtool_dialog = Some(SubToolDialog::Group { group, name });
    }
    if palette {
        ui.separator();
    }
    if let Some(s) = &cur {
        if item(ui, "Duplicate sub tool…", true) {
            st.subtool_dialog = Some(SubToolDialog::Duplicate {
                name: format!("{} 2", s.name),
            });
        }
        if item(ui, "Sub tool settings…", true) {
            st.subtool_dialog = Some(SubToolDialog::Settings {
                name: s.name.clone(),
            });
        }
        if item(ui, "Delete sub tool", count > 1) {
            let sel = st.tools.sets[&tool].groups[group].selected;
            st.pending_subtool_op = Some(SubToolOp::Delete(sel));
        }
    }
    if item(ui, "Create custom sub tool…", true) {
        st.subtool_dialog = Some(SubToolDialog::Create {
            name: tools::info(tool).name.to_owned(),
        });
    }
    ui.separator();
    if item(ui, "Add from default…", true) {
        st.pending_subtool_op = Some(SubToolOp::AddDefaults);
    }
    ui.separator();
    if let Some(s) = &cur {
        let locked = s.locked();
        if ui.add(egui::Button::selectable(locked, "Lock")).clicked() {
            st.pending_subtool_op = Some(SubToolOp::ToggleLock);
            ui.close();
        }
        if item(ui, "Reset to default", true) {
            st.pending_subtool_op = Some(SubToolOp::Reset);
        }
        if item(ui, "Save as default", true) {
            st.pending_subtool_op = Some(SubToolOp::SaveDefault);
        }
    }
    ui.separator();
    item(ui, "Modifier key settings…", false);
    ui.separator();
    if item(ui, "Import sub tool…", true) {
        st.import_brush = Some(String::new());
    }
    if item(ui, "Export sub tool…", cur.is_some()) {
        let name = cur.as_ref().map_or("brush".into(), |s| s.name.clone());
        st.subtool_dialog = Some(SubToolDialog::Export {
            path: format!("{name}.brshbrush"),
        });
    }
    item(ui, "Register sub tool as material…", false);
    if item(ui, "Add sub tool…", true) {
        st.import_brush = Some(String::new());
    }
    if !palette {
        return;
    }
    ui.separator();
    ui.menu_button("View", |ui| {
        if ui
            .add(egui::Button::selectable(!st.prefs.subtool_text, "Stroke"))
            .clicked()
        {
            st.prefs.subtool_text = false;
            ui.close();
        }
        ui.add_enabled(false, egui::Button::new("Tile"));
        ui.add_enabled(false, egui::Button::new("Tile small"));
        if ui
            .add(egui::Button::selectable(st.prefs.subtool_text, "Text"))
            .clicked()
        {
            st.prefs.subtool_text = true;
            ui.close();
        }
    });
    if ui
        .add(egui::Button::selectable(
            st.prefs.show_group_names,
            "Show group name of sub tool",
        ))
        .clicked()
    {
        st.prefs.show_group_names = !st.prefs.show_group_names;
        ui.close();
    }
    if ui
        .add(egui::Button::selectable(
            st.prefs.subtool_bar,
            "Show command bar",
        ))
        .clicked()
    {
        st.prefs.subtool_bar = !st.prefs.subtool_bar;
        ui.close();
    }
    ui.menu_button("Change order", |ui| {
        if ui
            .add(egui::Button::selectable(!st.prefs.drag_needs_ctrl, "Drag"))
            .clicked()
        {
            st.prefs.drag_needs_ctrl = false;
            ui.close();
        }
        if ui
            .add(egui::Button::selectable(
                st.prefs.drag_needs_ctrl,
                "Ctrl + Drag",
            ))
            .clicked()
        {
            st.prefs.drag_needs_ctrl = true;
            ui.close();
        }
    });
    ui.separator();
    if item(
        ui,
        &format!("Hide Sub Tool: {} palette", tools::info(tool).name),
        true,
    ) {
        st.hidden.insert("sub_tool");
    }
}

pub fn sub_tool(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tools.tool;
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let menu = widgets::title(
        ui,
        c,
        title_r,
        icons::PAINT_BRUSH,
        &format!("Sub Tool: {}", tools::info(tool).name),
    );
    egui::Popup::menu(&menu).show(|ui| sub_tool_menu(ui, st, true));
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let Some(set) = st.tools.sets.get(&tool).cloned() else {
        ui.painter().text(
            body.center(),
            egui::Align2::CENTER_CENTER,
            "No sub tools",
            egui::FontId::proportional(11.0),
            c.text_dim,
        );
        return;
    };
    // Group tabs, two per row (U-004); right-click opens group settings (U-011).
    let mut y = body.top() + 5.0;
    let tab_w = (body.width() - 8.0) / 2.0;
    let mut switch_group = None;
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
            tools::group_icon(tool, i),
            13.0,
            c.icon,
        );
        if st.prefs.show_group_names {
            ui.painter().text(
                pos2(tr.left() + 24.0, tr.center().y),
                egui::Align2::LEFT_CENTER,
                &g.name,
                egui::FontId::proportional(11.0),
                c.text,
            );
        }
        if resp.clicked() && set.group != i {
            switch_group = Some(i);
        }
        let name = g.name.clone();
        resp.context_menu(|ui| {
            if ui.button("Sub tool group settings…").clicked() {
                st.subtool_dialog = Some(SubToolDialog::Group {
                    group: i,
                    name: name.clone(),
                });
                ui.close();
            }
        });
    }
    if let Some(i) = switch_group {
        st.cmd(tool_cmds::select_group(tool, i));
    }
    y += set.groups.len().div_ceil(2) as f32 * 28.0 + 1.0;
    let bar_h = if st.prefs.subtool_bar {
        m::BOTTOM_BAR + 1.0
    } else {
        0.0
    };
    let list = Rect::from_min_max(
        pos2(body.left(), y),
        pos2(body.right(), body.bottom() - bar_h),
    );
    let group = set.group;
    let items: Vec<SubTool> = set.groups[group].items.clone();
    let selected = set.groups[group].selected;
    let row_h = if st.prefs.subtool_text {
        20.0
    } else {
        m::SUB_TOOL_ROW
    };
    let mut drop: Option<(usize, usize)> = None;
    let mut select = None;
    widgets::area(ui, list, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("subtools")
            .show(ui, |ui| {
                let mut rects = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let w = ui.available_width();
                    let (rr, resp) =
                        ui.allocate_exact_size(vec2(w, row_h), Sense::click_and_drag());
                    rects.push(rr);
                    if i == selected {
                        widgets::fill(ui, rr, c.selection);
                    }
                    let font = egui::FontId::proportional(11.0);
                    if st.prefs.subtool_text {
                        ui.painter().text(
                            pos2(rr.left() + 8.0, rr.center().y),
                            egui::Align2::LEFT_CENTER,
                            &item.name,
                            font,
                            c.text,
                        );
                    } else {
                        let tex = preview_texture(ui, st, &item.preset, (w as u32).max(40), 30);
                        ui.painter().image(
                            tex.id(),
                            Rect::from_min_size(rr.min, vec2(w, 30.0)),
                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
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
                    }
                    if item.locked() {
                        widgets::draw_icon(
                            ui,
                            pos2(rr.left() + 8.0, rr.top() + 8.0),
                            icons::LOCK,
                            9.0,
                            c.icon,
                        );
                    }
                    ui.painter().line_segment(
                        [
                            pos2(rr.left(), rr.bottom() - 0.5),
                            pos2(rr.right(), rr.bottom() - 0.5),
                        ],
                        Stroke::new(1.0, c.row_separator),
                    );
                    if resp.clicked() || resp.secondary_clicked() {
                        select = Some(i);
                    }
                    resp.context_menu(|ui| sub_tool_menu(ui, st, false));
                }
                // Drag a row onto another to reorder (U-011; optionally only with Ctrl).
                let ctrl_ok = !st.prefs.drag_needs_ctrl || ui.input(|i| i.modifiers.command);
                if ctrl_ok
                    && ui.ctx().dragged_id().is_some()
                    && let Some(pos) = ui.ctx().pointer_interact_pos()
                    && let Some(from) = rects.iter().position(|r| {
                        r.contains(ui.ctx().input(|i| i.pointer.press_origin()).unwrap_or(pos))
                    })
                    && let Some(to) = rects.iter().position(|r| r.contains(pos))
                {
                    if to != from {
                        let r = rects[to];
                        ui.painter().line_segment(
                            [r.left_top(), r.right_top()],
                            Stroke::new(2.0, c.accent),
                        );
                    }
                    if ui.input(|i| i.pointer.any_released()) && to != from {
                        drop = Some((from, to));
                    }
                }
            });
    });
    if let Some(i) = select
        && i != selected
    {
        st.cmd(tool_cmds::select(Target {
            tool,
            group,
            item: i,
        }));
    }
    if let Some((from, to)) = drop {
        let at = |item| Target { tool, group, item };
        st.cmd(tool_cmds::move_to(at(from), to));
        st.cmd(tool_cmds::select(at(to)));
    }
    if st.prefs.subtool_bar {
        let bottom = Rect::from_min_max(pos2(body.left(), list.bottom() + 1.0), body.max);
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
                    "Create copy of currently selected sub tool",
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
                    "Add sub tool (import a .brshbrush)",
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
    }
    apply_op(st, tool);
}

#[derive(Clone, Copy, Debug)]
pub enum SubToolOp {
    Copy(usize),
    Delete(usize),
    AddDefaults,
    ToggleLock,
    Reset,
    SaveDefault,
}

fn apply_op(st: &mut UiState, tool: ToolId) {
    let Some(op) = st.pending_subtool_op.take() else {
        return;
    };
    let Ok(cur) = st.tools.target(Some(tool), None, None) else {
        return;
    };
    let at = |item| Target { item, ..cur };
    let cmd = match op {
        SubToolOp::Copy(i) => tool_cmds::copy(at(i), None),
        SubToolOp::Delete(i) => tool_cmds::delete(at(i)),
        SubToolOp::AddDefaults => tool_cmds::add_defaults(tool),
        SubToolOp::ToggleLock => tool_cmds::lock(cur, !st.tools.get(cur).locked()),
        SubToolOp::Reset => tool_cmds::reset(cur),
        SubToolOp::SaveDefault => tool_cmds::save_default(cur),
    };
    st.cmd(cmd);
}

/// Sub Tool palette dialogs (U-011).
pub fn sub_tool_dialogs(ctx: &egui::Context, st: &mut UiState) -> Option<String> {
    let mut d = st.subtool_dialog.take()?;
    let tool = st.tools.tool;
    let mut open = true;
    let mut done = false;
    let mut status = None;
    let title = match d {
        SubToolDialog::Group { .. } => "Sub tool group settings",
        SubToolDialog::Duplicate { .. } => "Duplicate sub tool",
        SubToolDialog::Settings { .. } => "Sub tool settings",
        SubToolDialog::Create { .. } => "Create custom sub tool",
        SubToolDialog::Export { .. } => "Export sub tool",
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            let (label, text) = match &mut d {
                SubToolDialog::Group { name, .. }
                | SubToolDialog::Duplicate { name }
                | SubToolDialog::Settings { name }
                | SubToolDialog::Create { name } => ("Name", name),
                SubToolDialog::Export { path } => ("File (.brshbrush)", path),
            };
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(egui::TextEdit::singleline(text).desired_width(260.0));
            });
            if matches!(
                d,
                SubToolDialog::Duplicate { .. } | SubToolDialog::Create { .. }
            ) {
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Output process");
                        ui.label("Direct draw");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Input process");
                        ui.label(tools::info(tool).name);
                    });
                });
            }
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    done = true;
                    let cur = st.tools.target(Some(tool), None, None).ok();
                    match &d {
                        SubToolDialog::Group { group, name } => {
                            if !name.trim().is_empty() {
                                st.cmd(tool_cmds::rename_group(tool, *group, name));
                            }
                        }
                        SubToolDialog::Duplicate { name } => {
                            if let Some(t) = cur {
                                st.cmd(tool_cmds::copy(t, Some(name.clone())));
                            }
                        }
                        SubToolDialog::Settings { name } => {
                            if let Some(t) = cur
                                && !name.trim().is_empty()
                            {
                                let old = st.tools.get(t).clone();
                                let mut new = old.clone();
                                new.name = name.trim().to_owned();
                                for cmd in tool_cmds::changes(t, &old, &new) {
                                    st.cmd(cmd);
                                }
                            }
                        }
                        SubToolDialog::Create { name } => {
                            st.cmd(tool_cmds::add(tool, name.trim(), None));
                        }
                        SubToolDialog::Export { path } => {
                            status = Some(export_sub_tool(st, path.trim()));
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    done = true;
                }
            });
        });
    if open && !done {
        st.subtool_dialog = Some(d);
    }
    status
}

/// Export the current sub tool as a `.brshbrush` file; returns the status message.
pub fn export_sub_tool(st: &UiState, path: &str) -> String {
    let Some(s) = st.tools.current(st.tools.tool) else {
        return "Choose a sub tool first".into();
    };
    let b = brushye_format::brush_file::BrushFile {
        name: s.name.clone(),
        settings: s.preset.to_json(),
        images: Default::default(),
    };
    let res = brushye_format::brush_file::save(&b)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            brushye_format::brsh::write_atomic(std::path::Path::new(path), &bytes)
                .map_err(|e| e.to_string())
        });
    match res {
        Ok(()) => format!("Exported “{}” to {path}", s.name),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------- tool property

fn shown_rows(s: &SubTool) -> Vec<rows::Row> {
    // Tool property lists the shown rows in Sub Tool Detail order.
    rows::ROWS
        .iter()
        .filter(|r| s.shown.iter().any(|k| k == r.key))
        .map(|r| r.row)
        .collect()
}

fn tool_property_menu(ui: &mut Ui, st: &mut UiState) {
    let tool = st.tools.tool;
    let locked = st
        .tools
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .is_some_and(|s| s.locked());
    let brush = tool.is_brush();
    if ui
        .add_enabled(brush, egui::Button::selectable(locked, "Lock"))
        .clicked()
    {
        st.pending_subtool_op = Some(SubToolOp::ToggleLock);
        ui.close();
    }
    if ui
        .add_enabled(brush, egui::Button::new("Reset to default"))
        .clicked()
    {
        st.pending_subtool_op = Some(SubToolOp::Reset);
        ui.close();
    }
    if ui
        .add_enabled(brush, egui::Button::new("Save as default"))
        .clicked()
    {
        st.pending_subtool_op = Some(SubToolOp::SaveDefault);
        ui.close();
    }
    ui.separator();
    ui.add_enabled(false, egui::Button::new("Modifier key settings…"));
    ui.separator();
    for (label, flag) in [
        ("Show category", &mut st.prefs.show_category),
        ("Show stroke preview", &mut st.prefs.show_preview),
        ("Show command bar", &mut st.prefs.tool_property_bar),
    ] {
        if ui.add(egui::Button::selectable(*flag, label)).clicked() {
            *flag = !*flag;
            ui.close();
        }
    }
    ui.separator();
    if ui.button("Hide Tool property palette").clicked() {
        st.hidden.insert("tool_property");
        ui.close();
    }
}

pub fn tool_property(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tools.tool;
    let name = st
        .tools
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.name.clone())
        .unwrap_or_else(|| tools::info(tool).name.to_owned());
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let menu = widgets::title(
        ui,
        c,
        title_r,
        icons::WRENCH,
        &format!("Tool property: {name}"),
    );
    egui::Popup::menu(&menu).show(|ui| tool_property_menu(ui, st));
    apply_op(st, tool);
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let bar_h = if st.prefs.tool_property_bar {
        m::BOTTOM_BAR
    } else {
        0.0
    };
    let bottom = Rect::from_min_max(pos2(body.left(), body.bottom() - bar_h), body.max);
    let Some(mut sub) = st.tools.sets.get(&tool).and_then(|s| s.current()).cloned() else {
        tool_property_other(
            ui,
            c,
            st,
            Rect::from_min_max(body.min, pos2(body.right(), bottom.top())),
        );
        widgets::fill(ui, bottom, c.bar);
        return;
    };
    let mut top = body.top();
    if st.prefs.show_preview {
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
        let lock = widgets::area(
            ui,
            Rect::from_min_size(
                pos2(strip.right() - 20.0, strip.bottom() - 18.0),
                vec2(18.0, 16.0),
            ),
            |ui| {
                widgets::icon_button(
                    ui,
                    c,
                    if sub.locked() {
                        icons::LOCK
                    } else {
                        icons::LOCK_OPEN
                    },
                    "Lock sub tool and don't save changes",
                    vec2(18.0, 16.0),
                    true,
                    sub.locked(),
                )
            },
        );
        if lock.clicked() {
            sub.base = if sub.locked() {
                None
            } else {
                Some(sub.preset.clone())
            };
        }
        top = strip.bottom() + 2.0;
    }
    let rows_r = Rect::from_min_max(pos2(body.left(), top), pos2(body.right(), bottom.top()));
    let mut dyn_open = None;
    widgets::area(ui, rows_r, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("tool-property")
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut last_cat = usize::MAX;
                for row in shown_rows(&sub) {
                    let info = rows::info(row);
                    if st.prefs.show_category && info.category != last_cat {
                        last_cat = info.category;
                        ui.label(
                            egui::RichText::new(rows::CATEGORIES[info.category].0.trim())
                                .size(10.0)
                                .color(c.text_dim),
                        );
                    }
                    let out = rows::draw(ui, c, row, &mut sub.preset);
                    if out.dynamics.is_some() {
                        dyn_open = out.dynamics;
                    }
                }
            });
    });
    if dyn_open.is_some() {
        st.dynamics_popup = dyn_open;
        st.dynamics_anchor = pointer_or_zero(ui.ctx());
    }
    if st.prefs.tool_property_bar {
        widgets::fill(ui, bottom, c.bar);
        widgets::area(ui, bottom, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if widgets::icon_button(
                    ui,
                    c,
                    icons::WRENCH,
                    "Show Sub Tool Detail and edit hidden settings",
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
                {
                    st.pending_subtool_op = Some(SubToolOp::Reset);
                }
            });
        });
    }
    queue_changes(st, tool, &sub);
}

/// Tool property for tools without sub tools.
fn tool_property_other(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    widgets::area(ui, r, |ui| {
        ui.add_space(6.0);
        match st.tools.tool {
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

// ---------------------------------------------------------------- dynamics pop-up

/// The pointer position, used to place pop-ups next to the button that opened them.
fn pointer_or_zero(ctx: &egui::Context) -> egui::Pos2 {
    ctx.input(|i| i.pointer.interact_pos())
        .unwrap_or(egui::Pos2::ZERO)
}

/// The dynamics pop-up (U-010): inputs with checkboxes and minimum values, and the pressure
/// and tilt graphs. Brushye has pen pressure (and, for the angle, the direction of the line);
/// tilt, velocity and random are listed and disabled.
pub fn dynamics_popup(ctx: &egui::Context, st: &mut UiState) {
    let Some(target) = st.dynamics_popup else {
        return;
    };
    let tool = st.tools.tool;
    let Some(mut p) = st.current_preset(tool).cloned() else {
        st.dynamics_popup = None;
        return;
    };
    let c = st.colors();
    let label = match target {
        rows::DynRow::Size => "Brush Size",
        rows::DynRow::Opacity => "Opacity",
        rows::DynRow::Density => "Brush density",
        rows::DynRow::Angle => "Angle",
    };
    let mut open = true;
    // Opened next to its button (U-010): to the left of buttons on the right half of the screen.
    let a = st.dynamics_anchor;
    let (pivot, at) = if a.x > ctx.content_rect().center().x {
        (egui::Align2::RIGHT_TOP, pos2(a.x - 8.0, a.y))
    } else {
        (egui::Align2::LEFT_TOP, pos2(a.x + 8.0, a.y))
    };
    egui::Window::new(format!("{label} Dynamics"))
        .collapsible(false)
        .resizable(false)
        .pivot(pivot)
        .fixed_pos(at)
        .constrain(true)
        .default_size([430.0, 357.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("Input affecting {}", label.to_lowercase()));
            ui.separator();
            if target == rows::DynRow::Angle {
                let mut line = p.angle_line == Some(true);
                if ui.checkbox(&mut line, "Direction of line").changed() {
                    p.angle_line = line.then_some(true);
                }
                for input in [
                    "Pen pressure",
                    "Tilt",
                    "Velocity",
                    "Random",
                    "Direction of pen",
                ] {
                    ui.add_enabled_ui(false, |ui| ui.checkbox(&mut false, input));
                }
                return;
            }
            let (on, curve) = match target {
                rows::DynRow::Size => (&mut p.size_pressure, &mut p.size_curve),
                rows::DynRow::Opacity => (&mut p.opacity_pressure, &mut p.opacity_curve),
                _ => (&mut p.density_pressure, &mut p.density_curve),
            };
            let mut enabled = on.is_some();
            ui.horizontal(|ui| {
                widgets::draw_icon(
                    ui,
                    ui.cursor().min + vec2(8.0, 9.0),
                    icons::PEN_NIB,
                    12.0,
                    c.icon,
                );
                ui.add_space(18.0);
                if ui.checkbox(&mut enabled, "Pen pressure").changed() {
                    *on = enabled.then_some(0);
                    if !enabled {
                        *curve = None;
                    }
                }
                ui.add_enabled_ui(enabled, |ui| {
                    let mut minimum = on.unwrap_or(0);
                    if ui
                        .add(
                            egui::Slider::new(&mut minimum, 0..=100)
                                .text("Minimum value")
                                .suffix(" %"),
                        )
                        .changed()
                    {
                        *on = Some(minimum);
                    }
                });
            });
            for input in ["Tilt", "Velocity", "Random"] {
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(18.0);
                        ui.checkbox(&mut false, input);
                    });
                });
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Pen pressure settings");
                    ui.add_enabled_ui(enabled, |ui| {
                        let mut pts: Vec<[f64; 2]> = curve
                            .as_ref()
                            .map(|c| c.points().to_vec())
                            .unwrap_or_default();
                        if crate::curve_editor::curve_editor(
                            ui,
                            ui.scope_id().with("dyn-curve"),
                            &mut pts,
                            176.0,
                        ) {
                            *curve = Curve::new(pts).ok().filter(|c| !c.is_linear());
                        }
                    });
                });
                ui.vertical(|ui| {
                    ui.label("Tilt settings");
                    let (r, _) = ui.allocate_exact_size(vec2(176.0, 176.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, c.value_field);
                    ui.painter().text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        "Not available yet",
                        egui::FontId::proportional(10.0),
                        c.text_disabled,
                    );
                });
            });
        });
    if !open {
        st.dynamics_popup = None;
    }
    if let Some(mut sub) = st.tools.current(tool).cloned()
        && sub.preset != p
    {
        sub.preset = p;
        queue_changes(st, tool, &sub);
    }
}

// ---------------------------------------------------------------- brush size palette

fn brush_size_menu(ui: &mut Ui, st: &mut UiState, cur: Option<f64>) {
    if ui
        .add(egui::Button::selectable(
            st.prefs.size_slider,
            "Show slider",
        ))
        .clicked()
    {
        st.prefs.size_slider = !st.prefs.size_slider;
        ui.close();
    }
    ui.separator();
    let has = cur.is_some_and(|v| st.brush_sizes.iter().any(|s| (s - v).abs() < 1e-9));
    if ui
        .add_enabled(
            cur.is_some() && !has,
            egui::Button::new("Add current size to preset"),
        )
        .clicked()
        && let Some(v) = cur
    {
        st.brush_sizes.push(v);
        st.brush_sizes.sort_by(f64::total_cmp);
        st.prefs_dirty = true;
        ui.close();
    }
    if ui
        .add_enabled(has, egui::Button::new("Delete preset of current size"))
        .clicked()
        && let Some(v) = cur
    {
        st.brush_sizes.retain(|s| (s - v).abs() >= 1e-9);
        st.prefs_dirty = true;
        ui.close();
    }
    ui.separator();
    if ui.button("Hide Brush size palette").clicked() {
        st.hidden.insert("brush_size");
        ui.close();
    }
}

pub fn brush_size(ui: &mut Ui, c: &Colors, st: &mut UiState, r: Rect) {
    let tool = st.tools.tool;
    let name = st
        .tools
        .sets
        .get(&tool)
        .and_then(|s| s.current())
        .map(|s| s.name.clone())
        .unwrap_or_default();
    let title_r = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let menu = widgets::title(
        ui,
        c,
        title_r,
        icons::CIRCLES_THREE,
        &format!("Brush size: {name}"),
    );
    let cur = st.current_preset(tool).map(|p| p.size);
    egui::Popup::menu(&menu).show(|ui| brush_size_menu(ui, st, cur));
    let body = Rect::from_min_max(pos2(r.left(), title_r.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    let Some(cur) = cur else { return };
    let mut pick = None;
    let mut grid = body.shrink2(vec2(2.0, 4.0));
    if st.prefs.size_slider {
        let sr = Rect::from_min_size(grid.min, vec2(grid.width(), 27.0));
        let mut v = cur;
        widgets::area(ui, sr, |ui| {
            if widgets::prop_slider(ui, c, "Brush Size", &mut v, 0.5..=2000.0, true, 1, None).0 {
                pick = Some(v);
            }
        });
        grid.min.y = sr.bottom() + 2.0;
    }
    let sizes = st.brush_sizes.clone();
    widgets::area(ui, grid, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("brush-sizes")
            .show(ui, |ui| {
                let [cw, ch] = m::BRUSH_CELL;
                let cols = 6;
                let rows_n = sizes.len().div_ceil(cols);
                let (area, _) = ui.allocate_exact_size(
                    vec2(cw * cols as f32, ch * rows_n as f32),
                    Sense::hover(),
                );
                for (i, &size) in sizes.iter().enumerate() {
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
        && let Some(mut sub) = st.tools.current(tool).cloned()
    {
        sub.preset.size = size;
        queue_changes(st, tool, &sub);
    }
}

// ---------------------------------------------------------------- sub tool detail

/// The Sub Tool Detail window (U-010): preview strip, category list, the page with its
/// description band and rows (eye and ▸ buttons per row), the help box, Show category, and
/// the two reset/save buttons.
pub fn detail_window(ctx: &egui::Context, st: &mut UiState) {
    if !st.detail_open {
        return;
    }
    let tool = st.tools.tool;
    let Some(mut sub) = st.tools.sets.get(&tool).and_then(|s| s.current()).cloned() else {
        return;
    };
    let c = st.colors();
    let others: Vec<(usize, usize, String)> = st
        .tools
        .sets
        .get(&tool)
        .map(|set| {
            set.groups
                .iter()
                .enumerate()
                .flat_map(|(gi, g)| {
                    g.items
                        .iter()
                        .enumerate()
                        .map(move |(ii, s)| (gi, ii, s.name.clone()))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut open = true;
    let mut copy: Option<(rows::Row, bool, usize, usize)> = None;
    let mut op = None;
    egui::Window::new("Sub Tool Detail")
        .open(&mut open)
        .default_pos([733.0, 325.0])
        .default_size([826.0, 750.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            // Preview strip (~100 px): name, stroke preview on a light band and the dark half.
            let (strip, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 100.0), Sense::hover());
            ui.painter().rect_filled(
                Rect::from_min_max(strip.min, pos2(strip.center().x, strip.bottom())),
                0.0,
                Color32::from_gray(200),
            );
            ui.painter().rect_filled(
                Rect::from_min_max(pos2(strip.center().x, strip.top()), strip.max),
                0.0,
                c.surround,
            );
            let mut pv = sub.preset.clone();
            pv.size = pv.size.min(60.0);
            let tex = preview_texture(ui, st, &pv, strip.width() as u32, 100);
            ui.painter().image(
                tex.id(),
                strip,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::from_gray(40),
            );
            ui.painter().text(
                strip.min + vec2(4.0, 8.0),
                egui::Align2::LEFT_CENTER,
                &sub.name,
                egui::FontId::proportional(11.0),
                Color32::from_gray(30),
            );
            ui.add_space(4.0);
            let body_h = ui.available_height() - 110.0;
            ui.horizontal_top(|ui| {
                // Category list (~135 px, text only, 19 px rows).
                ui.vertical(|ui| {
                    ui.set_width(135.0);
                    for (i, (name, available, _)) in rows::CATEGORIES.iter().enumerate() {
                        let (rr, resp) = ui.allocate_exact_size(vec2(135.0, 19.0), Sense::click());
                        if st.detail_category == i {
                            ui.painter().rect_filled(rr, 0.0, c.selection);
                        }
                        ui.painter().text(
                            pos2(rr.left() + 4.0, rr.center().y),
                            egui::Align2::LEFT_CENTER,
                            *name,
                            egui::FontId::proportional(11.0),
                            if *available { c.text } else { c.text_disabled },
                        );
                        if resp.clicked() {
                            st.detail_category = i;
                        }
                    }
                });
                ui.separator();
                // Page: description band, then rows.
                ui.vertical(|ui| {
                    let (_, _, desc) = rows::CATEGORIES[st.detail_category];
                    let (band, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                    ui.painter().rect_filled(band, 0.0, c.header);
                    ui.painter().text(
                        pos2(band.left() + 8.0, band.center().y),
                        egui::Align2::LEFT_CENTER,
                        desc,
                        egui::FontId::proportional(11.0),
                        c.text,
                    );
                    egui::ScrollArea::vertical()
                        .id_salt("detail-rows")
                        .max_height(body_h - 44.0)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let cat = st.detail_category;
                            let rows_here: Vec<&rows::RowInfo> =
                                rows::ROWS.iter().filter(|r| r.category == cat).collect();
                            if rows_here.is_empty() {
                                ui.add_space(8.0);
                                ui.colored_label(c.text_disabled, "Not available yet");
                            }
                            for info in rows_here {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    let shown = sub.shown.iter().any(|k| k == info.key);
                                    let (eye, menu) = rows::eye_and_menu(ui, c, shown, true);
                                    if eye.clicked() {
                                        if shown {
                                            sub.shown.retain(|k| k != info.key);
                                        } else {
                                            sub.shown.push(info.key.to_owned());
                                        }
                                    }
                                    egui::Popup::menu(&menu).show(|ui| {
                                        for (label, status) in
                                            [("Copy value", false), ("Copy displayed status", true)]
                                        {
                                            ui.menu_button(label, |ui| {
                                                for (gi, ii, name) in &others {
                                                    if *name != sub.name
                                                        && ui.button(name).clicked()
                                                    {
                                                        copy = Some((info.row, status, *gi, *ii));
                                                        ui.close();
                                                    }
                                                }
                                            });
                                        }
                                    });
                                    let out = ui
                                        .add_enabled_ui(info.enabled, |ui| {
                                            ui.vertical(|ui| {
                                                rows::draw(ui, c, info.row, &mut sub.preset)
                                            })
                                            .inner
                                        })
                                        .inner;
                                    if out.hovered {
                                        st.detail_hover = Some(info.row);
                                    }
                                    if out.dynamics.is_some() {
                                        st.dynamics_popup = out.dynamics;
                                        st.dynamics_anchor = pointer_or_zero(ui.ctx());
                                    }
                                });
                            }
                        });
                });
            });
            ui.separator();
            // Help box: explains the hovered row.
            ui.horizontal(|ui| {
                widgets::draw_icon(
                    ui,
                    ui.cursor().min + vec2(8.0, 8.0),
                    icons::INFO,
                    12.0,
                    c.icon,
                );
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("About parameter help").strong());
                    match st.detail_hover {
                        Some(row) => {
                            let i = rows::info(row);
                            ui.label(format!("About [{}]: {}", i.label, i.help));
                        }
                        None => {
                            ui.label("Hover over an item to see its description here.");
                        }
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.checkbox(&mut st.prefs.show_category, "Show category")
                    .on_hover_text("Show category names in the Tool property palette");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save all settings as default").clicked() {
                        op = Some(SubToolOp::SaveDefault);
                    }
                    if ui.button("Reset all settings to default").clicked() {
                        op = Some(SubToolOp::Reset);
                    }
                });
            });
            if let Err(e) = sub.preset.validate() {
                ui.colored_label(Color32::LIGHT_RED, e);
            }
        });
    st.detail_open = open;
    if let Some((row, status, gi, ii)) = copy
        && let Ok(t) = st
            .tools
            .target(Some(tool), Some(gi as u32), Some(ii as u32))
    {
        let old = st.tools.get(t).clone();
        let mut new = old.clone();
        copy_row(row, status, &sub, &mut new);
        for cmd in tool_cmds::changes(t, &old, &new) {
            st.cmd(cmd);
        }
    }
    queue_changes(st, tool, &sub);
    if op.is_some() {
        st.pending_subtool_op = op;
        apply_op(st, tool);
    }
}

/// Copy one row's value (or its shown-in-Tool-property status) to another sub tool (U-010).
fn copy_row(row: rows::Row, status: bool, from: &SubTool, to: &mut SubTool) {
    let key = rows::info(row).key;
    if status {
        let shown = from.shown.iter().any(|k| k == key);
        to.shown.retain(|k| k != key);
        if shown {
            to.shown.push(key.to_owned());
        }
        return;
    }
    let (a, b) = (&from.preset, &mut to.preset);
    use rows::Row as R;
    match row {
        R::BrushSize => {
            b.size = a.size;
            b.size_pressure = a.size_pressure;
            b.size_curve = a.size_curve.clone();
        }
        R::Opacity => {
            b.opacity = a.opacity;
            b.opacity_pressure = a.opacity_pressure;
            b.opacity_curve = a.opacity_curve.clone();
        }
        R::BlendMode => b.ink = a.ink,
        R::AntiAliasing => b.aa = a.aa,
        R::Hardness => b.hardness = a.hardness,
        R::Thickness => b.thickness = a.thickness,
        R::Direction => b.direction = a.direction,
        R::Angle => {
            b.angle = a.angle;
            b.angle_line = a.angle_line;
        }
        R::Density => {
            b.density = a.density;
            b.density_pressure = a.density_pressure;
            b.density_curve = a.density_curve.clone();
        }
        R::DensityByGap => b.density_by_gap = a.density_by_gap,
        R::Gap | R::GapValue => b.gap = a.gap,
        R::Stabilization => b.stabilization = a.stabilization,
        _ => {}
    }
}
