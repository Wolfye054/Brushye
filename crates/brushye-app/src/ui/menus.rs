//! The menu bar (spec U-002): CSP 1.13.2's nine menus in order, with Brushye's features in the
//! same menu and group as CSP's item. Items Brushye doesn't have yet are shown disabled in their
//! place; CSP-ecosystem items (CLIP STUDIO launcher, TABMATE, QUMARION, smartphone) are left out.
//! Shortcuts follow CSP's where the feature exists.

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers as M, Ui};

use super::tools::ToolId;
use crate::app::{Action, App, Dialog};

fn item(ui: &mut Ui, label: &str, shortcut: &str, enabled: bool) -> bool {
    let b = egui::Button::new(label).shortcut_text(shortcut);
    let r = ui.add_enabled(enabled, b);
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

fn check(ui: &mut Ui, label: &str, shortcut: &str, checked: bool, enabled: bool) -> bool {
    let b = egui::Button::selectable(checked, label).shortcut_text(shortcut);
    let r = ui.add_enabled(enabled, b);
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

fn off(ui: &mut Ui, labels: &[&str]) {
    for l in labels {
        item(ui, l, "", false);
    }
}

fn sub(ui: &mut Ui, label: &str, enabled: bool, f: impl FnOnce(&mut Ui)) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.menu_button(label, f);
    });
}

pub fn menu_bar(app: &mut App, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    // CSP spaces its menu titles widely (U-002).
    ui.spacing_mut().button_padding = egui::vec2(8.0, 2.0);
    egui::MenuBar::new().ui(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        ui.menu_button("File", |ui| file_menu(app, ui, &ctx));
        ui.menu_button("Edit", |ui| edit_menu(app, ui));
        ui.menu_button("Animation", animation_menu);
        ui.menu_button("Layer", |ui| layer_menu(app, ui));
        ui.menu_button("Select", select_menu);
        ui.menu_button("View", |ui| view_menu(app, ui));
        ui.menu_button("Filter", filter_menu);
        ui.menu_button("Window", |ui| window_menu(app, ui));
        ui.menu_button("Help", |ui| {
            off(
                ui,
                &[
                    "Tutorials",
                    "Getting started…",
                    "Instruction manual",
                    "Support",
                ],
            );
            ui.separator();
            if item(ui, "About Brushye…", "", true) || item(ui, "Version information…", "", true)
            {
                app.ui.about_open = true;
            }
        });
    });
}

fn file_menu(app: &mut App, ui: &mut Ui, ctx: &egui::Context) {
    if item(ui, "New…", "Ctrl+N", true) {
        app.request(Action::New, ctx);
    }
    item(ui, "Create new from clipboard", "", false);
    if item(ui, "Open…", "Ctrl+O", true) {
        app.request(Action::Open, ctx);
    }
    let recent = app.settings.recent.clone();
    sub(ui, "Recent", !recent.is_empty(), |ui| {
        for (i, p) in recent.iter().enumerate() {
            if item(ui, &format!("{} {p}", i + 1), "", true) {
                app.request(Action::OpenPath(p.clone()), ctx);
            }
        }
    });
    ui.separator();
    if item(ui, "Close", "Ctrl+W", true) {
        app.request(Action::Close, ctx);
    }
    if item(ui, "Save", "Ctrl+S", true) {
        app.save();
    }
    if item(ui, "Save as…", "Shift+Alt+S", true) {
        app.dialog = Some(app.save_as_dialog());
    }
    sub(ui, "Save duplicate", true, |ui| {
        if item(ui, ".brsh (Brushye)", "", true) {
            app.dialog = Some(Dialog::SaveDuplicate {
                path: app.suggest_path("copy.brsh"),
            });
        }
        if item(ui, ".png", "", true) {
            app.dialog = Some(Dialog::ExportPng {
                path: app.suggest_path("png"),
            });
        }
        off(ui, &[".jpg", ".psd"]);
    });
    if item(ui, "Revert…", "", app.path.is_some() && app.dirty()) {
        app.request(Action::Revert, ctx);
    }
    ui.separator();
    sub(ui, "Export (single layer)", true, |ui| {
        if item(ui, ".png", "", true) {
            app.dialog = Some(Dialog::ExportPng {
                path: app.suggest_path("png"),
            });
        }
        off(ui, &[".bmp", ".jpg", ".tif", ".psd"]);
    });
    sub(ui, "Export animation", false, |_| {});
    off(ui, &["Export webtoon…", "Export vectors…"]);
    ui.separator();
    sub(ui, "Timelapse", false, |_| {});
    ui.separator();
    sub(ui, "Import", false, |_| {});
    ui.separator();
    off(ui, &["Print settings…", "Print…"]);
    ui.separator();
    if item(ui, "Preferences…", "Ctrl+K", true) {
        app.ui.prefs_open = true;
    }
    off(
        ui,
        &[
            "Command bar settings…",
            "Shortcut settings…",
            "Modifier key settings…",
        ],
    );
    ui.separator();
    item(ui, "Manage fonts…", "", false);
    ui.separator();
    if item(ui, "Pen pressure settings…", "", true) {
        app.dialog = Some(Dialog::PenPressure {
            points: app.settings.pressure_curve.clone(),
        });
    }
    ui.separator();
    if item(ui, "Quit Brushye", "Ctrl+Q", true) {
        app.request(Action::Quit, ctx);
    }
}

