//! Menus, palettes, dialogs and shortcuts. Every change is a command (D-011).

use brushye_command::brush_preset::BrushPreset;
use brushye_command::commands::{self, Command};
use brushye_command::core::brush::{Aa, Gap, InkMode, TipAxis};
use brushye_command::core::{BlendMode, LayerId};
use brushye_format::brush_file;
use eframe::egui;

use crate::app::{Action, App, Dialog, ToolKind};

pub fn frame(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    // Only when it changes: a viewport command every frame keeps the app repainting.
    let title = app.title();
    if title != app.shown_title {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
        app.shown_title = title;
    }
    shortcuts(app, &ctx);
    close_request(app, &ctx);

    egui::Panel::top("menu").show(ui, |ui| menu(app, ui));
    egui::Panel::bottom("status").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(app.status_line());
            if !app.status.is_empty() {
                ui.separator();
                ui.label(&app.status);
            }
        });
    });
    egui::Panel::left("tools")
        .resizable(true)
        .default_size(250.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| tool_palette(app, ui));
        });
    egui::Panel::right("layers")
        .resizable(true)
        .default_size(240.0)
        .show(ui, |ui| layer_palette(app, ui));
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_gray(64)))
        .show(ui, |ui| app.canvas_ui(ui));
    dialogs(app, &ctx);
}

fn shortcut(ctx: &egui::Context, mods: egui::Modifiers, key: egui::Key) -> bool {
    ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(mods, key)))
}

fn shortcuts(app: &mut App, ctx: &egui::Context) {
    use egui::{Key, Modifiers as M};
    if app.dialog.is_some() {
        return;
    }
    let cmd_shift = M::COMMAND | M::SHIFT;
    if shortcut(ctx, cmd_shift, Key::Z) || shortcut(ctx, M::COMMAND, Key::Y) {
        app.redo();
    }
    if shortcut(ctx, M::COMMAND, Key::Z) {
        app.undo();
    }
    if shortcut(ctx, cmd_shift, Key::S) {
        app.dialog = Some(app.save_as_dialog());
    }
    if shortcut(ctx, M::COMMAND, Key::S) {
        app.save();
    }
    if shortcut(ctx, M::COMMAND, Key::N) {
        app.request(Action::New, ctx);
    }
    if shortcut(ctx, M::COMMAND, Key::O) {
        app.request(Action::Open, ctx);
    }
    if shortcut(ctx, M::COMMAND | M::ALT, Key::Num0) {
        app.actual_size();
    }
    if shortcut(ctx, M::COMMAND, Key::Num0) {
        app.fit();
    }
    if shortcut(ctx, M::COMMAND, Key::Plus) || shortcut(ctx, M::COMMAND, Key::Equals) {
        let c = app.canvas_center();
        app.view.step_zoom(1, c);
    }
    if shortcut(ctx, M::COMMAND, Key::Minus) {
        let c = app.canvas_center();
        app.view.step_zoom(-1, c);
    }
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    if shortcut(ctx, M::NONE, Key::P) || shortcut(ctx, M::NONE, Key::B) {
        app.tool = ToolKind::Pen;
    }
    if shortcut(ctx, M::NONE, Key::E) {
        app.tool = ToolKind::Eraser;
    }
    if shortcut(ctx, M::NONE, Key::OpenBracket) {
        let p = app.active_preset();
        p.size = (p.size / 1.2).max(0.5);
    }
    if shortcut(ctx, M::NONE, Key::CloseBracket) {
        let p = app.active_preset();
        p.size = (p.size * 1.2).min(2000.0);
    }
}

fn close_request(app: &mut App, ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().close_requested()) && app.dirty() && !app.allow_close {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        app.dialog = Some(Dialog::Confirm(Box::new(Action::Quit)));
    }
}

