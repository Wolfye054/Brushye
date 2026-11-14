//! Right dock (specs U-006, U-007): Navigator group, Layer Property group, Layer group (Layer
//! palette, History, Auto Action).

use std::sync::Arc;

use brushye_command::commands::{self, Command};
use brushye_command::core::{BlendMode, Document, Layer, LayerId};
use eframe::egui::{self, Color32, ColorImage, Rect, Sense, Stroke, Ui, pos2, vec2};

use super::chrome;
use super::theme::{Colors, m};
use super::widgets::{self, Tab};
use crate::app::App;
use crate::icons;

// ---------------------------------------------------------------- navigator

pub fn navigator_group(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    let title = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let tabs = [
        Tab {
            icon: icons::COMPASS,
            label: "Navigator",
            enabled: true,
        },
        Tab {
            icon: icons::IMAGE,
            label: "Sub View",
            enabled: false,
        },
        Tab {
            icon: icons::BOOKMARK_SIMPLE,
            label: "Item bank",
            enabled: false,
        },
        Tab {
            icon: icons::INFO,
            label: "Information",
            enabled: true,
        },
    ];
    let mut tab = app.ui.nav_tab;
    let menu = widgets::title_bar(ui, c, title, &tabs, &mut tab, false);
    egui::Popup::menu(&menu).show(|ui| navigator_menu(app, ui));
    if tabs[tab].enabled {
        app.ui.nav_tab = tab;
    }
    let body = Rect::from_min_max(pos2(r.left(), title.bottom()), r.max);
    match app.ui.nav_tab {
        3 => information(app, ui, c, body),
        _ => navigator(app, ui, c, body),
    }
}

fn navigator(app: &mut App, ui: &mut Ui, c: &Colors, body: Rect) {
    // The control rows are the Navigator's "command bar" (≡ > Show command bar, U-012).
    let controls_h = if app.ui.prefs.navigator_bar {
        44.0
    } else {
        0.0
    };
    let preview = Rect::from_min_max(body.min, pos2(body.right(), body.bottom() - controls_h));
    widgets::fill(ui, preview, c.surround);
    if let Some(doc) = app.doc() {
        let (w, h) = (doc.width as f32, doc.height as f32);
        let k = ((preview.width() - 6.0) / w).min((preview.height() - 6.0) / h);
        let img_r = Rect::from_center_size(preview.center(), vec2(w * k, h * k));
        let generation = app.display_generation();
        let stale = app
            .ui
            .nav_texture
            .as_ref()
            .is_none_or(|(g, _)| *g != generation);
        if stale {
            let (tw, th) = (
                (img_r.width().round() as usize).max(1),
                (img_r.height().round() as usize).max(1),
            );
            let mut px = Vec::with_capacity(tw * th * 4);
            for y in 0..th {
                for x in 0..tw {
                    let cx = ((x as f32 + 0.5) / tw as f32 * w) as u32;
                    let cy = ((y as f32 + 0.5) / th as f32 * h) as u32;
                    let p = app.display_pixel(cx, cy).unwrap_or([0; 4]);
                    // Transparent areas over the checker colours.
                    let bg = if ((x / 4) + (y / 4)) % 2 == 0 {
                        255u32
                    } else {
                        204
                    };
                    let a = u32::from(p[3]);
                    let mix = |v: u8| ((u32::from(v) * a + bg * (255 - a)) / 255) as u8;
                    px.extend_from_slice(&[mix(p[0]), mix(p[1]), mix(p[2]), 255]);
                }
            }
            let tex = ui.ctx().load_texture(
                "navigator",
                ColorImage::from_rgba_unmultiplied([tw, th], &px),
                egui::TextureOptions::LINEAR,
            );
            app.ui.nav_texture = Some((generation, tex));
        }
        if let Some((_, tex)) = &app.ui.nav_texture {
            ui.painter().image(
                tex.id(),
                img_r,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        // The visible area as a red outline (canvas viewport mapped back to the document).
        let (min, size) = app.canvas_px();
        let corners = [
            [min[0], min[1]],
            [min[0] + size[0], min[1]],
            [min[0] + size[0], min[1] + size[1]],
            [min[0], min[1] + size[1]],
        ];
        let pts: Vec<egui::Pos2> = corners
            .iter()
            .map(|&q| {
                let p = app.view.to_canvas(q);
                pos2(
                    img_r.left() + p[0] as f32 * k,
                    img_r.top() + p[1] as f32 * k,
                )
            })
            .collect();
        ui.painter()
            .with_clip_rect(preview)
            .add(egui::Shape::closed_line(
                pts,
                Stroke::new(1.0, Color32::from_rgb(230, 60, 60)),
            ));
        // Click/drag in the preview centres the view there.
        let resp = ui.interact(
            preview,
            ui.scope_id().with("nav-preview"),
            Sense::click_and_drag(),
        );
        if (resp.dragged() || resp.clicked())
            && let Some(q) = resp.interact_pointer_pos()
        {
            let target = [
                f64::from((q.x - img_r.left()) / k),
                f64::from((q.y - img_r.top()) / k),
            ];
            let now = app.view.to_screen(target);
            let centre = app.canvas_center();
            app.view.pan_by([centre[0] - now[0], centre[1] - now[1]]);
        }
    }
    let controls = Rect::from_min_max(pos2(body.left(), preview.bottom()), body.max);
    widgets::fill(ui, controls, c.bar);
    let center = app.canvas_center();
    for row in 0..2 {
        let rr = Rect::from_min_size(
            pos2(
                controls.left() + 4.0,
                controls.top() + 2.0 + row as f32 * 21.0,
            ),
            vec2(controls.width() - 8.0, 19.0),
        );
        let wedge_r = Rect::from_min_size(rr.min + vec2(0.0, 3.0), vec2(82.0, 13.0));
        let font = egui::FontId::proportional(10.0);
        if row == 0 {
            if let Some(t) =
                chrome::wedge(ui, c, wedge_r, chrome::zoom_to_t(app.view.zoom), "nav-zoom")
            {
                app.view.set_zoom(chrome::t_to_zoom(t), center);
            }
            ui.painter().text(
                pos2(wedge_r.right() + 34.0, rr.center().y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", app.view.zoom * 100.0),
                font,
                c.text,
            );
        } else {
            let t = ((chrome::signed_angle(app.view.angle) + 180.0) / 360.0) as f32;
            if let Some(t) = chrome::wedge(ui, c, wedge_r, t, "nav-rot") {
                let a = app.view.angle;
                app.view.rotate_by(
                    f64::from(t) * 360.0 - 180.0 - chrome::signed_angle(a),
                    center,
                );
            }
            ui.painter().text(
                pos2(wedge_r.right() + 34.0, rr.center().y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", chrome::signed_angle(app.view.angle)),
                font,
                c.text,
            );
        }
        let buttons = Rect::from_min_max(pos2(wedge_r.right() + 36.0, rr.top()), rr.max);
        widgets::area(ui, buttons, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let s = vec2(18.0, 18.0);
                if row == 0 {
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::MAGNIFYING_GLASS_MINUS,
                        "Zoom out",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.view.step_zoom(-1, center);
                    }
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::MAGNIFYING_GLASS_PLUS,
                        "Zoom in",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.view.step_zoom(1, center);
                    }
                    if widgets::icon_button(ui, c, icons::SQUARE, "100 %", s, true, false).clicked()
                    {
                        app.actual_size();
                    }
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::ARROWS_IN,
                        "Fit to navigator",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.fit();
                    }
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::CORNERS_OUT,
                        "Fit to screen",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.fit();
                    }
                } else {
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::ARROW_ARC_LEFT,
                        "Rotate left",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.view.rotate_by(-15.0, center);
                    }
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::ARROW_ARC_RIGHT,
                        "Rotate right",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        app.view.rotate_by(15.0, center);
                    }
                    if widgets::icon_button(
                        ui,
                        c,
                        icons::ARROW_COUNTER_CLOCKWISE,
                        "Reset rotation",
                        s,
                        true,
                        false,
                    )
                    .clicked()
                    {
                        let a = app.view.angle;
                        app.view.rotate_by(-a, center);
                    }
                    widgets::icon_button(
                        ui,
                        c,
                        icons::FLIP_HORIZONTAL,
                        "Flip horizontal",
                        s,
                        false,
                        false,
                    );
                    widgets::icon_button(
                        ui,
                        c,
                        icons::FLIP_VERTICAL,
                        "Flip vertical",
                        s,
                        false,
                        false,
                    );
                }
            });
        });
    }
}