fn edit_menu(app: &mut App, ui: &mut Ui) {
    if item(ui, "Undo", "Ctrl+Z", app.can_undo()) {
        app.undo();
    }
    if item(ui, "Redo", "Ctrl+Y", app.can_redo()) {
        app.redo();
    }
    ui.separator();
    off(
        ui,
        &[
            "Cut",
            "Copy",
            "Copy vectors as SVG",
            "Paste",
            "Paste to shown position",
        ],
    );
    ui.separator();
    if item(ui, "Delete", "Del", app.can_edit_layer()) {
        app.clear_layer();
    }
    item(ui, "Delete outside selection", "Shift+Del", false);
    item(ui, "Smart smoothing…", "", false);
    if item(ui, "Fill", "Alt+Del", app.can_edit_layer()) {
        app.fill_layer();
    }
    off(ui, &["Advanced fill…"]);
    sub(ui, "Colorize", false, |_| {});
    off(
        ui,
        &[
            "Convert to drawing color",
            "Outline selection…",
            "Convert brightness to opacity",
        ],
    );
    ui.separator();
    sub(ui, "Register material", false, |_| {});
    sub(ui, "Tonal correction", false, |_| {});
    sub(ui, "Transform", false, |_| {});
    ui.separator();
    off(
        ui,
        &["Change image resolution…", "Change canvas size…", "Crop"],
    );
    sub(ui, "Rotate/Flip canvas", false, |_| {});
    item(ui, "Canvas properties…", "", false);
    ui.separator();
    sub(ui, "Clear memory", false, |_| {});
    ui.separator();
    off(
        ui,
        &["Pick screen color…", "Hide windows and pick screen color…"],
    );
}

fn animation_menu(ui: &mut Ui) {
    sub(ui, "New animation layer", false, |_| {});
    item(ui, "New animation cel", "", false);
    sub(ui, "Edit track", false, |_| {});
    sub(ui, "Animation curve", false, |_| {});
    ui.separator();
    item(ui, "Play/Stop", "", false);
    sub(ui, "Playback settings", false, |_| {});
    sub(ui, "Move frame", false, |_| {});
    ui.separator();
    sub(ui, "Label", false, |_| {});
    sub(ui, "Timeline", false, |_| {});
    ui.separator();
    sub(ui, "Show animation cels", false, |_| {});
    sub(ui, "Light table", false, |_| {});
    off(
        ui,
        &[
            "Show 2D camera frames",
            "Show camera path",
            "Show all field guides",
        ],
    );
}

