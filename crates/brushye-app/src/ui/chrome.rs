//! Canvas-column chrome (specs U-003, U-007, U-008): command bar, document tabs, status strip,
//! scroll bars, the collapsed timeline dock, and the icon column.

use eframe::egui::{self, Rect, Sense, Stroke, Ui, pos2, vec2};

use super::theme::{Colors, m};
use super::widgets;
use crate::app::{Action, App};
use crate::icons;

pub fn command_bar(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    widgets::fill(ui, r, c.bar);
    let can_undo = app.can_undo();
    let can_redo = app.can_redo();
    let ctx = ui.ctx().clone();
    widgets::area(ui, r.shrink2(vec2(4.0, 0.0)), |ui| {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let s = vec2(24.0, 24.0);
            if widgets::icon_button(ui, c, icons::FILE_PLUS, "New", s, true, false).clicked() {
                app.request(Action::New, &ctx);
            }
            if widgets::icon_button(ui, c, icons::FOLDER_OPEN, "Open", s, true, false).clicked() {
                app.request(Action::Open, &ctx);
            }
            if widgets::icon_button(ui, c, icons::FLOPPY_DISK, "Save", s, true, false).clicked() {
                app.save();
            }
            widgets::bar_separator(ui, c, 24.0);
            if widgets::icon_button(ui, c, icons::ARROW_U_UP_LEFT, "Undo", s, can_undo, false)
                .clicked()
            {
                app.undo();
            }
            if widgets::icon_button(ui, c, icons::ARROW_U_UP_RIGHT, "Redo", s, can_redo, false)
                .clicked()
            {
                app.redo();
            }
            widgets::bar_separator(ui, c, 24.0);
            if widgets::icon_button(
                ui,
                c,
                icons::X_SQUARE,
                "Delete (clear the layer)",
                s,
                app.can_edit_layer(),
                false,
            )
            .clicked()
            {
                app.clear_layer();
            }
            widgets::icon_button(
                ui,
                c,
                icons::SELECTION_SLASH,
                "Delete outside selection",
                s,
                false,
                false,
            );
            if widgets::icon_button(
                ui,
                c,
                icons::PAINT_BUCKET,
                "Fill (the layer with the drawing colour)",
                s,
                app.can_edit_layer(),
                false,
            )
            .clicked()
            {
                app.fill_layer();
            }
            widgets::icon_button(ui, c, icons::BOUNDING_BOX, "Scale/Rotate", s, false, false);
            widgets::bar_separator(ui, c, 24.0);
            widgets::icon_button(ui, c, icons::SELECTION, "Deselect", s, false, false);
            widgets::icon_button(
                ui,
                c,
                icons::SELECTION_INVERSE,
                "Invert selected area",
                s,
                false,
                false,
            );
            widgets::icon_button(
                ui,
                c,
                icons::SELECTION_FOREGROUND,
                "Show border of selected area",
                s,
                false,
                false,
            );
            widgets::bar_separator(ui, c, 24.0);
            widgets::icon_button(ui, c, icons::RULER, "Snap to ruler", s, false, false);
            widgets::icon_button(
                ui,
                c,
                icons::MAGNET,
                "Snap to special ruler",
                s,
                false,
                false,
            );
            widgets::icon_button(ui, c, icons::SQUARES_FOUR, "Snap to grid", s, false, false);
            widgets::bar_separator(ui, c, 24.0);
            if widgets::icon_button(ui, c, icons::QUESTION, "About Brushye", s, true, false)
                .clicked()
            {
                app.ui.about_open = true;
            }
        });
    });
}