fn information(app: &mut App, ui: &mut Ui, c: &Colors, body: Rect) {
    widgets::fill(ui, body, c.body);
    widgets::area(ui, body.shrink(6.0), |ui| {
        if let Some(d) = app.doc() {
            ui.label(format!(
                "Canvas  {} × {} px, {} dpi",
                d.width, d.height, d.dpi
            ));
        }
        ui.label(app.tablet_status());
        match app.cursor_canvas() {
            Some(p) => {
                ui.label(format!("X {:.1}   Y {:.1}", p[0], p[1]));
                if let Some(px) = app.display_pixel(p[0].max(0.0) as u32, p[1].max(0.0) as u32) {
                    let hsv = super::colour::rgb_to_hsv([px[0], px[1], px[2]]);
                    ui.label(format!(
                        "R {}  G {}  B {}  A {}",
                        px[0], px[1], px[2], px[3]
                    ));
                    ui.label(format!(
                        "H {:.0}  S {:.0}  V {:.0}",
                        hsv[0],
                        hsv[1] * 100.0,
                        hsv[2] * 100.0
                    ));
                }
            }
            None => {
                ui.colored_label(c.text_dim, "Move the pointer over the canvas");
            }
        }
    });
}

// ---------------------------------------------------------------- layer property

pub fn layer_property_group(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    let title = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let tabs = [
        Tab {
            icon: icons::STACK,
            label: "Layer Property",
            enabled: true,
        },
        Tab {
            icon: icons::MAGNIFYING_GLASS,
            label: "Search Layer",
            enabled: false,
        },
        Tab {
            icon: icons::FILM_STRIP,
            label: "Animation cels",
            enabled: false,
        },
    ];
    let mut tab = 0;
    let menu = widgets::title_bar(ui, c, title, &tabs, &mut tab, false);
    egui::Popup::menu(&menu).show(|ui| {
        ui.add_enabled(false, egui::Button::new("Save as default…"));
        ui.separator();
        if ui.button("Hide Layer Property palette").clicked() {
            app.ui.hidden.insert("layer_property");
            ui.close();
        }
        ui.add_enabled(false, egui::Button::new("Show Search Layer palette"));
        ui.add_enabled(false, egui::Button::new("Show Animation cels palette"));
    });
    let body = Rect::from_min_max(pos2(r.left(), title.bottom()), r.max);
    widgets::fill(ui, body, c.body);
    widgets::area(ui, body.shrink2(vec2(6.0, 6.0)), |ui| {
        group_label(ui, c, "Effect");
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), 20.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                let s = vec2(20.0, 18.0);
                widgets::icon_button(
                    ui,
                    c,
                    icons::CARET_DOWN,
                    "More effects",
                    vec2(12.0, 18.0),
                    false,
                    false,
                );
                widgets::icon_button(ui, c, icons::SQUARES_FOUR, "Tone", s, false, false);
                widgets::icon_button(ui, c, icons::DOTS_NINE, "Mosaic", s, false, false);
                widgets::icon_button(ui, c, icons::CIRCLE, "Border", s, false, false);
            },
        );
        ui.add_space(6.0);
        group_label(ui, c, "Expression color");
        ui.add_enabled_ui(false, |ui| {
            egui::ComboBox::from_id_salt("expression")
                .selected_text("Color")
                .width(150.0)
                .show_ui(ui, |_| {});
        });
    });
}