/// The Layer menu's first four groups (also the layer list's right-click menu, U-006).
pub fn layer_context_items(app: &mut App, ui: &mut Ui) {
    let editable = app.can_edit_layer();
    let has_layer = app.session.active_layer().is_some();
    if item(ui, "New raster layer", "Ctrl+Shift+N", true) {
        app.new_raster_layer();
    }
    sub(ui, "New layer", true, |ui| {
        if item(ui, "Raster layer…", "", true) {
            app.new_raster_layer();
        }
        off(
            ui,
            &[
                "Vector layer…",
                "Gradient",
                "Fill…",
                "Tone…",
                "Frame border folder…",
                "3D layer",
            ],
        );
        ui.separator();
        item(ui, "Paper…", "", false);
    });
    sub(ui, "New correction layer", false, |_| {});
    off(
        ui,
        &[
            "New layer folder",
            "Create folder and insert layer",
            "Ungroup layer folder",
            "Duplicate layer",
        ],
    );
    if item(ui, "Delete layer", "", editable) {
        app.delete_layer();
    }
    ui.separator();
    sub(ui, "Layer mask", false, |_| {});
    sub(ui, "Ruler/Frame", false, |_| {});
    sub(ui, "File object", false, |_| {});
    sub(ui, "Layer settings", has_layer, |ui| {
        off(
            ui,
            &[
                "Set as reference layer",
                "Set as draft layer",
                "Clip to layer below",
            ],
        );
        let (locked, lock_alpha, visible) = app.active_flags();
        if check(ui, "Lock layer", "", locked, true) {
            app.toggle_lock(false);
        }
        if check(ui, "Lock transparent pixels", "", lock_alpha, !locked) {
            app.toggle_lock(true);
        }
        ui.separator();
        if check(ui, "Show layer", "", visible, true) {
            app.toggle_visible();
        }
        if item(ui, "Change layer name…", "", !locked) {
            app.start_rename();
        }
        ui.separator();
        off(
            ui,
            &[
                "Display palette color in parent folder",
                "Display palette color on canvas",
            ],
        );
        ui.separator();
        item(ui, "Advanced settings…", "", false);
    });
    sub(ui, "Selection from layer", false, |_| {});
    off(ui, &["Rasterize", "Convert layer…"]);
    ui.separator();
    off(
        ui,
        &[
            "Transfer to lower layer",
            "Merge with layer below",
            "Merge selected layers",
            "Merge visible layers",
            "Merge visible to new layer",
            "Flatten image",
        ],
    );
}

fn layer_menu(app: &mut App, ui: &mut Ui) {
    layer_context_items(app, ui);
    ui.separator();
    let has_layer = app.session.active_layer().is_some();
    sub(ui, "Layer order", has_layer, |ui| {
        for (label, which) in [("Top", 0), ("Up", 1), ("Down", 2), ("Bottom", 3)] {
            if item(ui, label, "", true) {
                app.move_active(which);
            }
        }
    });
    sub(ui, "Change selected layer", has_layer, |ui| {
        if item(ui, "Layer above", "Alt+]", true) {
            app.select_neighbour(true);
        }
        if item(ui, "Layer below", "Alt+[", true) {
            app.select_neighbour(false);
        }
    });
}

fn select_menu(ui: &mut Ui) {
    off(
        ui,
        &["Select all", "Deselect", "Reselect", "Invert selected area"],
    );
    ui.separator();
    off(
        ui,
        &[
            "Expand selected area…",
            "Shrink selected area…",
            "Blur border…",
        ],
    );
    ui.separator();
    item(ui, "Select color gamut…", "", false);
    ui.separator();
    off(
        ui,
        &["Select overlapping vectors", "Select vectors within area"],
    );
    ui.separator();
    off(
        ui,
        &[
            "Quick mask",
            "Convert to selection layer",
            "Convert selection layer to selection",
        ],
    );
}

