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

/// Whether to use the OS file dialogs (D-019). `BRUSHYE_INAPP_DIALOGS=1` keeps the in-app path
/// field instead (for test displays without a desktop portal).
pub fn native_dialogs() -> bool {
    std::env::var_os("BRUSHYE_INAPP_DIALOGS").is_none()
}

/// A native dialog starting at `path` (its folder and file name), filtered to one extension.
fn file_dialog(path: &str, filter: &str, ext: &str) -> rfd::FileDialog {
    let p = std::path::Path::new(path.trim());
    let mut d = rfd::FileDialog::new().add_filter(filter, &[ext]);
    if let Some(dir) = p.parent().filter(|d| d.is_dir()) {
        d = d.set_directory(dir);
    }
    if let Some(name) = p.file_name() {
        d = d.set_file_name(name.to_string_lossy());
    }
    d
}

/// Show the OS file dialog for a file dialog that was just requested, then do what the in-app
/// dialog's OK button does. Cancelling does nothing. Runs on the UI thread (macOS needs that).
pub fn native(app: &mut App) {
    if !native_dialogs() {
        return;
    }
    let path_dialog = matches!(
        app.dialog,
        Some(
            Dialog::Open { .. }
                | Dialog::SaveAs { .. }
                | Dialog::SaveDuplicate { .. }
                | Dialog::ExportPng { .. }
        )
    );
    if path_dialog {
        match app.dialog.take() {
            Some(Dialog::Open { path }) => {
                if let Some(p) = file_dialog(&path, "Brushye file", "brsh")
                    .set_title("Open")
                    .pick_file()
                {
                    app.open_path(&p.to_string_lossy());
                }
            }
            Some(Dialog::SaveAs { path, .. }) => {
                if let Some(p) = file_dialog(&path, "Brushye file", "brsh")
                    .set_title("Save as")
                    .save_file()
                {
                    app.save_to(path_with_ext(&p.to_string_lossy(), "brsh"));
                }
            }
            Some(Dialog::SaveDuplicate { path }) => {
                if let Some(p) = file_dialog(&path, "Brushye file", "brsh")
                    .set_title("Save duplicate")
                    .save_file()
                {
                    app.save_duplicate(path_with_ext(&p.to_string_lossy(), "brsh"));
                }
            }
            Some(Dialog::ExportPng { path }) => {
                if let Some(p) = file_dialog(&path, "PNG image", "png")
                    .set_title("Export (.png)")
                    .save_file()
                {
                    let p = path_with_ext(&p.to_string_lossy(), "png");
                    app.export_png(&p.to_string_lossy());
                }
            }
            other => app.dialog = other,
        }
    }
    if let Some(path) = app.ui.import_brush.take() {
        let dir = crate::settings::config_dir().join("brushes");
        let start = if path.is_empty() {
            dir.join("brush.brshbrush").to_string_lossy().into_owned()
        } else {
            path
        };
        if let Some(p) = file_dialog(&start, "Brushye brush", "brshbrush")
            .set_title("Import sub tool")
            .pick_file()
        {
            app.import_sub_tool(&p.to_string_lossy());
        }
    }
    if let Some(super::left::SubToolDialog::Export { path }) = &app.ui.subtool_dialog {
        let path = path.clone();
        app.ui.subtool_dialog = None;
        if let Some(p) = file_dialog(&path, "Brushye brush", "brshbrush")
            .set_title("Export sub tool")
            .save_file()
        {
            let p = path_with_ext(&p.to_string_lossy(), "brshbrush");
            app.status = super::left::export_sub_tool(&app.ui, &p.to_string_lossy());
        }
    }
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