fn menu(app: &mut App, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New…  (Ctrl+N)").clicked() {
                app.request(Action::New, &ctx);
            }
            if ui.button("Open…  (Ctrl+O)").clicked() {
                app.request(Action::Open, &ctx);
            }
            if ui.button("Save  (Ctrl+S)").clicked() {
                app.save();
            }
            if ui.button("Save As…  (Ctrl+Shift+S)").clicked() {
                app.dialog = Some(app.save_as_dialog());
            }
            ui.separator();
            if ui.button("Export PNG…").clicked() {
                let path = app
                    .path
                    .as_ref()
                    .map(|p| p.with_extension("png").to_string_lossy().into_owned())
                    .unwrap_or_else(|| "untitled.png".into());
                app.dialog = Some(Dialog::ExportPng { path });
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                app.request(Action::Quit, &ctx);
            }
        });
        ui.menu_button("Edit", |ui| {
            if ui.button("Undo  (Ctrl+Z)").clicked() {
                app.undo();
            }
            if ui.button("Redo  (Ctrl+Y)").clicked() {
                app.redo();
            }
        });
        ui.menu_button("View", |ui| {
            let c = app.canvas_center();
            if ui.button("Zoom in  (Ctrl++)").clicked() {
                app.view.step_zoom(1, c);
            }
            if ui.button("Zoom out  (Ctrl+-)").clicked() {
                app.view.step_zoom(-1, c);
            }
            if ui.button("100 %  (Ctrl+Alt+0)").clicked() {
                app.actual_size();
            }
            if ui.button("Fit  (Ctrl+0)").clicked() {
                app.fit();
            }
            ui.separator();
            if ui.button("Rotate left 15°").clicked() {
                app.view.rotate_by(-15.0, c);
            }
            if ui.button("Rotate right 15°").clicked() {
                app.view.rotate_by(15.0, c);
            }
            if ui.button("Reset rotation").clicked() {
                let a = app.view.angle;
                app.view.rotate_by(-a, c);
            }
            ui.separator();
            ui.label("Pan: Space+drag or middle drag");
            ui.label("Rotate: Shift+Space+drag");
            ui.label("Zoom: mouse wheel");
        });
    });
}

fn pct(ui: &mut egui::Ui, label: &str, v: &mut Option<u8>, default: u8) {
    let mut x = v.unwrap_or(default);
    if ui
        .add(egui::Slider::new(&mut x, 1..=100).text(label).suffix(" %"))
        .changed()
    {
        *v = (x != default).then_some(x);
    }
}

fn pressure(ui: &mut egui::Ui, label: &str, v: &mut Option<u8>) {
    let mut on = v.is_some();
    ui.horizontal(|ui| {
        if ui.checkbox(&mut on, label).changed() {
            *v = on.then_some(0);
        }
        if let Some(m) = v {
            ui.add(egui::Slider::new(m, 0..=100).text("min").suffix(" %"));
        }
    });
}