fn view_menu(app: &mut App, ui: &mut Ui) {
    let c = app.canvas_center();
    sub(ui, "Rotate/Flip", true, |ui| {
        if item(ui, "Reset rotation/inversion", "", true) || item(ui, "Reset rotation", "", true) {
            let a = app.view.angle;
            app.view.rotate_by(-a, c);
        }
        for (label, deg) in [
            ("Rotate 90", 90.0),
            ("Rotate 180", 180.0),
            ("Rotate 270", 270.0),
        ] {
            if item(ui, label, "", true) {
                let a = app.view.angle;
                app.view.rotate_by(deg - a, c);
            }
        }
        ui.separator();
        off(ui, &["Flip horizontal", "Flip vertical"]);
        ui.separator();
        if item(ui, "Rotate left", "-", true) {
            app.view.rotate_by(-15.0, c);
        }
        if item(ui, "Rotate right", "^", true) {
            app.view.rotate_by(15.0, c);
        }
    });
    if item(ui, "Zoom in", "Ctrl+Num+", true) {
        app.view.step_zoom(1, c);
    }
    if item(ui, "Zoom out", "Ctrl+Num-", true) {
        app.view.step_zoom(-1, c);
    }
    if item(ui, "100%", "Ctrl+Alt+0", true) {
        app.actual_size();
    }
    if item(ui, "200%", "", true) {
        app.actual_size();
        app.view.set_zoom(2.0, c);
    }
    if item(ui, "Fit to screen", "Ctrl+0", true) {
        app.fit();
    }
    if item(ui, "Reset display", "Ctrl+`", true) {
        app.fit();
    }
    if item(ui, "Fit to navigator", "", true) {
        app.fit();
    }
    item(ui, "Print size", "", false);
    ui.separator();
    let sb = app.ui.visible.scroll_bars;
    if check(ui, "Scroll bar", "", sb, true) {
        app.ui.visible.scroll_bars = !sb;
    }
    off(
        ui,
        &[
            "Selection border",
            "Selection launcher",
            "Selection launcher settings…",
            "Transform launcher",
            "Text launcher",
            "Object launcher",
        ],
    );
    ui.separator();
    off(
        ui,
        &[
            "Grid",
            "Ruler bar",
            "Grid/Ruler bar settings…",
            "Crop mark/Default border",
            "Crop mark/Default border settings…",
            "Safety margin",
            "On-screen area (webtoon)",
            "On-screen area settings (webtoon)…",
            "Paper",
        ],
    );
    sub(ui, "Show tone area", false, |_| {});
    sub(ui, "Show vector paths", false, |_| {});
    ui.separator();
    item(ui, "Show playback fps", "", false);
    ui.separator();
    off(
        ui,
        &[
            "Snap to ruler",
            "Snap to special ruler",
            "Snap to grid",
            "Toggle special ruler snapping",
        ],
    );
    ui.separator();
    sub(ui, "Color profile", false, |_| {});
}

fn filter_menu(ui: &mut Ui) {
    for m in [
        "Blur",
        "Correction",
        "Distort",
        "Effect",
        "Render",
        "Sharpen",
    ] {
        sub(ui, m, false, |_| {});
    }
}