fn group_label(ui: &mut Ui, c: &Colors, text: &str) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 14.0), Sense::hover());
    ui.painter().text(
        pos2(r.left(), r.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        c.text,
    );
    let tw = ui.fonts_mut(|f| {
        f.layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.0), c.text)
            .size()
            .x
    });
    ui.painter().line_segment(
        [
            pos2(r.left() + tw + 4.0, r.center().y),
            pos2(r.right(), r.center().y),
        ],
        Stroke::new(1.0, c.splitter),
    );
}

// ---------------------------------------------------------------- layer palette

pub fn layer_group(app: &mut App, ui: &mut Ui, r: Rect) {
    let c = app.ui.colors();
    let title = Rect::from_min_size(r.min, vec2(r.width(), m::TITLE_BAR));
    let tabs = [
        Tab {
            icon: icons::STACK,
            label: "Layer",
            enabled: true,
        },
        Tab {
            icon: icons::CLOCK_COUNTER_CLOCKWISE,
            label: "History",
            enabled: true,
        },
        Tab {
            icon: icons::PLAY,
            label: "Auto Action",
            enabled: false,
        },
    ];
    let mut tab = app.ui.layer_tab;
    let menu = widgets::title_bar(ui, c, title, &tabs, &mut tab, false);
    egui::Popup::menu(&menu).show(|ui| layer_menu(app, ui));
    if tabs[tab].enabled {
        app.ui.layer_tab = tab;
    }
    let body = Rect::from_min_max(pos2(r.left(), title.bottom()), r.max);
    match app.ui.layer_tab {
        1 => history(app, ui, c, body),
        _ => layers(app, ui, c, body),
    }
}

fn set_cmd(id: LayerId) -> commands::LayerSet {
    commands::LayerSet {
        layer: id,
        name: None,
        opacity: None,
        blend: None,
        locked: None,
        lock_alpha: None,
        clip: None,
    }
}