fn preset_ui(ui: &mut egui::Ui, p: &mut BrushPreset, eraser: bool) {
    ui.add(
        egui::Slider::new(&mut p.size, 0.5..=2000.0)
            .logarithmic(true)
            .text("Size")
            .suffix(" px"),
    );
    pct(ui, "Opacity", &mut p.opacity, 100);
    pct(ui, "Density", &mut p.density, 100);
    let mut h = p.hardness.unwrap_or(5);
    if ui
        .add(egui::Slider::new(&mut h, 1..=5).text("Hardness"))
        .changed()
    {
        p.hardness = (h != 5).then_some(h);
    }
    let aa = p.aa.unwrap_or_default();
    egui::ComboBox::from_label("Anti-aliasing")
        .selected_text(aa.name())
        .show_ui(ui, |ui| {
            for a in [Aa::None, Aa::Weak, Aa::Middle, Aa::Strong] {
                if ui.selectable_label(aa == a, a.name()).clicked() {
                    p.aa = (a != Aa::default()).then_some(a);
                }
            }
        });
    let gap = p.gap.unwrap_or_default();
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("gap")
            .selected_text(match gap {
                Gap::Narrow => "Narrow".to_owned(),
                Gap::Normal => "Normal".to_owned(),
                Gap::Wide => "Wide".to_owned(),
                Gap::Fixed(v) => format!("Fixed {v} %"),
            })
            .show_ui(ui, |ui| {
                for (g, n) in [
                    (Gap::Narrow, "Narrow"),
                    (Gap::Normal, "Normal"),
                    (Gap::Wide, "Wide"),
                    (Gap::Fixed(10.0), "Fixed"),
                ] {
                    if ui
                        .selectable_label(
                            std::mem::discriminant(&gap) == std::mem::discriminant(&g),
                            n,
                        )
                        .clicked()
                    {
                        p.gap = (g != Gap::Normal).then_some(g);
                    }
                }
            });
        if let Some(Gap::Fixed(v)) = &mut p.gap {
            ui.add(egui::DragValue::new(v).range(1.0..=1000.0).suffix(" %"));
        }
        ui.label("Gap");
    });
    let mut dbg = p.density_by_gap.unwrap_or(false);
    if ui.checkbox(&mut dbg, "Adjust density by gap").changed() {
        p.density_by_gap = dbg.then_some(true);
    }
    let mut st = p.stabilization.unwrap_or(0);
    if ui
        .add(egui::Slider::new(&mut st, 0..=100).text("Stabilization"))
        .changed()
    {
        p.stabilization = (st != 0).then_some(st);
    }
    ui.collapsing("Tip shape", |ui| {
        pct(ui, "Thickness", &mut p.thickness, 100);
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
        if ui.checkbox(&mut line, "Angle follows the line").changed() {
            p.angle_line = line.then_some(true);
        }
    });
    ui.collapsing("Pen pressure", |ui| {
        pressure(ui, "Size", &mut p.size_pressure);
        pressure(ui, "Opacity", &mut p.opacity_pressure);
        pressure(ui, "Density", &mut p.density_pressure);
        ui.label("(Curves: brush editor, M7)");
    });
    if !eraser {
        let ink = p.ink.unwrap_or_default();
        egui::ComboBox::from_label("Ink")
            .selected_text(ink.name())
            .show_ui(ui, |ui| {
                for &m in InkMode::ALL {
                    if ui.selectable_label(ink == m, m.name()).clicked() {
                        p.ink = (m != InkMode::Normal).then_some(m);
                    }
                }
            });
    }
}

fn tool_palette(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Tool");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.tool, ToolKind::Pen, "Pen (P)");
        ui.selectable_value(&mut app.tool, ToolKind::Eraser, "Eraser (E)");
    });
    ui.separator();
    ui.heading("Colour");
    egui::widgets::color_picker::color_picker_color32(
        ui,
        &mut app.color,
        egui::color_picker::Alpha::Opaque,
    );
    let c = app.color;
    ui.monospace(format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b()));
    ui.separator();
    ui.heading(match app.tool {
        ToolKind::Pen => "Pen",
        ToolKind::Eraser => "Eraser",
    });
    let eraser = app.tool == ToolKind::Eraser;
    preset_ui(ui, app.active_preset(), eraser);
    if let Err(e) = app.active_preset().validate() {
        ui.colored_label(egui::Color32::LIGHT_RED, e);
    }
    ui.separator();
    brush_library(app, ui);
}

fn brush_library(app: &mut App, ui: &mut egui::Ui) {
    ui.collapsing("Brush library", |ui| {
        ui.horizontal(|ui| {
            ui.label("Folder");
            ui.text_edit_singleline(&mut app.brush_dir);
        });
        match brush_file::scan_library(std::path::Path::new(&app.brush_dir)) {
            Ok(list) if !list.is_empty() => {
                for e in list {
                    match &e.brush {
                        Ok(b) => {
                            if ui.button(format!("Use “{}”", b.name)).clicked() {
                                match BrushPreset::from_json(&b.settings) {
                                    Ok(p) => {
                                        *app.active_preset() = p;
                                        app.status = format!("Brush “{}”", b.name);
                                    }
                                    Err(x) => app.status = format!("{}: {x}", b.name),
                                }
                            }
                        }
                        Err(x) => {
                            ui.colored_label(
                                egui::Color32::LIGHT_RED,
                                format!("{}: {x}", e.path.display()),
                            );
                        }
                    }
                }
            }
            Ok(_) => {
                ui.label("No brushes yet.");
            }
            Err(_) => {
                ui.label("Folder not found (it's created on the first save).");
            }
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app.brush_name);
            if ui.button("Save brush").clicked() {
                save_brush(app);
            }
        });
    });
}