fn window_menu(app: &mut App, ui: &mut Ui) {
    sub(ui, "Canvas", false, |_| {});
    ui.separator();
    sub(ui, "Workspace", true, |ui| {
        if item(ui, "Reset to default", "", true) {
            app.ui.reset_layout();
        }
        ui.separator();
        off(ui, &["Register workspace…", "Manage workspace…"]);
    });
    ui.separator();
    let v = app.ui.visible;
    if check(ui, "Command bar", "", v.command_bar, true) {
        app.ui.visible.command_bar = !v.command_bar;
    }
    item(ui, "Quick access", "", false);
    ui.separator();
    if check(ui, "Tool", "", v.tool, true) {
        app.ui.visible.tool = !v.tool;
    }
    let tool = super::tools::info(app.ui.tool).name;
    let shown = |app: &App, key: &str, dock: bool| dock && !app.ui.hidden.contains(key);
    let toggle = |app: &mut App, key: &'static str, right: bool| {
        if right {
            app.ui.visible.right_dock = true;
        } else {
            app.ui.visible.left_dock = true;
        }
        if !app.ui.hidden.remove(key) {
            app.ui.hidden.insert(key);
        }
    };
    for (label, key) in [
        (format!("Sub Tool: {tool}"), "sub_tool"),
        (format!("Tool property: {tool}"), "tool_property"),
        (format!("Brush size: {tool}"), "brush_size"),
    ] {
        if check(ui, &label, "", shown(app, key, v.left_dock), true) {
            toggle(app, key, false);
        }
    }
    let detail = app.ui.detail_open;
    if check(ui, "Sub Tool Detail", "", detail, app.ui.tool.is_brush()) {
        app.ui.detail_open = !detail;
    }
    ui.separator();
    for (i, label) in [
        "Color Wheel",
        "Color Set",
        "Color Slider",
        "Color Mixing",
        "Color History",
        "Approximate Color",
        "Intermediate Color",
    ]
    .iter()
    .enumerate()
    {
        let available = !matches!(i, 3 | 6);
        if check(
            ui,
            label,
            "",
            shown(app, "colour", v.left_dock) && app.ui.colour.tab == i,
            available,
        ) {
            app.ui.visible.left_dock = true;
            app.ui.hidden.remove("colour");
            app.ui.colour.tab = i;
        }
    }
    ui.separator();
    if check(
        ui,
        "Layer",
        "",
        shown(app, "layer", v.right_dock) && app.ui.layer_tab == 0,
        true,
    ) {
        app.ui.visible.right_dock = true;
        app.ui.hidden.remove("layer");
        app.ui.layer_tab = 0;
    }
    item(ui, "Search layer", "", false);
    if check(
        ui,
        "Layer property",
        "",
        shown(app, "layer_property", v.right_dock),
        true,
    ) {
        toggle(app, "layer_property", true);
    }
    ui.separator();
    off(ui, &["Animation cels", "Timeline"]);
    ui.separator();
    if check(
        ui,
        "Navigator",
        "",
        shown(app, "navigator", v.right_dock) && app.ui.nav_tab == 0,
        true,
    ) {
        app.ui.visible.right_dock = true;
        app.ui.hidden.remove("navigator");
        app.ui.nav_tab = 0;
    }
    item(ui, "Sub view", "", false);
    ui.separator();
    if check(
        ui,
        "History",
        "",
        shown(app, "layer", v.right_dock) && app.ui.layer_tab == 1,
        true,
    ) {
        app.ui.visible.right_dock = true;
        app.ui.hidden.remove("layer");
        app.ui.layer_tab = 1;
    }
    item(ui, "Auto action", "", false);
    if check(
        ui,
        "Information",
        "",
        shown(app, "navigator", v.right_dock) && app.ui.nav_tab == 3,
        true,
    ) {
        app.ui.visible.right_dock = true;
        app.ui.hidden.remove("navigator");
        app.ui.nav_tab = 3;
    }
    item(ui, "Item bank", "", false);
    ui.separator();
    sub(ui, "Material", false, |_| {});
    ui.separator();
    if check(ui, "Hide all palettes", "Tab", v.hide_all, true) {
        app.ui.visible.hide_all = !v.hide_all;
    }
    off(ui, &["Hide title bar", "Always show tab in canvas"]);
    sub(ui, "Palette dock", false, |_| {});
}

fn shortcut(ctx: &egui::Context, mods: M, key: Key) -> bool {
    ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(mods, key)))
}

