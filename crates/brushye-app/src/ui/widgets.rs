//! Widgets drawn the way CSP 1.13.2 draws them (spec U-009): flat bars, square corners, icon
//! buttons with an accent background when on, palette title bars with tabs, and Tool property
//! style setting rows (label, thin value bar, number with spinner, dynamics button).

use eframe::egui::{self, Color32, FontId, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};

use super::theme::Colors;
use crate::icons;

/// Paint a flat rectangle.
pub fn fill(ui: &Ui, r: Rect, c: Color32) {
    ui.painter().rect_filled(r, 0.0, c);
}

/// Run `f` in a child UI confined to `r`.
pub fn area<R>(ui: &mut Ui, r: Rect, f: impl FnOnce(&mut Ui) -> R) -> R {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(*ui.layout()));
    child.set_clip_rect(r.intersect(ui.clip_rect()));
    f(&mut child)
}

pub fn icon_font(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Draw a centred icon glyph.
pub fn draw_icon(ui: &Ui, center: egui::Pos2, icon: &str, size: f32, color: Color32) {
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        icon,
        icon_font(size),
        color,
    );
}

/// An icon button of `size`, with the accent background when `on` and a dimmed icon when
/// disabled (drawn, never hidden: U-003).
pub fn icon_button(
    ui: &mut Ui,
    c: &Colors,
    icon: &str,
    tip: &str,
    size: Vec2,
    enabled: bool,
    on: bool,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if on {
        fill(ui, rect, c.accent);
    } else if enabled && resp.hovered() {
        fill(ui, rect, c.accent.gamma_multiply(0.55));
    }
    let color = if enabled { c.icon } else { c.icon_dim };
    draw_icon(
        ui,
        rect.center(),
        icon,
        (size.y - 4.0).clamp(10.0, 20.0),
        color,
    );
    let resp = resp.on_hover_text(if enabled {
        tip.to_owned()
    } else {
        format!("{tip} (not available yet)")
    });
    if enabled {
        resp
    } else {
        resp.on_hover_cursor(egui::CursorIcon::NotAllowed)
    }
}

/// A vertical separator for icon bars: 1 px line with ~6 px padding on each side.
pub fn bar_separator(ui: &mut Ui, c: &Colors, h: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(13.0, h), Sense::hover());
    ui.painter().line_segment(
        [
            pos2(r.center().x, r.top() + 3.0),
            pos2(r.center().x, r.bottom() - 3.0),
        ],
        Stroke::new(1.0, c.tool_separator),
    );
}

/// One tab of a palette title bar.
pub struct Tab<'a> {
    pub icon: &'a str,
    pub label: &'a str,
    pub enabled: bool,
}

/// A 17 px palette title bar: "≡" menu button at the left, then tabs. The active tab shows icon
/// and name on the bar colour; inactive tabs show only their icon (U-001). With `icons_only`
/// even the active tab shows just its icon (colour group). Returns the "≡" button's response.
pub fn title_bar(
    ui: &mut Ui,
    c: &Colors,
    r: Rect,
    tabs: &[Tab<'_>],
    active: &mut usize,
    icons_only: bool,
) -> Response {
    fill(ui, r, c.header);
    let menu_r = Rect::from_min_size(r.min, vec2(14.0, r.height()));
    let menu = ui.interact(
        menu_r,
        ui.scope_id()
            .with(("palette-menu", r.min.x as i32, r.min.y as i32)),
        Sense::click(),
    );
    draw_icon(ui, menu_r.center(), icons::LIST, 11.0, c.icon);
    let mut x = menu_r.right() + 2.0;
    let font = FontId::proportional(11.0);
    for (i, t) in tabs.iter().enumerate() {
        let is_active = i == *active;
        let label_w = if is_active && !icons_only {
            ui.fonts_mut(|f| {
                f.layout_no_wrap(t.label.to_owned(), font.clone(), c.text)
                    .size()
                    .x
            }) + 6.0
        } else {
            0.0
        };
        let w = 22.0 + label_w;
        let tr = Rect::from_min_size(pos2(x, r.top()), vec2(w, r.height()));
        let resp = ui
            .interact(
                tr,
                ui.scope_id()
                    .with(("tab", r.min.x as i32, r.min.y as i32, i)),
                Sense::click(),
            )
            .on_hover_text(if t.enabled {
                t.label.to_owned()
            } else {
                format!("{} (not available yet)", t.label)
            });
        if is_active {
            fill(ui, tr, if icons_only { c.accent } else { c.bar });
        } else if resp.hovered() {
            fill(ui, tr, c.bar.gamma_multiply(0.8));
        }
        let ic = if t.enabled { c.icon } else { c.icon_dim };
        draw_icon(ui, pos2(tr.left() + 11.0, tr.center().y), t.icon, 12.0, ic);
        if label_w > 0.0 {
            ui.painter().text(
                pos2(tr.left() + 22.0, tr.center().y),
                egui::Align2::LEFT_CENTER,
                t.label,
                font.clone(),
                c.text,
            );
        }
        if resp.clicked() {
            *active = i;
        }
        x += w + 1.0;
        if x > r.right() {
            break;
        }
    }
    menu
}

/// A single-palette title: "≡", palette icon and title text (e.g. "Sub Tool: Pen").
pub fn title(ui: &mut Ui, c: &Colors, r: Rect, icon: &str, text: &str) -> Response {
    let mut a = 0;
    title_bar(
        ui,
        c,
        r,
        &[Tab {
            icon,
            label: text,
            enabled: true,
        }],
        &mut a,
        false,
    )
}

/// Format a value like CSP's fields: decimals only where the setting has them.
pub fn fmt_value(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}")
}