pub fn doc_tabs(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    widgets::fill(ui, r, c.header);
    let name = app.doc_name();
    let label = format!("{name}{}", if app.dirty() { "*" } else { "" });
    let font = egui::FontId::proportional(11.0);
    let tw = ui.fonts_mut(|f| {
        f.layout_no_wrap(label.clone(), font.clone(), c.text)
            .size()
            .x
    });
    let tab = Rect::from_min_size(
        r.min + vec2(2.0, 0.0),
        vec2((tw + 24.0).clamp(99.0, 260.0), r.height()),
    );
    widgets::fill(ui, tab, c.accent);
    ui.painter().text(
        pos2(tab.left() + 6.0, tab.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        c.text_on_accent,
    );
    if app.dirty() {
        ui.painter().circle_filled(
            pos2(tab.right() - 8.0, tab.center().y),
            2.5,
            c.text_on_accent,
        );
    }
}

/// A wedge-shaped value slider (status strip, navigator): returns the new fraction if dragged.
pub fn wedge(ui: &mut Ui, c: &Colors, r: Rect, t: f32, id: &str) -> Option<f32> {
    let resp = ui.interact(r, ui.scope_id().with(id), Sense::click_and_drag());
    let p = ui.painter();
    p.add(egui::Shape::convex_polygon(
        vec![r.left_bottom(), r.right_top(), r.right_bottom()],
        c.slider_fill,
        Stroke::NONE,
    ));
    let x = r.left() + r.width() * t.clamp(0.0, 1.0);
    p.line_segment(
        [pos2(x, r.top()), pos2(x, r.bottom())],
        Stroke::new(2.0, c.text),
    );
    if resp.dragged() || resp.clicked() {
        resp.interact_pointer_pos()
            .map(|q| ((q.x - r.left()) / r.width()).clamp(0.0, 1.0))
    } else {
        None
    }
}

/// Zoom as a 0..1 slider position (log scale, 1 % … 6400 %).
pub fn zoom_to_t(z: f64) -> f32 {
    ((z.ln() - 0.01f64.ln()) / (64f64.ln() - 0.01f64.ln())) as f32
}
pub fn t_to_zoom(t: f32) -> f64 {
    (0.01f64.ln() + f64::from(t) * (64f64.ln() - 0.01f64.ln())).exp()
}

pub fn status_strip(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    widgets::fill(ui, r, c.status);
    let font = egui::FontId::proportional(10.0);
    let mut x = r.left() + 2.0;
    let field = |ui: &mut Ui, x: f32, text: String| {
        let fr = Rect::from_min_size(pos2(x, r.top()), vec2(54.0, r.height()));
        widgets::fill(ui, fr, c.value_field);
        ui.painter().text(
            pos2(fr.left() + 3.0, fr.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            font.clone(),
            c.text,
        );
        fr
    };
    let center = app.canvas_center();
    // Zoom field, wedge, −, +, fit.
    let fr = field(ui, x, format!("{:.1}", app.view.zoom * 100.0));
    x = fr.right() + 2.0;
    if let Some(t) = wedge(
        ui,
        c,
        Rect::from_min_size(pos2(x, r.top() + 1.0), vec2(48.0, r.height() - 2.0)),
        zoom_to_t(app.view.zoom),
        "zoom-wedge",
    ) {
        app.view.set_zoom(t_to_zoom(t), center);
    }
    x += 52.0;
    let small = vec2(16.0, r.height());
    let btn = |ui: &mut Ui, x: &mut f32, icon: &str, tip: &str| {
        let br = Rect::from_min_size(pos2(*x, r.top()), small);
        *x += 17.0;
        widgets::area(ui, br, |ui| {
            widgets::icon_button(ui, c, icon, tip, small, true, false).clicked()
        })
    };
    if btn(ui, &mut x, icons::MINUS, "Zoom out") {
        app.view.step_zoom(-1, center);
    }
    if btn(ui, &mut x, icons::PLUS, "Zoom in") {
        app.view.step_zoom(1, center);
    }
    if btn(ui, &mut x, icons::SQUARE, "100 %") {
        app.actual_size();
    }
    x += 6.0;
    // Rotation field, wedge, rotate left/right, reset.
    let fr = field(ui, x, format!("{:.1}", signed_angle(app.view.angle)));
    x = fr.right() + 2.0;
    let t = ((signed_angle(app.view.angle) + 180.0) / 360.0) as f32;
    if let Some(t) = wedge(
        ui,
        c,
        Rect::from_min_size(pos2(x, r.top() + 1.0), vec2(48.0, r.height() - 2.0)),
        t,
        "rot-wedge",
    ) {
        let target = f64::from(t) * 360.0 - 180.0;
        let a = app.view.angle;
        app.view.rotate_by(target - signed_angle(a), center);
    }
    x += 52.0;
    if btn(ui, &mut x, icons::ARROW_ARC_LEFT, "Rotate left") {
        app.view.rotate_by(-15.0, center);
    }
    if btn(ui, &mut x, icons::ARROW_ARC_RIGHT, "Rotate right") {
        app.view.rotate_by(15.0, center);
    }
    if btn(ui, &mut x, icons::ARROW_COUNTER_CLOCKWISE, "Reset rotation") {
        let a = app.view.angle;
        app.view.rotate_by(-a, center);
    }
    x += 4.0;
    // AI access indicator (D-022), at the right end while File > Allow AI clients is on.
    let mut right = r.right();
    if let Some(n) = app.mcp.as_ref().map(|l| l.connected()) {
        let w = 190.0;
        right -= w;
        let ir = Rect::from_min_max(pos2(right, r.top()), r.max);
        widgets::fill(ui, ir, c.status);
        let (icon, text) = if n > 0 {
            (icons::PLUGS_CONNECTED, "AI client connected")
        } else {
            (icons::PLUGS, "AI clients allowed")
        };
        let mut off = false;
        widgets::area(ui, ir, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(egui::RichText::new(icon).size(12.0).color(c.text));
                ui.label(egui::RichText::new(text).size(10.0).color(c.text));
                off = ui
                    .small_button(if n > 0 { "Disconnect" } else { "Turn off" })
                    .on_hover_text(
                        "Stop AI clients from using this window (File > Allow AI clients)",
                    )
                    .clicked();
            });
        });
        if off {
            let ctx = ui.ctx().clone();
            app.set_mcp(false, &ctx);
        }
    }
    if app.ui.visible.scroll_bars {
        h_scroll(
            app,
            ui,
            Rect::from_min_max(pos2(x, r.top()), pos2(right, r.bottom())),
        );
    }
}