/// Keyboard shortcuts (CSP's where Brushye has the feature, U-002).
pub fn shortcuts(app: &mut App, ctx: &egui::Context) {
    if app.dialog.is_some() {
        return;
    }
    let cmd = M::COMMAND;
    if shortcut(ctx, cmd | M::SHIFT, Key::Z) || shortcut(ctx, cmd, Key::Y) {
        app.redo();
    }
    if shortcut(ctx, cmd, Key::Z) {
        app.undo();
    }
    if shortcut(ctx, M::SHIFT | M::ALT, Key::S) {
        app.dialog = Some(app.save_as_dialog());
    }
    if shortcut(ctx, cmd, Key::S) {
        app.save();
    }
    if shortcut(ctx, cmd | M::SHIFT, Key::N) {
        app.new_raster_layer();
    }
    if shortcut(ctx, cmd, Key::N) {
        app.request(Action::New, ctx);
    }
    if shortcut(ctx, cmd, Key::O) {
        app.request(Action::Open, ctx);
    }
    if shortcut(ctx, cmd, Key::W) {
        app.request(Action::Close, ctx);
    }
    if shortcut(ctx, cmd, Key::Q) {
        app.request(Action::Quit, ctx);
    }
    if shortcut(ctx, cmd, Key::K) {
        app.ui.prefs_open = true;
    }
    if shortcut(ctx, cmd | M::ALT, Key::Num0) {
        app.actual_size();
    }
    if shortcut(ctx, cmd, Key::Num0) || shortcut(ctx, cmd, Key::Backtick) {
        app.fit();
    }
    let c = app.canvas_center();
    if shortcut(ctx, cmd, Key::Plus) || shortcut(ctx, cmd, Key::Equals) {
        app.view.step_zoom(1, c);
    }
    if shortcut(ctx, cmd, Key::Minus) {
        app.view.step_zoom(-1, c);
    }
    if shortcut(ctx, M::ALT, Key::CloseBracket) {
        app.select_neighbour(true);
    }
    if shortcut(ctx, M::ALT, Key::OpenBracket) {
        app.select_neighbour(false);
    }
    if shortcut(ctx, M::ALT, Key::Delete) {
        app.fill_layer();
    }
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    if shortcut(ctx, M::NONE, Key::Delete) {
        app.clear_layer();
    }
    if shortcut(ctx, M::NONE, Key::Tab) {
        app.ui.visible.hide_all = !app.ui.visible.hide_all;
    }
    // Tool keys: pressing a key again cycles the tools that share it (CSP).
    let cycle = |app: &mut App, ids: &[ToolId]| {
        let i = ids
            .iter()
            .position(|&t| t == app.ui.tool)
            .map_or(0, |i| (i + 1) % ids.len());
        app.ui.tool = ids[i];
    };
    if shortcut(ctx, M::NONE, Key::P) {
        cycle(app, &[ToolId::Pen, ToolId::Pencil]);
    }
    if shortcut(ctx, M::NONE, Key::B) {
        cycle(app, &[ToolId::Brush, ToolId::Airbrush]);
    }
    // X switches the active chip between main and sub colour; it doesn't swap them (U-013).
    if shortcut(ctx, M::NONE, Key::X) {
        use super::colour::Chip;
        let next = if app.ui.colour.active == Chip::Main {
            Chip::Sub
        } else {
            Chip::Main
        };
        app.ui.colour.select(next);
    }
    if shortcut(ctx, M::NONE, Key::E) {
        app.ui.tool = ToolId::Eraser;
    }
    if shortcut(ctx, M::NONE, Key::I) {
        app.ui.tool = ToolId::Eyedropper;
    }
    if shortcut(ctx, M::NONE, Key::Slash) {
        app.ui.tool = ToolId::Zoom;
    }
    if shortcut(ctx, M::NONE, Key::H) {
        app.ui.tool = ToolId::Hand;
    }
    if shortcut(ctx, M::NONE, Key::OpenBracket) {
        app.scale_brush(1.0 / 1.2);
    }
    if shortcut(ctx, M::NONE, Key::CloseBracket) {
        app.scale_brush(1.2);
    }
}