fn layers(app: &mut App, ui: &mut Ui, c: &Colors, body: Rect) {
    let Some(doc) = app.doc() else { return };
    let active = app.session.active_layer();
    let layer = active.and_then(|a| doc.layer(a)).cloned();
    // Top bar: palette colour, blend mode, opacity (U-006).
    // Bars can be hidden from the palette's ≡ menu (U-012); a hidden bar has zero height and its
    // widgets are clipped away.
    let prop_h = if app.ui.prefs.layer_property_bar {
        26.0
    } else {
        0.0
    };
    let (row1_h, row2_h) = if app.ui.prefs.layer_bar {
        (21.0, 23.0)
    } else {
        (0.0, 0.0)
    };
    let top = Rect::from_min_size(body.min, vec2(body.width(), prop_h));
    widgets::fill(ui, top, c.bar);
    widgets::area(ui, top.shrink2(vec2(3.0, 3.0)), |ui| {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            widgets::icon_button(
                ui,
                c,
                icons::PALETTE,
                "Change palette color",
                vec2(26.0, 20.0),
                false,
                false,
            );
            let enabled = layer
                .as_ref()
                .is_some_and(|l| !l.locked && !clip_base_hidden(&doc, l.id));
            ui.add_enabled_ui(enabled, |ui| {
                let blend = layer.as_ref().map_or(BlendMode::Normal, |l| l.blend);
                // A clipped folder can't pass through: CSP shows Through in red and renders it
                // as Normal (L-004 run 3).
                let red_through = layer.as_ref().is_some_and(|l| l.is_folder() && l.clip);
                let label = |m: BlendMode| {
                    let t = egui::RichText::new(blend_label(m));
                    if m == BlendMode::Through && red_through {
                        t.color(THROUGH_RED)
                    } else {
                        t
                    }
                };
                egui::ComboBox::from_id_salt("layer-blend")
                    .width(74.0)
                    .selected_text(label(blend))
                    .show_ui(ui, |ui| {
                        // Folders get Through as the first entry (U-014).
                        let folder = layer.as_ref().is_some_and(|l| l.is_folder());
                        let modes = folder
                            .then_some(BlendMode::Through)
                            .into_iter()
                            .chain(BlendMode::ALL.iter().copied());
                        for mode in modes {
                            if ui.selectable_label(blend == mode, label(mode)).clicked()
                                && let Some(l) = &layer
                                && l.blend != mode
                            {
                                let mut s = set_cmd(l.id);
                                s.blend = Some(mode);
                                app.exec(Command::LayerSet(s));
                            }
                        }
                    });
                let mut op = layer.as_ref().map_or(100, |l| l.opacity);
                ui.spacing_mut().slider_width = 64.0;
                let changed = ui
                    .add(egui::Slider::new(&mut op, 0..=100).trailing_fill(true))
                    .on_hover_text("Opacity")
                    .changed();
                if changed && let Some(l) = &layer {
                    let mut s = set_cmd(l.id);
                    s.opacity = Some(op);
                    app.exec(Command::LayerSet(s));
                }
            });
        });
    });
    // Toolbar rows (U-006).
    let row1 = Rect::from_min_size(
        pos2(
            body.left(),
            top.bottom() + if prop_h > 0.0 { 1.0 } else { 0.0 },
        ),
        vec2(body.width(), row1_h),
    );
    let row2 = Rect::from_min_size(
        pos2(
            body.left(),
            row1.bottom() + if row1_h > 0.0 { 1.0 } else { 0.0 },
        ),
        vec2(body.width(), row2_h),
    );
    widgets::fill(ui, row1, c.bar);
    widgets::fill(ui, row2, c.bar);
    let s = vec2(20.0, 20.0);
    let can_edit = layer.as_ref().is_some_and(|l| !l.locked);
    widgets::area(ui, row1, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            widgets::icon_button(
                ui,
                c,
                icons::SWATCHES,
                "Change layer color",
                s,
                false,
                false,
            );
            widgets::icon_button(ui, c, icons::RULER, "Set ruler range", s, false, false);
            widgets::icon_button(ui, c, icons::MASK_HAPPY, "Enable mask", s, false, false);
            if let Some(l) = &layer {
                if widgets::icon_button(
                    ui,
                    c,
                    icons::CHECKERBOARD,
                    "Lock transparent pixels",
                    s,
                    !l.locked,
                    l.lock_alpha,
                )
                .clicked()
                {
                    let mut cmd = set_cmd(l.id);
                    cmd.lock_alpha = Some(!l.lock_alpha);
                    app.exec(Command::LayerSet(cmd));
                }
                if widgets::icon_button(ui, c, icons::LOCK, "Lock layer", s, true, l.locked)
                    .clicked()
                {
                    let mut cmd = set_cmd(l.id);
                    cmd.locked = Some(!l.locked);
                    app.exec(Command::LayerSet(cmd));
                }
            }
            widgets::icon_button(
                ui,
                c,
                icons::PENCIL_LINE,
                "Set as draft layer",
                s,
                false,
                false,
            );
            widgets::icon_button(
                ui,
                c,
                icons::BOOKMARK_SIMPLE,
                "Set as reference layer",
                s,
                false,
                false,
            );
            let clip = layer.as_ref().is_some_and(|l| l.clip);
            if widgets::icon_button(
                ui,
                c,
                icons::PAPERCLIP,
                "Clip to layer below",
                s,
                can_edit,
                clip,
            )
            .clicked()
            {
                app.toggle_clip();
            }
        });
    });
    let mut new_layer = false;
    let mut new_folder = false;
    let mut delete = false;
    let can_delete = app.can_delete_layer();
    widgets::area(ui, row2, |ui| {
        ui.horizontal_centered(|ui| {
            widgets::icon_button(
                ui,
                c,
                icons::COLUMNS,
                "Show layer in 2 panes",
                s,
                false,
                false,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                delete =
                    widgets::icon_button(ui, c, icons::TRASH, "Delete layer", s, can_delete, false)
                        .clicked();
                widgets::icon_button(
                    ui,
                    c,
                    icons::ARROW_LINE_DOWN,
                    "Apply mask to layer",
                    s,
                    false,
                    false,
                );
                widgets::icon_button(
                    ui,
                    c,
                    icons::MASK_HAPPY,
                    "Create layer mask",
                    s,
                    false,
                    false,
                );
                widgets::icon_button(
                    ui,
                    c,
                    icons::STACK_SIMPLE,
                    "Merge with layer below",
                    s,
                    false,
                    false,
                );
                widgets::icon_button(
                    ui,
                    c,
                    icons::ARROW_FAT_LINES_DOWN,
                    "Transfer to lower layer",
                    s,
                    false,
                    false,
                );
                new_folder = widgets::icon_button(
                    ui,
                    c,
                    icons::FOLDER_SIMPLE_PLUS,
                    "New layer folder",
                    s,
                    true,
                    false,
                )
                .clicked();
                widgets::icon_button(
                    ui,
                    c,
                    icons::PEN_NIB_STRAIGHT,
                    "New vector layer",
                    s,
                    false,
                    false,
                );
                new_layer = widgets::icon_button(
                    ui,
                    c,
                    icons::FILE_PLUS,
                    "New raster layer",
                    s,
                    true,
                    false,
                )
                .clicked();
            });
        });
    });
    if new_layer {
        app.new_raster_layer();
    }
    if new_folder {
        app.new_folder();
    }
    if delete && let Some(l) = &layer {
        app.exec(Command::LayerDelete(commands::LayerDelete { layer: l.id }));
    }
    // Layer rows, top to bottom (U-006), then the paper.
    let list = Rect::from_min_max(pos2(body.left(), row2.bottom() + 1.0), body.max);
    widgets::fill(ui, list, c.body);
    let layers = palette_rows(&doc);
    let mut hover_drop: Option<(LayerId, crate::app::MoveTo)> = None;
    let dragging = ui.ctx().dragged_id().is_some();
    widgets::area(ui, list, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("layers")
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut rows = Vec::new();
                for (i, row) in layers.iter().enumerate() {
                    let rr = layer_row(app, ui, c, &doc, row, active == Some(row.layer.id));
                    rows.push((rr, row.layer.id, i));
                }
                if let Some(p) = doc.paper {
                    paper_row(ui, c, p.0);
                }
                // Drag a row onto another (U-015): a layer row's upper/lower half puts it
                // above/below; a folder row's top quarter above it, middle half into it at the
                // bottom, bottom quarter at the top of its contents (expanded) or below it
                // (collapsed).
                if let Some(dragged) = ui.ctx().dragged_id()
                    && let Some(&(_, id, _)) = rows.iter().find(|(r, _, _)| r.1 == dragged)
                    && let Some(pos) = ui.ctx().pointer_interact_pos()
                    && let Some(&(_, _, j)) = rows.iter().find(|(r, _, _)| r.0.contains(pos))
                {
                    let rr = rows[j].0.0;
                    let t = &layers[j].layer;
                    let f = (pos.y - rr.top()) / rr.height();
                    use crate::app::MoveTo;
                    let to = if t.is_folder() {
                        if f < 0.25 {
                            MoveTo::Above(t.id)
                        } else if f < 0.75 {
                            MoveTo::IntoBottom(t.id)
                        } else if t.expanded {
                            MoveTo::IntoTop(t.id)
                        } else {
                            MoveTo::Below(t.id)
                        }
                    } else if f < 0.5 {
                        MoveTo::Above(t.id)
                    } else {
                        MoveTo::Below(t.id)
                    };
                    // A layer can't go onto itself or into its own folder tree.
                    let valid = t.id != id && !doc.is_inside(t.id, id);
                    if valid {
                        let stroke = Stroke::new(2.0, c.accent);
                        match to {
                            MoveTo::IntoBottom(_) => {
                                ui.painter().rect_stroke(
                                    rr.shrink(1.0),
                                    0.0,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }
                            MoveTo::Above(_) => {
                                ui.painter()
                                    .line_segment([rr.left_top(), rr.right_top()], stroke);
                            }
                            _ => {
                                ui.painter()
                                    .line_segment([rr.left_bottom(), rr.right_bottom()], stroke);
                            }
                        }
                        hover_drop = Some((id, to));
                    }
                }
            });
    });
    // egui ends the drag in the frame the button is released, so the target seen while
    // dragging is kept and applied when the drag ends.
    if dragging {
        app.ui.layer_drop = hover_drop;
    } else if let Some((id, to)) = app.ui.layer_drop.take() {
        app.move_layer_to(id, to);
    }
}