/// Angle in -180..180.
pub fn signed_angle(a: f64) -> f64 {
    let a = a.rem_euclid(360.0);
    if a > 180.0 { a - 360.0 } else { a }
}

/// Canvas bounds on screen (physical px) and the viewport, along one axis.
fn extent(app: &App, axis: usize) -> Option<(f64, f64, f64, f64)> {
    let d = app.doc()?;
    let corners = [
        [0.0, 0.0],
        [f64::from(d.width), 0.0],
        [0.0, f64::from(d.height)],
        [f64::from(d.width), f64::from(d.height)],
    ];
    let pts: Vec<[f64; 2]> = corners.iter().map(|&p| app.view.to_screen(p)).collect();
    let lo = pts.iter().map(|p| p[axis]).fold(f64::MAX, f64::min);
    let hi = pts.iter().map(|p| p[axis]).fold(f64::MIN, f64::max);
    let (min, size) = app.canvas_px();
    Some((lo, hi, min[axis], min[axis] + size[axis]))
}

fn scroll(app: &mut App, ui: &mut Ui, r: Rect, axis: usize) {
    let c = app.ui.colors();
    widgets::fill(ui, r, c.status);
    let Some((lo, hi, v0, v1)) = extent(app, axis) else {
        return;
    };
    // Content = canvas plus half a viewport of margin on each side (so it can scroll away).
    let margin = (v1 - v0) / 2.0;
    let (c0, c1) = (lo.min(v0) - margin, hi.max(v1) + margin);
    let len = if axis == 0 { r.width() } else { r.height() };
    let start = if axis == 0 { r.left() } else { r.top() };
    let k = f64::from(len) / (c1 - c0);
    let a = start + ((v0 - c0) * k) as f32;
    let b = start + ((v1 - c0) * k) as f32;
    let thumb = if axis == 0 {
        Rect::from_min_max(pos2(a, r.top() + 2.0), pos2(b, r.bottom() - 2.0))
    } else {
        Rect::from_min_max(pos2(r.left() + 2.0, a), pos2(r.right() - 2.0, b))
    };
    ui.painter().rect_filled(thumb, 0.0, c.scroll_track);
    let resp = ui.interact(thumb, ui.scope_id().with(("scroll", axis)), Sense::drag());
    if resp.dragged() {
        let d = resp.drag_delta();
        let delta = f64::from(if axis == 0 { d.x } else { d.y }) / k;
        let ppp = f64::from(ui.ctx().pixels_per_point());
        let mut pan = [0.0, 0.0];
        pan[axis] = -delta * ppp;
        app.view.pan_by(pan);
    }
}