fn save_brush(app: &mut App) {
    let name = app.brush_name.trim().to_owned();
    if name.is_empty() {
        app.status = "Name the brush first".into();
        return;
    }
    let file: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let dir = std::path::PathBuf::from(&app.brush_dir);
    let b = brush_file::BrushFile {
        name: name.clone(),
        settings: app.active_preset().to_json(),
        images: Default::default(),
    };
    let result = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|()| brush_file::save(&b).map_err(|e| e.to_string()))
        .and_then(|bytes| {
            brushye_format::brsh::write_atomic(
                &dir.join(format!("{file}.{}", brush_file::EXTENSION)),
                &bytes,
            )
            .map_err(|e| e.to_string())
        });
    app.status = match result {
        Ok(()) => format!("Saved brush “{name}”"),
        Err(e) => e,
    };
}

fn next_layer_id(app: &App) -> LayerId {
    let max = app
        .doc()
        .map(|d| d.layers().iter().map(|l| l.id.0).max().unwrap_or(0))
        .unwrap_or(0);
    LayerId(max + 1)
}

fn layer_set(app: &mut App, id: LayerId) -> commands::LayerSet {
    let _ = app;
    commands::LayerSet {
        layer: id,
        name: None,
        opacity: None,
        blend: None,
        locked: None,
        lock_alpha: None,
    }
}

fn layer_palette(app: &mut App, ui: &mut egui::Ui) {
    ui.heading("Layers");
    let Some(doc) = app.doc() else { return };
    let active = app.session.active_layer();
    ui.horizontal(|ui| {
        if ui.button("New").clicked() {
            let id = next_layer_id(app);
            app.exec(Command::LayerNew(commands::LayerNew {
                layer: id,
                kind: None,
                name: Some(format!("Layer {}", id.0)),
                above: active,
            }));
        }
        let Some(a) = active else { return };
        if ui.button("Delete").clicked() {
            app.exec(Command::LayerDelete(commands::LayerDelete { layer: a }));
        }
        let ids: Vec<LayerId> = doc.layers().iter().map(|l| l.id).collect();
        let i = ids.iter().position(|&x| x == a).unwrap_or(0);
        if ui.button("Up").clicked() && i + 1 < ids.len() {
            app.exec(Command::LayerMove(commands::LayerMove {
                layer: a,
                above: Some(ids[i + 1]),
                to: None,
            }));
        }
        if ui.button("Down").clicked() && i > 0 {
            let cmd = if i >= 2 {
                commands::LayerMove {
                    layer: a,
                    above: Some(ids[i - 2]),
                    to: None,
                }
            } else {
                commands::LayerMove {
                    layer: a,
                    above: None,
                    to: Some(commands::Place::Bottom),
                }
            };
            app.exec(Command::LayerMove(cmd));
        }
    });
    if let Some(l) = active.and_then(|a| doc.layer(a)).cloned() {
        ui.separator();
        let id = l.id;
        let mut name = l.name.clone();
        let r = ui.text_edit_singleline(&mut name);
        if r.changed() && !name.is_empty() {
            let mut c = layer_set(app, id);
            c.name = Some(name);
            app.exec(Command::LayerSet(c));
        }
        egui::ComboBox::from_label("Blend")
            .selected_text(l.blend.name())
            .show_ui(ui, |ui| {
                for &m in BlendMode::ALL {
                    if ui.selectable_label(l.blend == m, m.name()).clicked() && l.blend != m {
                        let mut c = layer_set(app, id);
                        c.blend = Some(m);
                        app.exec(Command::LayerSet(c));
                    }
                }
            });
        let mut op = l.opacity;
        if ui
            .add(
                egui::Slider::new(&mut op, 0..=100)
                    .text("Opacity")
                    .suffix(" %"),
            )
            .changed()
        {
            let mut c = layer_set(app, id);
            c.opacity = Some(op);
            app.exec(Command::LayerSet(c));
        }
        ui.horizontal(|ui| {
            let mut lock = l.locked;
            if ui.checkbox(&mut lock, "Lock").changed() {
                let mut c = layer_set(app, id);
                c.locked = Some(lock);
                app.exec(Command::LayerSet(c));
            }
            let mut la = l.lock_alpha;
            if ui.checkbox(&mut la, "Lock transparency").changed() {
                let mut c = layer_set(app, id);
                c.lock_alpha = Some(la);
                app.exec(Command::LayerSet(c));
            }
        });
    }
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for l in doc.layers().iter().rev() {
            ui.horizontal(|ui| {
                let mut vis = l.visible;
                if ui
                    .checkbox(&mut vis, "")
                    .on_hover_text("Show/hide")
                    .changed()
                {
                    app.exec(Command::LayerShow(commands::LayerShow {
                        layer: l.id,
                        visible: vis,
                    }));
                }
                let label = format!(
                    "{}{}  ·  {} {} %",
                    l.name,
                    if l.locked { " 🔒" } else { "" },
                    l.blend.name(),
                    l.opacity
                );
                if ui.selectable_label(active == Some(l.id), label).clicked()
                    && active != Some(l.id)
                {
                    app.exec(Command::LayerSelect(commands::LayerSelect { layer: l.id }));
                }
            });
        }
    });
}