fn blend_label(m: BlendMode) -> String {
    let n = m.name().replace('_', " ");
    let mut cs = n.chars();
    cs.next()
        .map(|f| f.to_uppercase().collect::<String>() + cs.as_str())
        .unwrap_or_default()
}

/// One palette row: a layer at its folder depth, with how its clip bar shows.
struct PaletteRow {
    layer: Arc<Layer>,
    depth: usize,
    /// `Some(true)`: clipped (red bar); `Some(false)`: clip set but off, because the base is
    /// a Through folder (grey bar, U-014 / L-006).
    clip: Option<bool>,
}

/// A clipped layer whose clip base is hidden isn't drawn either (L-006 run 3): CSP dims its
/// eye and greys out its blend and opacity controls.
fn clip_base_hidden(doc: &Document, id: LayerId) -> bool {
    let Some(l) = doc.layer(id) else { return false };
    if !l.clip {
        return false;
    }
    let sibs = doc.children(l.parent);
    let Some(i) = sibs.iter().position(|s| s.id == id) else {
        return false;
    };
    sibs[..i]
        .iter()
        .rev()
        .find(|b| !b.clip)
        .is_some_and(|b| !b.visible && !(b.is_folder() && b.blend == BlendMode::Through))
}

/// The rows the Layer palette shows, top to bottom: every layer whose folders are all open,
/// indented by folder depth (U-014).
fn palette_rows(doc: &Document) -> Vec<PaletteRow> {
    fn walk(doc: &Document, parent: Option<LayerId>, depth: usize, out: &mut Vec<PaletteRow>) {
        let kids = doc.children(parent);
        for (i, l) in kids.iter().enumerate().rev() {
            let clip = l.clip.then(|| {
                // The clip base: the nearest unclipped sibling below.
                kids[..i]
                    .iter()
                    .rev()
                    .find(|b| !b.clip)
                    .is_some_and(|b| !(b.is_folder() && b.blend == BlendMode::Through))
            });
            out.push(PaletteRow {
                layer: Arc::clone(l),
                depth,
                clip,
            });
            if l.is_folder() && l.expanded {
                walk(doc, Some(l.id), depth + 1, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(doc, None, 0, &mut out);
    out
}

/// Clip bar: red when clipped onto a base, grey when there is no usable base (U-014).
fn clip_bar_color(on: bool) -> Color32 {
    if on {
        Color32::from_rgb(226, 112, 112)
    } else {
        Color32::from_gray(150)
    }
}

/// Through on a clipped folder (CSP draws it red: it renders as Normal there).
const THROUGH_RED: Color32 = Color32::from_rgb(230, 30, 30);

/// Folder rows are shorter than layer rows (they have no thumbnail, U-014).
const FOLDER_ROW: f32 = 31.0;
/// Indent per folder level (U-014).
const INDENT: f32 = 16.0;
/// Width of the clip bar; the thumbnail moves right by it (U-014).
const CLIP_BAR: f32 = 4.0;

/// One layer row: eye, editing pen, then at the folder depth either a folder's arrow and icon
/// or a layer's thumbnail (after the clip bar when clipped), and "opacity % blend" over the
/// name.
fn layer_row(
    app: &mut App,
    ui: &mut Ui,
    c: &Colors,
    doc: &Document,
    row: &PaletteRow,
    selected: bool,
) -> (Rect, egui::Id) {
    let l: &Layer = &row.layer;
    let w = ui.available_width();
    let h = if l.is_folder() {
        FOLDER_ROW
    } else {
        m::LAYER_ROW
    };
    let (rr, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click_and_drag());
    let eye = Rect::from_min_size(rr.min, vec2(20.0, h - 1.0));
    let pen = Rect::from_min_size(pos2(eye.right(), rr.top()), vec2(19.0, h - 1.0));
    let x0 = pen.right() + 2.0 + INDENT * row.depth as f32;
    let shift = if row.clip.is_some() {
        CLIP_BAR + 2.0
    } else {
        0.0
    };
    let thumb = Rect::from_min_size(pos2(x0 + shift, rr.top() + 1.0), vec2(38.0, 38.0));
    if selected {
        widgets::fill(
            ui,
            Rect::from_min_max(pos2(pen.right(), rr.top()), rr.max),
            c.selection,
        );
    }
    let eye_resp = ui.interact(eye, resp.id.with("eye"), Sense::click());
    widgets::draw_icon(
        ui,
        eye.center(),
        if l.visible {
            icons::EYE
        } else {
            icons::EYE_SLASH
        },
        13.0,
        if l.visible && !clip_base_hidden(doc, l.id) {
            c.icon
        } else {
            c.icon_dim
        },
    );
    if eye_resp.clicked() {
        app.exec(Command::LayerShow(commands::LayerShow {
            layer: l.id,
            visible: !l.visible,
        }));
    }
    if selected {
        widgets::draw_icon(ui, pen.center(), icons::PEN_NIB, 12.0, c.icon);
    }
    let (tx, line1, line2) = if l.is_folder() {
        // Disclosure arrow (opens/closes, not a History step) and the folder icon.
        // A clipped folder shows the clip bar left of its arrow (L-004 run 3).
        if let Some(on) = row.clip {
            let bar = Rect::from_min_size(pos2(x0, rr.top() + 1.0), vec2(CLIP_BAR, h - 3.0));
            ui.painter().rect_filled(bar, 0.0, clip_bar_color(on));
        }
        let arrow = Rect::from_min_size(pos2(x0 + shift, rr.top()), vec2(14.0, h - 1.0));
        let open = l.expanded;
        widgets::draw_icon(
            ui,
            arrow.center(),
            if open {
                icons::CARET_DOWN
            } else {
                icons::CARET_RIGHT
            },
            10.0,
            c.icon,
        );
        if ui
            .interact(arrow, resp.id.with("arrow"), Sense::click())
            .clicked()
        {
            app.exec(Command::LayerExpand(commands::LayerExpand {
                folder: l.id,
                open: !open,
            }));
        }
        widgets::draw_icon(
            ui,
            pos2(arrow.right() + 10.0, arrow.center().y),
            if open {
                icons::FOLDER_OPEN
            } else {
                icons::FOLDER
            },
            15.0,
            c.icon,
        );
        (arrow.right() + 24.0, rr.top() + 9.0, rr.top() + 21.0)
    } else {
        if let Some(on) = row.clip {
            let bar = Rect::from_min_size(pos2(x0, rr.top() + 1.0), vec2(CLIP_BAR, 38.0));
            ui.painter().rect_filled(bar, 0.0, clip_bar_color(on));
        }
        let tex = layer_thumbnail(app, ui, doc, l);
        ui.painter().image(
            tex.id(),
            thumb,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().rect_stroke(
            thumb,
            0.0,
            Stroke::new(1.0, c.splitter),
            egui::StrokeKind::Outside,
        );
        (thumb.right() + 6.0, rr.top() + 11.0, rr.top() + 29.0)
    };
    let small = egui::FontId::proportional(10.0);
    let head = ui.painter().text(
        pos2(tx, line1),
        egui::Align2::LEFT_CENTER,
        format!("{} % ", l.opacity),
        small.clone(),
        c.text,
    );
    let red = l.is_folder() && l.clip && l.blend == BlendMode::Through;
    ui.painter().text(
        pos2(head.right(), line1),
        egui::Align2::LEFT_CENTER,
        blend_label(l.blend),
        small,
        if red { THROUGH_RED } else { c.text },
    );
    let mut tag = String::new();
    if l.locked {
        tag.push_str(icons::LOCK);
    }
    if l.lock_alpha {
        tag.push_str(icons::CHECKERBOARD);
    }
    let renaming = app
        .ui
        .renaming
        .as_ref()
        .is_some_and(|(id, _)| *id == l.id.0);
    if renaming {
        let r = Rect::from_min_size(pos2(tx, line2 - 8.0), vec2(rr.right() - tx - 4.0, 17.0));
        let mut done = None;
        widgets::area(ui, r, |ui| {
            if let Some((_, text)) = app.ui.renaming.as_mut() {
                let te = ui.text_edit_singleline(text);
                te.request_focus();
                if te.lost_focus() {
                    done = Some(text.clone());
                }
            }
        });
        if let Some(name) = done {
            app.ui.renaming = None;
            if !name.trim().is_empty() && name != l.name {
                let mut s = set_cmd(l.id);
                s.name = Some(name);
                app.exec(Command::LayerSet(s));
            }
        }
    } else {
        ui.painter().text(
            pos2(tx, line2),
            egui::Align2::LEFT_CENTER,
            format!(
                "{}{}",
                l.name,
                if tag.is_empty() {
                    String::new()
                } else {
                    format!("  {tag}")
                }
            ),
            egui::FontId::proportional(11.0),
            c.text,
        );
    }
    ui.painter().line_segment(
        [
            pos2(rr.left(), rr.bottom() - 0.5),
            pos2(rr.right(), rr.bottom() - 0.5),
        ],
        Stroke::new(1.0, c.row_separator),
    );
    if resp.double_clicked() {
        app.ui.renaming = Some((l.id.0, l.name.clone()));
    } else if resp.clicked() && !selected {
        app.exec(Command::LayerSelect(commands::LayerSelect { layer: l.id }));
    }
    resp.context_menu(|ui| {
        if app.session.active_layer() != Some(l.id) {
            app.exec(Command::LayerSelect(commands::LayerSelect { layer: l.id }));
        }
        super::menus::layer_context_items(app, ui);
    });
    (rr, resp.id)
}

fn paper_row(ui: &mut Ui, c: &Colors, paper: [u8; 4]) {
    let w = ui.available_width();
    let (rr, _) = ui.allocate_exact_size(vec2(w, m::LAYER_ROW), Sense::hover());
    let eye = Rect::from_min_size(rr.min, vec2(20.0, 40.0));
    widgets::draw_icon(ui, eye.center(), icons::EYE, 13.0, c.icon_dim);
    let thumb = Rect::from_min_size(pos2(rr.left() + 41.0, rr.top() + 1.0), vec2(38.0, 38.0));
    ui.painter()
        .rect_filled(thumb, 0.0, Color32::from_rgb(paper[0], paper[1], paper[2]));
    ui.painter().rect_stroke(
        thumb,
        0.0,
        Stroke::new(1.0, c.splitter),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        pos2(thumb.right() + 6.0, rr.top() + 11.0),
        egui::Align2::LEFT_CENTER,
        "100 % Normal",
        egui::FontId::proportional(10.0),
        c.text_dim,
    );
    ui.painter().text(
        pos2(thumb.right() + 6.0, rr.top() + 29.0),
        egui::Align2::LEFT_CENTER,
        "Paper",
        egui::FontId::proportional(11.0),
        c.text_dim,
    );
    ui.painter().line_segment(
        [
            pos2(rr.left(), rr.bottom() - 0.5),
            pos2(rr.right(), rr.bottom() - 0.5),
        ],
        Stroke::new(1.0, c.row_separator),
    );
}

/// A layer's 38 px thumbnail, recomputed only when its tiles change (sampled, not resampled:
/// cheap for any canvas size).
fn layer_thumbnail(app: &mut App, ui: &Ui, doc: &Document, l: &Layer) -> egui::TextureHandle {
    let sig: Vec<usize> = std::iter::once(doc.width as usize)
        .chain(std::iter::once(doc.height as usize))
        .chain(l.tiles.iter().map(|(_, t)| Arc::as_ptr(t) as usize))
        .collect();
    if let Some((s, tex)) = app.ui.thumbs.get(&l.id.0)
        && (*s == sig || app.is_drawing())
    {
        return tex.clone();
    }
    let n = 38usize;
    let (w, h) = (doc.width as f32, doc.height as f32);
    let k = (n as f32 / w).min(n as f32 / h);
    let (ox, oy) = ((n as f32 - w * k) / 2.0, (n as f32 - h * k) / 2.0);
    let mut px = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let bg = if ((x / 4) + (y / 4)) % 2 == 0 {
                255u32
            } else {
                204
            };
            let (cx, cy) = ((x as f32 + 0.5 - ox) / k, (y as f32 + 0.5 - oy) / k);
            let p = if cx >= 0.0 && cy >= 0.0 && cx < w && cy < h {
                l.pixel(cx as u32, cy as u32)
            } else {
                [0; 4]
            };
            let a = u32::from(p[3]);
            let mix = |v: u8| ((u32::from(v) * a + bg * (255 - a)) / 255) as u8;
            let i = (y * n + x) * 4;
            px[i..i + 4].copy_from_slice(&[mix(p[0]), mix(p[1]), mix(p[2]), 255]);
        }
    }
    let tex = ui.ctx().load_texture(
        format!("layer-thumb-{}", l.id.0),
        ColorImage::from_rgba_unmultiplied([n, n], &px),
        egui::TextureOptions::LINEAR,
    );
    app.ui.thumbs.insert(l.id.0, (sig, tex.clone()));
    tex
}

fn history(app: &mut App, ui: &mut Ui, c: &Colors, body: Rect) {
    widgets::fill(ui, body, c.body);
    let (labels, pos) = app.history();
    let mut goto = None;
    widgets::area(ui, body, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("history")
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for (i, label) in labels.iter().enumerate() {
                    let w = ui.available_width();
                    let (rr, resp) = ui.allocate_exact_size(vec2(w, 19.0), Sense::click());
                    if i == pos {
                        widgets::fill(ui, rr, c.selection);
                    }
                    let color = if i > pos { c.text_disabled } else { c.text };
                    ui.painter().text(
                        pos2(rr.left() + 6.0, rr.center().y),
                        egui::Align2::LEFT_CENTER,
                        history_label(label, i, &app.doc_name()),
                        egui::FontId::proportional(11.0),
                        color,
                    );
                    if resp.clicked() {
                        goto = Some(i);
                    }
                }
            });
    });
    if let Some(i) = goto {
        app.goto_history(i);
    }
}