pub fn h_scroll(app: &mut App, ui: &mut Ui, r: Rect) {
    scroll(app, ui, r, 0);
}

pub fn v_scroll(app: &mut App, ui: &mut Ui, r: Rect) {
    scroll(app, ui, r, 1);
}

pub fn timeline_dock(ui: &mut Ui, c: &Colors, r: Rect) {
    widgets::fill(ui, r, c.header);
    widgets::draw_icon(
        ui,
        pos2(r.left() + 9.0, r.top() + 10.0),
        icons::CARET_DOUBLE_UP,
        9.0,
        c.icon_dim,
    );
    widgets::draw_icon(
        ui,
        pos2(r.left() + 9.0, r.bottom() - 10.0),
        icons::CARET_DOUBLE_DOWN,
        9.0,
        c.icon_dim,
    );
    let br = Rect::from_min_size(pos2(r.left() + 22.0, r.top() + 4.0), vec2(30.0, 30.0));
    widgets::area(ui, br, |ui| {
        widgets::icon_button(
            ui,
            c,
            icons::FILM_STRIP,
            "Timeline",
            vec2(30.0, 30.0),
            false,
            false,
        );
    });
}

/// Collapsed palettes as icon buttons: Quick Access and the material palettes (U-001). They're
/// CSP-ecosystem features Brushye doesn't have; their slots are kept, disabled.
pub fn icon_column(ui: &mut Ui, c: &Colors, r: Rect) {
    widgets::fill(ui, r, c.body);
    let header = Rect::from_min_size(r.min, vec2(r.width(), m::DOCK_HEADER));
    widgets::fill(ui, header, c.header);
    let items = [
        (icons::LIGHTNING, "Quick access"),
        (icons::CHECKERBOARD, "Material: colour pattern"),
        (icons::IMAGE, "Material: background"),
        (icons::SQUARE_HALF, "Material: monochrome pattern"),
        (icons::STACK, "Material: basic"),
        (icons::STAR, "Material: manga"),
        (icons::LINE_SEGMENTS, "Material: effect lines"),
        (icons::IMAGE_SQUARE, "Material: images"),
        (icons::PAINT_BRUSH_BROAD, "Material: illustration"),
        (icons::CUBE, "Material: 3D"),
        (icons::PERSON, "Material: pose"),
    ];
    for (i, (icon, tip)) in items.iter().enumerate() {
        let br = Rect::from_min_size(
            pos2(r.left() + 4.0, header.bottom() + 2.0 + i as f32 * 36.0),
            vec2(32.0, 32.0),
        );
        ui.painter().rect_stroke(
            br,
            0.0,
            Stroke::new(1.0, c.splitter),
            egui::StrokeKind::Inside,
        );
        widgets::area(ui, br, |ui| {
            widgets::icon_button(ui, c, icon, tip, vec2(32.0, 32.0), false, false);
        });
    }
}
