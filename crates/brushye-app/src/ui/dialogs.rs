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

/// What to do with the file a native dialog returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileJob {
    Open,
    SaveAs,
    SaveDuplicate,
    ExportPng,
    ImportBrush,
    ExportBrush,
}

/// A native file dialog that is open: its result arrives on `rx` from a waiting thread.
pub struct PendingFile {
    job: FileJob,
    rx: std::sync::mpsc::Receiver<Option<std::path::PathBuf>>,
}

/// A native dialog starting at `path` (its folder and file name), filtered to one extension.
fn file_dialog(path: &str, filter: &str, ext: &str, title: &str) -> rfd::AsyncFileDialog {
    file_dialog_exts(path, filter, &[ext], title)
}

fn file_dialog_exts(path: &str, filter: &str, exts: &[&str], title: &str) -> rfd::AsyncFileDialog {
    let p = std::path::Path::new(path.trim());
    let mut d = rfd::AsyncFileDialog::new()
        .add_filter(filter, exts)
        .set_title(title);
    if let Some(dir) = p.parent().filter(|d| d.is_dir()) {
        d = d.set_directory(dir);
    }
    if let Some(name) = p.file_name() {
        d = d.set_file_name(name.to_string_lossy());
    }
    d
}

/// Run a future to completion on the current thread (a waiting thread; no runtime needed).
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::park();
    }
}

/// The native dialog a just-made request asks for (taking the request), if any.
fn take_request(app: &mut App) -> Option<(FileJob, String)> {
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
        return match app.dialog.take() {
            Some(Dialog::Open { path }) => Some((FileJob::Open, path)),
            Some(Dialog::SaveAs { path, .. }) => Some((FileJob::SaveAs, path)),
            Some(Dialog::SaveDuplicate { path }) => Some((FileJob::SaveDuplicate, path)),
            Some(Dialog::ExportPng { path }) => Some((FileJob::ExportPng, path)),
            other => {
                app.dialog = other;
                None
            }
        };
    }
    if let Some(path) = app.ui.import_brush.take() {
        let path = if path.is_empty() {
            let dir = crate::settings::config_dir().join("brushes");
            dir.join("brush.brshbrush").to_string_lossy().into_owned()
        } else {
            path
        };
        return Some((FileJob::ImportBrush, path));
    }
    if let Some(super::left::SubToolDialog::Export { path }) = &app.ui.subtool_dialog {
        let path = path.clone();
        app.ui.subtool_dialog = None;
        return Some((FileJob::ExportBrush, path));
    }
    None
}

/// Do what the in-app dialog's OK button does, with the chosen file.
fn finish(app: &mut App, job: FileJob, p: &std::path::Path) {
    let p = p.to_string_lossy();
    match job {
        FileJob::Open => app.open_path(&p),
        FileJob::SaveAs => app.save_to(path_with_ext(&p, "brsh")),
        FileJob::SaveDuplicate => app.save_duplicate(path_with_ext(&p, "brsh")),
        FileJob::ExportPng => {
            let p = path_with_ext(&p, "png");
            app.export_png(&p.to_string_lossy());
        }
        FileJob::ImportBrush => app.import_sub_tool(&p),
        FileJob::ExportBrush => {
            let p = path_with_ext(&p, "brshbrush");
            app.status = super::left::export_sub_tool(&app.ui, &p.to_string_lossy());
        }
    }
}

/// OS file dialogs (D-019). A request made last frame opens the dialog, attached to the
/// window. The dialog is created here on the UI thread (macOS needs that) and awaited on
/// another thread, so the UI keeps running and the desktop doesn't report "not responding".
/// When the result arrives, this does what the in-app dialog's OK button does; cancelling does
/// nothing.
pub fn native(app: &mut App, frame: &eframe::Frame, ctx: &egui::Context) {
    if !native_dialogs() {
        return;
    }
    if let Some(p) = &app.file_dialog {
        match p.rx.try_recv() {
            Ok(result) => {
                let job = p.job;
                app.file_dialog = None;
                if let Some(path) = result {
                    finish(app, job, &path);
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // One dialog at a time: drop requests made while it is open.
                let _ = take_request(app);
                return;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => app.file_dialog = None,
        }
    }
    let Some((job, path)) = take_request(app) else {
        return;
    };
    let d = match job {
        FileJob::Open => file_dialog(&path, "Brushye file", "brsh", "Open"),
        FileJob::SaveAs => file_dialog(&path, "Brushye file", "brsh", "Save as"),
        FileJob::SaveDuplicate => file_dialog(&path, "Brushye file", "brsh", "Save duplicate"),
        FileJob::ExportPng => file_dialog(&path, "PNG image", "png", "Export (.png)"),
        FileJob::ImportBrush => file_dialog_exts(
            &path,
            "Brushes (Brushye, Clip Studio Paint)",
            &["brshbrush", "sut"],
            "Import sub tool",
        ),
        FileJob::ExportBrush => file_dialog(&path, "Brushye brush", "brshbrush", "Export sub tool"),
    }
    .set_parent(frame);
    let pick = matches!(job, FileJob::Open | FileJob::ImportBrush);
    type Pick =
        std::pin::Pin<Box<dyn std::future::Future<Output = Option<rfd::FileHandle>> + Send>>;
    let future: Pick = if pick {
        Box::pin(d.pick_file())
    } else {
        Box::pin(d.save_file())
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("file-dialog".into())
        .spawn(move || {
            let result = block_on(future).map(|h| h.path().to_path_buf());
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    match spawned {
        Ok(_) => app.file_dialog = Some(PendingFile { job, rx }),
        Err(e) => app.status = format!("Couldn't open the file dialog: {e}"),
    }
}

/// What a .sut import couldn't bring over exactly (M8).
fn sut_report(app: &mut App, ctx: &egui::Context) {
    let Some(report) = &app.ui.sut_report else {
        return;
    };
    let mut open = true;
    let mut ok = false;
    egui::Window::new("Import sub tool")
        .collapsible(false)
        .resizable(true)
        .default_width(520.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    for (name, notes) in report {
                        ui.strong(format!("“{name}” was added."));
                        if notes.is_empty() {
                            ui.label("Every setting was imported.");
                        }
                        for n in notes {
                            ui.label(format!("• {n}"));
                        }
                        ui.add_space(6.0);
                    }
                });
            if ui.button("OK").clicked() {
                ok = true;
            }
        });
    if !open || ok {
        app.ui.sut_report = None;
    }
}

pub fn dialogs(app: &mut App, ctx: &egui::Context) {
    sut_report(app, ctx);
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
            ui.label("Brush file (.brshbrush, or a Clip Studio Paint .sut)");
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