fn history_label(cmd: &str, i: usize, doc: &str) -> String {
    if i == 0 {
        return doc.to_owned();
    }
    match cmd {
        "brush.stroke" => "Stroke".into(),
        "layer.new" => "New layer".into(),
        "layer.delete" => "Delete layer".into(),
        "layer.move" => "Move layer".into(),
        "layer.set" => "Layer settings".into(),
        "layer.fill" => "Fill".into(),
        "layer.pixels" => "Edit pixels".into(),
        // CSP's own labels (H-002).
        "layer.group" => "Change order of layer".into(),
        "layer.ungroup" => "Ungroup Layer Folder".into(),
        other => other.into(),
    }
}

/// The Navigator's ≡ menu (U-012).
fn navigator_menu(app: &mut App, ui: &mut Ui) {
    let center = app.canvas_center();
    ui.menu_button("Rotate/Flip", |ui| {
        if ui.button("Reset rotation").clicked() {
            let a = app.view.angle;
            app.view.rotate_by(-a, center);
            ui.close();
        }
        for (label, deg) in [
            ("Rotate 90", 90.0),
            ("Rotate 180", 180.0),
            ("Rotate 270", 270.0),
        ] {
            if ui.button(label).clicked() {
                let a = app.view.angle;
                app.view.rotate_by(deg - a, center);
                ui.close();
            }
        }
        ui.add_enabled(false, egui::Button::new("Flip horizontal"));
        ui.add_enabled(false, egui::Button::new("Flip vertical"));
    });
    if ui.button("Zoom in").clicked() {
        app.view.step_zoom(1, center);
        ui.close();
    }
    if ui.button("Zoom out").clicked() {
        app.view.step_zoom(-1, center);
        ui.close();
    }
    if ui.button("100%").clicked() {
        app.actual_size();
        ui.close();
    }
    if ui.button("200%").clicked() {
        app.actual_size();
        app.view.set_zoom(2.0, center);
        ui.close();
    }
    if ui.button("Fit to screen").clicked() || ui.button("Reset display").clicked() {
        app.fit();
        ui.close();
    }
    ui.separator();
    if ui.button("Fit to navigator").clicked() {
        app.fit();
        ui.close();
    }
    ui.add_enabled(false, egui::Button::new("Print size"));
    ui.separator();
    if ui
        .add(egui::Button::selectable(
            app.ui.prefs.navigator_bar,
            "Show command bar",
        ))
        .clicked()
    {
        app.ui.prefs.navigator_bar = !app.ui.prefs.navigator_bar;
        app.ui.prefs_dirty = true;
        ui.close();
    }
    ui.separator();
    if ui.button("Hide Navigator palette").clicked() {
        app.ui.hidden.insert("navigator");
        ui.close();
    }
    ui.add_enabled(false, egui::Button::new("Show Sub View palette"));
    ui.add_enabled(false, egui::Button::new("Show Item bank palette"));
    if ui.button("Show Information palette").clicked() {
        app.ui.nav_tab = 3;
        ui.close();
    }
}