fn dialogs(app: &mut App, ctx: &egui::Context) {
    let Some(mut d) = app.dialog.take() else {
        return;
    };
    let mut open = true;
    let mut done = false;
    let title = match &d {
        Dialog::New { .. } => "New document",
        Dialog::Open { .. } => "Open",
        Dialog::SaveAs { .. } => "Save as",
        Dialog::ExportPng { .. } => "Export PNG",
        Dialog::Confirm(_) => "Unsaved changes",
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| match &mut d {
            Dialog::New {
                width,
                height,
                dpi,
                paper,
            } => {
                egui::Grid::new("new").show(ui, |ui| {
                    ui.label("Width");
                    ui.add(egui::DragValue::new(width).range(1..=32768).suffix(" px"));
                    ui.end_row();
                    ui.label("Height");
                    ui.add(egui::DragValue::new(height).range(1..=32768).suffix(" px"));
                    ui.end_row();
                    ui.label("Resolution");
                    ui.add(egui::DragValue::new(dpi).range(1..=2400).suffix(" dpi"));
                    ui.end_row();
                    ui.label("Paper");
                    ui.checkbox(paper, "White paper");
                    ui.end_row();
                });
                if ui.button("Create").clicked() {
                    let (w, h, r, p) = (*width, *height, *dpi, *paper);
                    app.new_document(w, h, r, p);
                    done = true;
                }
            }
            Dialog::Open { path } => {
                ui.label("File (.brsh)");
                let r = ui.text_edit_singleline(path);
                if ui.button("Open").clicked()
                    || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    let p = path.clone();
                    app.open_path(&p);
                    done = true;
                }
            }
            Dialog::SaveAs {
                path,
                embed_journal,
            } => {
                ui.label("File (.brsh)");
                ui.text_edit_singleline(path);
                ui.checkbox(embed_journal, "Embed the session journal in the file");
                ui.label("The journal is also written next to the file as .bcmd.");
                if ui.button("Save").clicked() {
                    let mut p = std::path::PathBuf::from(path.trim());
                    if p.extension().is_none() {
                        p.set_extension("brsh");
                    }
                    app.embed_journal = *embed_journal;
                    app.save_to(p);
                    done = true;
                }
            }
            Dialog::ExportPng { path } => {
                ui.label("File (.png)");
                ui.text_edit_singleline(path);
                if ui.button("Export").clicked() {
                    let p = path.clone();
                    app.export_png(&p);
                    done = true;
                }
            }
            Dialog::Confirm(action) => {
                ui.label("The document has unsaved changes. Discard them?");
                ui.horizontal(|ui| {
                    if ui.button("Discard").clicked() {
                        let a = (**action).clone();
                        app.run_action(a, ctx);
                        done = true;
                    }
                    if ui.button("Cancel").clicked() {
                        done = true;
                    }
                });
            }
        });
    if open && !done && app.dialog.is_none() {
        app.dialog = Some(d);
    }
}
