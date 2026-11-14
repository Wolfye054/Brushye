//! Journal window (M9, Brushye-only: CSP has no command journal). Lists the session's journal
//! with stroke samples summarised and undone steps greyed, as `brushye journal show` does.
//! Reached from Window > Journal (Brushye), next to Auto action.

use eframe::egui::{self, RichText};

use crate::app::App;
use brushye_command::journal::{self, JournalView};

/// Cached view, keyed by the journal length and the current document version, so the journal
/// text is only rebuilt when a command ran or the session was replaced.
#[derive(Default)]
pub struct JournalWindow {
    pub open: bool,
    cache: Option<((usize, usize), JournalView)>,
}

pub fn window(app: &mut App, ctx: &egui::Context) {
    if !app.ui.journal.open {
        return;
    }
    let key = (
        app.session.journal_len(),
        app.session
            .document()
            .map_or(0, |d| std::sync::Arc::as_ptr(&d) as usize),
    );
    if app.ui.journal.cache.as_ref().map(|c| c.0) != Some(key) {
        let v = journal::view(&app.session.journal_text());
        app.ui.journal.cache = Some((key, v));
    }
    let c = app.ui.colors();
    let mut open = true;
    let mut copy = false;
    egui::Window::new("Journal")
        .open(&mut open)
        .default_pos([700.0, 120.0])
        .default_size([560.0, 420.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            let Some((_, v)) = &app.ui.journal.cache else {
                return;
            };
            let undone = v.entries.iter().filter(|e| e.undone).count();
            ui.horizontal(|ui| {
                ui.label(format!("{} commands, {} undone", v.entries.len(), undone));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    copy = ui
                        .button("Copy")
                        .on_hover_text("Copy the full journal text")
                        .clicked();
                });
            });
            ui.separator();
            egui::ScrollArea::both()
                .id_salt("journal")
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for e in &v.entries {
                        let color = if e.undone { c.text_disabled } else { c.text };
                        let label = egui::Label::new(
                            RichText::new(&e.text).monospace().size(11.0).color(color),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend);
                        let r = ui.add(label);
                        if e.undone {
                            r.on_hover_text("Undone");
                        }
                    }
                });
        });
    if copy {
        ctx.copy_text(app.session.journal_text());
    }
    app.ui.journal.open = open;
}