/// The Layer palette's ≡ menu (U-012): the Layer menu's first groups, display toggles, and the
/// hide/show group.
fn layer_menu(app: &mut App, ui: &mut Ui) {
    super::menus::layer_context_items(app, ui);
    ui.separator();
    ui.add_enabled_ui(false, |ui| {
        ui.menu_button("Thumbnail size", |_| {});
        ui.menu_button("Thumbnail display settings", |_| {});
    });
    for (label, which) in [("Show property bar", 0), ("Show command bar", 1)] {
        let on = if which == 0 {
            app.ui.prefs.layer_property_bar
        } else {
            app.ui.prefs.layer_bar
        };
        if ui.add(egui::Button::selectable(on, label)).clicked() {
            if which == 0 {
                app.ui.prefs.layer_property_bar = !on;
            } else {
                app.ui.prefs.layer_bar = !on;
            }
            app.ui.prefs_dirty = true;
            ui.close();
        }
    }
    ui.add_enabled(false, egui::Button::new("Set command bar below list"));
    ui.separator();
    if ui.button("Hide Layer palette").clicked() {
        app.ui.hidden.insert("layer");
        ui.close();
    }
    if ui.button("Show History palette").clicked() {
        app.ui.layer_tab = 1;
        ui.close();
    }
    ui.add_enabled(false, egui::Button::new("Show Auto Action palette"));
}