/// A Tool property row (U-004): label at top-left, value right-aligned with a spinner, an
/// optional square dynamics button at the far right, and a thin value bar under the label.
/// Dragging on the row sets the value. Returns (changed, dynamics button response).
#[allow(clippy::too_many_arguments)]
pub fn prop_slider(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    log: bool,
    decimals: usize,
    dynamics: Option<bool>,
) -> (bool, Option<Response>) {
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(vec2(w, 27.0), Sense::click_and_drag());
    let (lo, hi) = (*range.start(), *range.end());
    let to_t = |v: f64| {
        if log {
            ((v.max(lo).ln() - lo.ln()) / (hi.ln() - lo.ln())).clamp(0.0, 1.0)
        } else {
            ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
        }
    };
    let from_t = |t: f64| {
        if log {
            (lo.ln() + t * (hi.ln() - lo.ln())).exp()
        } else {
            lo + t * (hi - lo)
        }
    };
    let dyn_w = if dynamics.is_some() { 16.0 } else { 0.0 };
    let bar = Rect::from_min_max(
        pos2(r.left() + 14.0, r.top() + 17.0),
        pos2(r.right() - 56.0 - dyn_w, r.top() + 22.0),
    );
    let p = ui.painter();
    p.text(
        pos2(r.left() + 14.0, r.top() + 8.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        c.text,
    );
    // Thin bar: baseline, then the fill with lighter top and bottom edges (U-004).
    p.line_segment(
        [
            pos2(bar.left(), bar.bottom()),
            pos2(bar.right(), bar.bottom()),
        ],
        Stroke::new(1.0, c.slider_base),
    );
    let fx = bar.left() + bar.width() * to_t(*value) as f32;
    let fr = Rect::from_min_max(bar.min, pos2(fx, bar.bottom()));
    p.rect_filled(fr, 0.0, c.slider_fill);
    p.line_segment(
        [fr.left_top(), fr.right_top()],
        Stroke::new(1.0, c.slider_top),
    );
    p.line_segment(
        [fr.left_bottom(), fr.right_bottom()],
        Stroke::new(1.0, c.slider_bottom),
    );
    // Value and spinner.
    let vx = r.right() - 20.0 - dyn_w;
    p.text(
        pos2(vx - 2.0, r.top() + 13.0),
        egui::Align2::RIGHT_CENTER,
        fmt_value(*value, decimals),
        FontId::proportional(11.0),
        c.text,
    );
    let up = Rect::from_min_size(pos2(vx, r.top() + 5.0), vec2(10.0, 8.0));
    let down = Rect::from_min_size(pos2(vx, r.top() + 13.0), vec2(10.0, 8.0));
    draw_icon(ui, up.center(), icons::CARET_UP, 8.0, c.icon);
    draw_icon(ui, down.center(), icons::CARET_DOWN, 8.0, c.icon);
    let mut changed = false;
    let step = if log {
        (*value * 0.05).max(0.1)
    } else {
        10f64.powi(-(decimals as i32)).max((hi - lo) / 100.0)
    };
    if resp.clicked()
        && let Some(q) = resp.interact_pointer_pos()
    {
        if up.expand(2.0).contains(q) {
            *value = (*value + step).min(hi);
            changed = true;
        } else if down.expand(2.0).contains(q) {
            *value = (*value - step).max(lo);
            changed = true;
        }
    }
    if (resp.dragged() || (resp.clicked() && !changed))
        && let Some(q) = resp.interact_pointer_pos()
        && q.x < vx - 30.0
    {
        let t = f64::from(((q.x - bar.left()) / bar.width()).clamp(0.0, 1.0));
        let nv = from_t(t);
        let nv = (nv * 10f64.powi(decimals as i32)).round() / 10f64.powi(decimals as i32);
        if nv != *value {
            *value = nv.clamp(lo, hi);
            changed = true;
        }
    }
    let dyn_resp = dynamics.map(|on| {
        let dr = Rect::from_min_size(
            pos2(r.right() - dyn_w - 2.0, r.top() + 5.0),
            vec2(14.0, 14.0),
        );
        let dresp = ui
            .interact(dr, resp.id.with("dyn"), Sense::click())
            .on_hover_text("Dynamics (pen pressure)");
        let p = ui.painter();
        p.rect_filled(dr, 0.0, if on { c.accent } else { c.button });
        draw_icon(ui, dr.center(), icons::PEN_NIB, 10.0, c.icon);
        dresp
    });
    ui.painter().line_segment(
        [
            pos2(r.left() + 8.0, r.bottom() - 0.5),
            pos2(r.right() - 4.0, r.bottom() - 0.5),
        ],
        Stroke::new(1.0, c.row_separator),
    );
    (changed, dyn_resp)
}

/// A labelled row with arbitrary content at the right (segmented controls, checkboxes).
pub fn prop_row<R>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    enabled: bool,
    content: impl FnOnce(&mut Ui) -> R,
) -> R {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 27.0), Sense::hover());
    ui.painter().text(
        pos2(r.left() + 14.0, r.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        if enabled { c.text } else { c.text_disabled },
    );
    let inner = Rect::from_min_max(
        pos2(r.left() + 86.0, r.top() + 4.0),
        pos2(r.right() - 4.0, r.bottom() - 4.0),
    );
    let out = area(ui, inner, |ui| {
        ui.add_enabled_ui(enabled, |ui| ui.horizontal_centered(content).inner)
            .inner
    });
    ui.painter().line_segment(
        [
            pos2(r.left() + 8.0, r.bottom() - 0.5),
            pos2(r.right() - 4.0, r.bottom() - 0.5),
        ],
        Stroke::new(1.0, c.row_separator),
    );
    out
}

