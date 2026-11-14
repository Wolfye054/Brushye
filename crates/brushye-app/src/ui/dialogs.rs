//! Dialogs and windows: New, Open, Save as, Save duplicate, Export, Pen pressure settings,
//! Preferences (interface colour), About, sub tool import, unsaved-changes confirmation.

use eframe::egui;

use super::theme::Interface;
use crate::app::{App, Dialog};

/// The window caption (CSP's shape: "<name>* (<w> x <h>px <dpi>dpi <zoom>%) - Brushye").
pub fn window_title(app: &mut App, ctx: &egui::Context) {
    let title = app.title();
    if title != app.shown_title {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
        app.shown_title = title;
    }
}

pub fn close_request(app: &mut App, ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().close_requested()) && app.dirty() && !app.allow_close {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        app.dialog = Some(Dialog::Confirm(Box::new(crate::app::Action::Quit)));
    }
}

fn path_with_ext(path: &str, ext: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(path.trim());
    if p.extension().is_none() {
        p.set_extension(ext);
    }
    p
}

pub fn dialogs(app: &mut App, ctx: &egui::Context) {
    preferences(app, ctx);
    about(app, ctx);
    import_brush(app, ctx);
    let Some(mut d) = app.dialog.take() else {
        return;
    };
    let mut open = true;
    let mut done = false;
    let title = match &d {
        Dialog::New { .. } => "New",
        Dialog::Open { .. } => "Open",
        Dialog::SaveAs { .. } => "Save as",
        Dialog::SaveDuplicate { .. } => "Save duplicate",
        Dialog::ExportPng { .. } => "Export (.png)",
        Dialog::PenPressure { .. } => "Pen pressure settings",
        Dialog::Confirm(_) => "Brushye",
    };
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| match &mut d {
            Dialog::New { width, height, dpi, paper } => {
                egui::Grid::new("new").num_columns(2).show(ui, |ui| {
                    ui.label("Width");
                    ui.add(egui::DragValue::new(width).range(1..=32768).suffix(" px"));
                    ui.end_row();
                    ui.label("Height");
                    ui.add(egui::DragValue::new(height).range(1..=32768).suffix(" px"));
                    ui.end_row();
                    ui.label("Resolution");
                    ui.add(egui::DragValue::new(dpi).range(1..=2400).suffix(" dpi"));
                    ui.end_row();
                    ui.label("Paper color");
                    ui.checkbox(paper, "White");
                    ui.end_row();
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        let (w, h, r, p) = (*width, *height, *dpi, *paper);
                        app.new_document(w, h, r, p);
                        done = true;
                    }
                    if ui.button("Cancel").clicked() {
                        done = true;
                    }
                });
            }
            Dialog::Open { path } => {
                ui.label("File (.brsh)");
                let r = ui.add(egui::TextEdit::singleline(path).desired_width(360.0));
                if ui.button("Open").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    let p = path.clone();
                    app.open_path(&p);
                    done = true;
                }
            }
            Dialog::SaveAs { path, embed_journal } => {
                ui.label("File (.brsh)");
                ui.add(egui::TextEdit::singleline(path).desired_width(360.0));
                ui.checkbox(embed_journal, "Embed the session journal in the file");
                ui.label("The journal is also written next to the file as .bcmd.");
                if ui.button("Save").clicked() {
                    app.embed_journal = *embed_journal;
                    app.save_to(path_with_ext(path, "brsh"));
                    done = true;
                }
            }
            Dialog::SaveDuplicate { path } => {
                ui.label("File (.brsh): a copy; the open document keeps its own file");
                ui.add(egui::TextEdit::singleline(path).desired_width(360.0));
                if ui.button("Save").clicked() {
                    app.save_duplicate(path_with_ext(path, "brsh"));
                    done = true;
                }
            }
            Dialog::ExportPng { path } => {
                ui.label("File (.png)");
                ui.add(egui::TextEdit::singleline(path).desired_width(360.0));
                if ui.button("Export").clicked() {
                    let p = path_with_ext(path, "png").to_string_lossy().into_owned();
                    app.export_png(&p);
                    done = true;
                }
            }
            Dialog::PenPressure { points } => {
                ui.label("Pen pressure curve, applied to the pen's raw pressure before each setting's own curve.");
                ui.label("Drag points; click to add a point, right-click to remove one.");
                crate::curve_editor::curve_editor(ui, ui.scope_id().with("global-pressure"), points, 240.0);
                ui.horizontal(|ui| {
                    if ui.button("Reset to default").clicked() {
                        points.clear();
                    }
                    if ui.button("OK").clicked() {
                        let p = points.clone();
                        app.set_pressure_curve(p);
                        done = true;
                    }
                    if ui.button("Cancel").clicked() {
                        done = true;
                    }
                });
            }
            Dialog::Confirm(action) => {
                ui.label("The canvas has unsaved changes. Discard them?");
                ui.horizontal(|ui| {
                    if ui.button("Discard changes").clicked() {
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

fn preferences(app: &mut App, ctx: &egui::Context) {
    if !app.ui.prefs_open {
        return;
    }
    let mut open = true;
    egui::Window::new("Preferences")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Interface");
            ui.horizontal(|ui| {
                ui.label("Interface color");
                let cur = app.ui.interface;
                egui::ComboBox::from_id_salt("interface")
                    .selected_text(if cur == Interface::Dark {
                        "Dark color"
                    } else {
                        "Light color"
                    })
                    .show_ui(ui, |ui| {
                        for (i, label) in [
                            (Interface::Light, "Light color"),
                            (Interface::Dark, "Dark color"),
                        ] {
                            if ui.selectable_label(cur == i, label).clicked() {
                                app.set_interface(i);
                            }
                        }
                    });
            });
            ui.add_space(4.0);
            ui.label(format!(
                "Settings folder: {}",
                crate::settings::config_dir().display()
            ));
        });
    app.ui.prefs_open = open;
}

fn about(app: &mut App, ctx: &egui::Context) {
    if !app.ui.about_open {
        return;
    }
    let mut open = true;
    egui::Window::new("About Brushye")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading(format!("Brushye {}", env!("CARGO_PKG_VERSION")));
            ui.label("A free, open-source painting, illustration and comics app.");
            ui.label("License: MIT OR Apache-2.0");
            ui.label("Fonts: Go fonts (BSD-3-Clause). Icons: Phosphor Icons (MIT).");
        });
    app.ui.about_open = open;
}

fn import_brush(app: &mut App, ctx: &egui::Context) {
    let Some(mut path) = app.ui.import_brush.take() else {
        return;
    };
    let mut open = true;
    let mut done = false;
    egui::Window::new("Import sub tool")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("Brush file (.brshbrush)");
            ui.add(egui::TextEdit::singleline(&mut path).desired_width(360.0));
            if ui.button("Import").clicked() {
                app.import_sub_tool(&path);
                done = true;
            }
        });
    if open && !done {
        app.ui.import_brush = Some(path);
    }
}