/// A checkbox row in the Tool property style: an optional "+" expander, the box, then the label
/// (U-004: "Adjust by speed", "Vector magnet").
pub fn prop_check(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    expander: bool,
    enabled: bool,
    value: &mut bool,
) -> bool {
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(
        vec2(w, 27.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    let mut x = r.left() + 6.0;
    if expander {
        let er = Rect::from_center_size(pos2(x + 5.0, r.center().y), vec2(9.0, 9.0));
        p.rect_stroke(
            er,
            0.0,
            Stroke::new(1.0, c.icon_dim),
            egui::StrokeKind::Inside,
        );
        p.line_segment(
            [
                pos2(er.left() + 2.0, er.center().y),
                pos2(er.right() - 2.0, er.center().y),
            ],
            Stroke::new(1.0, c.icon_dim),
        );
        p.line_segment(
            [
                pos2(er.center().x, er.top() + 2.0),
                pos2(er.center().x, er.bottom() - 2.0),
            ],
            Stroke::new(1.0, c.icon_dim),
        );
    }
    x += 12.0;
    let br = Rect::from_center_size(pos2(x + 6.0, r.center().y), vec2(12.0, 12.0));
    p.rect_filled(br, 0.0, if enabled { c.button } else { c.bar });
    if *value {
        draw_icon(ui, br.center(), icons::CHECK, 10.0, c.text);
    }
    ui.painter().text(
        pos2(x + 17.0, r.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        if enabled { c.text } else { c.text_disabled },
    );
    ui.painter().line_segment(
        [
            pos2(r.left() + 8.0, r.bottom() - 0.5),
            pos2(r.right() - 4.0, r.bottom() - 0.5),
        ],
        Stroke::new(1.0, c.row_separator),
    );
    if resp.clicked() {
        *value = !*value;
        true
    } else {
        false
    }
}

/// A segmented control of icon cells; returns the clicked index.
pub fn segmented(
    ui: &mut Ui,
    c: &Colors,
    cells: &[(&str, &str)],
    selected: usize,
) -> Option<usize> {
    let mut clicked = None;
    ui.spacing_mut().item_spacing.x = 0.0;
    for (i, (icon, tip)) in cells.iter().enumerate() {
        if icon_button(ui, c, icon, tip, vec2(22.0, 19.0), true, i == selected).clicked() {
            clicked = Some(i);
        }
    }
    clicked
}
