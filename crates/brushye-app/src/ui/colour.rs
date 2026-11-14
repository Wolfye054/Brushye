//! Drawing colours and the colour palettes (spec U-005): main / sub / transparent chips, the
//! Color Wheel (hue ring with red at the left, hue running clockwise over the top, and a
//! saturation/value square), Color Set, Color Slider (RGB / HSV / CMY), Color History and
//! Approximate Color. Color Mixing and Intermediate Color keep their tabs but aren't built yet.

use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};

use super::theme::Colors;
use super::widgets::{self, Tab};
use crate::icons;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chip {
    Main,
    Sub,
    Transparent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderMode {
    Rgb,
    Hsv,
    Cmy,
}

pub struct ColourState {
    pub main: [u8; 3],
    pub sub: [u8; 3],
    pub active: Chip,
    /// HSV of the active colour, kept so hue survives grey colours.
    pub hsv: [f32; 3],
    pub history: Vec<[u8; 3]>,
    pub tab: usize,
    pub slider_mode: SliderMode,
}

impl Default for ColourState {
    fn default() -> Self {
        let main = [32, 32, 40];
        Self {
            main,
            sub: [255, 255, 255],
            active: Chip::Main,
            hsv: rgb_to_hsv(main),
            history: Vec::new(),
            tab: 0,
            slider_mode: SliderMode::Rgb,
        }
    }
}

impl ColourState {
    /// The colour strokes use; `None` = the transparent colour (erases, as in CSP).
    pub fn drawing(&self) -> Option<[u8; 3]> {
        match self.active {
            Chip::Main => Some(self.main),
            Chip::Sub => Some(self.sub),
            Chip::Transparent => None,
        }
    }

    fn active_rgb(&self) -> [u8; 3] {
        match self.active {
            Chip::Sub => self.sub,
            _ => self.main,
        }
    }

    /// Set the active chip's colour (selecting Main if the transparent chip is active).
    pub fn set(&mut self, rgb: [u8; 3]) {
        if self.active == Chip::Transparent {
            self.active = Chip::Main;
        }
        match self.active {
            Chip::Sub => self.sub = rgb,
            _ => self.main = rgb,
        }
        let hsv = rgb_to_hsv(rgb);
        // Keep hue (and saturation) where the new colour doesn't define them.
        self.hsv = if hsv[1] == 0.0 || hsv[2] == 0.0 {
            [
                self.hsv[0],
                if hsv[2] == 0.0 { self.hsv[1] } else { hsv[1] },
                hsv[2],
            ]
        } else {
            hsv
        };
    }

    fn set_hsv(&mut self, hsv: [f32; 3]) {
        self.hsv = hsv;
        let rgb = hsv_to_rgb(hsv);
        if self.active == Chip::Transparent {
            self.active = Chip::Main;
        }
        match self.active {
            Chip::Sub => self.sub = rgb,
            _ => self.main = rgb,
        }
    }

    pub fn select(&mut self, chip: Chip) {
        self.active = chip;
        if chip != Chip::Transparent {
            self.hsv = rgb_to_hsv(self.active_rgb());
        }
    }

    /// Record a colour used for drawing (Color History, newest first).
    pub fn used(&mut self) {
        if let Some(c) = self.drawing() {
            self.history.retain(|&h| h != c);
            self.history.insert(0, c);
            self.history.truncate(64);
        }
    }
}

pub fn rgb_to_hsv(c: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = c.map(|v| f32::from(v) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h, if max == 0.0 { 0.0 } else { d / max }, max]
}

pub fn hsv_to_rgb(hsv: [f32; 3]) -> [u8; 3] {
    let [h, s, v] = hsv;
    let c = v * s;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [r, g, b].map(|u| ((u + m) * 255.0).round().clamp(0.0, 255.0) as u8)
}

fn c32(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// Screen angle (radians, clockwise from +x on screen) of a hue: red at the left (U-005).
fn hue_angle(h: f32) -> f32 {
    (180.0 + h).to_radians()
}

fn checker(ui: &Ui, r: Rect, cell: f32) {
    ui.painter().rect_filled(r, 0.0, Color32::WHITE);
    let (nx, ny) = (
        (r.width() / cell).ceil() as i32,
        (r.height() / cell).ceil() as i32,
    );
    for y in 0..ny {
        for x in 0..nx {
            if (x + y) % 2 == 1 {
                let cr = Rect::from_min_size(
                    r.min + vec2(x as f32 * cell, y as f32 * cell),
                    vec2(cell, cell),
                )
                .intersect(r);
                ui.painter().rect_filled(cr, 0.0, Color32::from_gray(204));
            }
        }
    }
}

/// Main / sub / transparent chips (also used in the tool column, U-004).
pub fn chips(ui: &mut Ui, c: &Colors, st: &mut ColourState, at: Pos2, size: f32) {
    let main_r = Rect::from_min_size(at, vec2(size, size));
    let sub_r = Rect::from_min_size(at + vec2(size * 0.55, size * 0.55), vec2(size, size));
    let tr_r = Rect::from_min_size(pos2(at.x, sub_r.bottom() + 4.0), vec2(size * 1.55, 6.0));
    let p = ui.painter();
    p.rect_filled(sub_r, 0.0, c32(st.sub));
    p.rect_stroke(
        sub_r,
        0.0,
        Stroke::new(
            1.0,
            if st.active == Chip::Sub {
                c.text
            } else {
                c.splitter
            },
        ),
        egui::StrokeKind::Inside,
    );
    p.rect_filled(main_r, 0.0, c32(st.main));
    p.rect_stroke(
        main_r,
        0.0,
        Stroke::new(
            1.0,
            if st.active == Chip::Main {
                c.text
            } else {
                c.splitter
            },
        ),
        egui::StrokeKind::Inside,
    );
    checker(ui, tr_r, 3.0);
    if st.active == Chip::Transparent {
        ui.painter().rect_stroke(
            tr_r.expand(1.0),
            0.0,
            Stroke::new(1.0, c.text),
            egui::StrokeKind::Inside,
        );
    }
    let id = ui.scope_id().with(("chips", at.x as i32, at.y as i32));
    if ui
        .interact(main_r, id.with(0), Sense::click())
        .on_hover_text("Main colour")
        .clicked()
    {
        st.select(Chip::Main);
    } else if ui
        .interact(sub_r, id.with(1), Sense::click())
        .on_hover_text("Sub colour")
        .clicked()
    {
        st.select(Chip::Sub);
    }
    if ui
        .interact(tr_r, id.with(2), Sense::click())
        .on_hover_text("Transparent colour")
        .clicked()
    {
        st.select(Chip::Transparent);
    }
}

const TAB_NAMES: [&str; 7] = [
    "Color Wheel",
    "Color Set",
    "Color Slider",
    "Color Mixing",
    "Color History",
    "Approximate Color",
    "Intermediate Color",
];

/// The colour group's ≡ menu (U-012): the active tab's own items, then hide/show palettes.
fn group_menu(ui: &mut Ui, ust: &mut super::UiState) {
    if ust.colour.tab == 0 {
        if ui
            .add(egui::Button::selectable(!ust.prefs.hls, "HSV color space"))
            .clicked()
        {
            ust.prefs.hls = false;
            ust.sets_dirty = true;
            ui.close();
        }
        if ui
            .add(egui::Button::selectable(ust.prefs.hls, "HLS color space"))
            .clicked()
        {
            ust.prefs.hls = true;
            ust.sets_dirty = true;
            ui.close();
        }
        ui.separator();
    }
    for (i, name) in TAB_NAMES.iter().enumerate() {
        let available = !matches!(i, 3 | 6);
        if i == ust.colour.tab {
            if ui.button(format!("Hide {name} palette")).clicked() {
                ust.hidden.insert("colour");
                ui.close();
            }
        } else if ui
            .add_enabled(available, egui::Button::new(format!("Show {name} palette")))
            .clicked()
        {
            ust.colour.tab = i;
            ui.close();
        }
    }
}

/// The colour palette group (left dock, bottom).
pub fn palette(ui: &mut Ui, c: &Colors, ust: &mut super::UiState, r: Rect) {
    let title = Rect::from_min_size(r.min, vec2(r.width(), 17.0));
    let tabs = [
        Tab {
            icon: icons::CIRCLE_HALF,
            label: "Color Wheel",
            enabled: true,
        },
        Tab {
            icon: icons::SWATCHES,
            label: "Color Set",
            enabled: true,
        },
        Tab {
            icon: icons::SLIDERS_HORIZONTAL,
            label: "Color Slider",
            enabled: true,
        },
        Tab {
            icon: icons::PALETTE,
            label: "Color Mixing",
            enabled: false,
        },
        Tab {
            icon: icons::CLOCK_COUNTER_CLOCKWISE,
            label: "Color History",
            enabled: true,
        },
        Tab {
            icon: icons::GRID_FOUR,
            label: "Approximate Color",
            enabled: true,
        },
        Tab {
            icon: icons::CIRCLES_THREE,
            label: "Intermediate Color",
            enabled: false,
        },
    ];
    let menu = widgets::title_bar(ui, c, title, &tabs[..6], &mut ust.colour.tab, true);
    egui::Popup::menu(&menu).show(|ui| group_menu(ui, ust));
    let hls = ust.prefs.hls;
    let st = &mut ust.colour;
    let body = Rect::from_min_max(pos2(r.left(), title.bottom()), r.max);
    widgets::fill(ui, body, c.bar);
    widgets::area(ui, body, |ui| match st.tab {
        0 => wheel(ui, c, st, body, hls),
        1 => set(ui, c, st, body),
        2 => sliders(ui, c, st, body),
        4 => history(ui, c, st, body),
        5 => approximate(ui, c, st, body),
        _ => {
            ui.centered_and_justified(|ui| ui.label("Not available yet"));
        }
    });
}

fn wheel(ui: &mut Ui, c: &Colors, st: &mut ColourState, body: Rect, hls: bool) {
    let d = (body.width() - 18.0).min(body.height() - 18.0).min(190.0);
    let center = pos2(body.center().x, body.top() + 6.0 + d / 2.0);
    let (r_out, r_in) = (d / 2.0, d / 2.0 - 16.0);
    // Hue ring.
    let mut mesh = Mesh::default();
    let n = 120;
    for i in 0..=n {
        let h = 360.0 * i as f32 / n as f32;
        let a = hue_angle(h);
        let col = c32(hsv_to_rgb([h, 1.0, 1.0]));
        let dir = vec2(a.cos(), a.sin());
        mesh.colored_vertex(center + dir * r_out, col);
        mesh.colored_vertex(center + dir * r_in, col);
        if i > 0 {
            let k = (2 * i) as u32;
            mesh.add_triangle(k - 2, k - 1, k);
            mesh.add_triangle(k - 1, k + 1, k);
        }
    }
    ui.painter().add(Shape::mesh(mesh));
    let hue_rgb = hsv_to_rgb([st.hsv[0], 1.0, 1.0]);
    let side = (r_in * 2.0 / 2f32.sqrt() - 4.0).floor();
    let sq = Rect::from_center_size(center, vec2(side, side));
    // HLS: a triangle with the pure hue at the right, white top-left, black bottom-left (U-013).
    let tri_r = r_in - 4.0;
    let tri = [
        center + vec2(tri_r, 0.0),
        center + vec2(-tri_r * 0.5, -tri_r * 0.866),
        center + vec2(-tri_r * 0.5, tri_r * 0.866),
    ];
    if hls {
        let mut mesh = Mesh::default();
        mesh.colored_vertex(tri[0], c32(hue_rgb));
        mesh.colored_vertex(tri[1], Color32::WHITE);
        mesh.colored_vertex(tri[2], Color32::BLACK);
        mesh.add_triangle(0, 1, 2);
        ui.painter().add(Shape::mesh(mesh));
    } else {
        // Saturation (left → right) / value (top → bottom), bilinear in RGB (exact for HSV at a
        // fixed hue).
        let mut mesh = Mesh::default();
        let k = 16;
        for y in 0..=k {
            for x in 0..=k {
                let (s, v) = (x as f32 / k as f32, 1.0 - y as f32 / k as f32);
                mesh.colored_vertex(
                    sq.min + vec2(s * side, (1.0 - v) * side),
                    c32(hsv_to_rgb([st.hsv[0], s, v])),
                );
                if x > 0 && y > 0 {
                    let i = (y * (k + 1) + x) as u32;
                    let w = (k + 1) as u32;
                    mesh.add_triangle(i - w - 1, i - w, i);
                    mesh.add_triangle(i - w - 1, i, i - 1);
                }
            }
        }
        ui.painter().add(Shape::mesh(mesh));
    }
    // Markers.
    let a = hue_angle(st.hsv[0]);
    let hm = center + vec2(a.cos(), a.sin()) * (r_out + r_in) / 2.0;
    ui.painter().rect_stroke(
        Rect::from_center_size(hm, vec2(10.0, 6.0)),
        0.0,
        Stroke::new(2.0, Color32::WHITE),
        egui::StrokeKind::Middle,
    );
    let cur = hsv_to_rgb(st.hsv).map(|v| f32::from(v) / 255.0);
    let (mx, mn) = (
        cur[0].max(cur[1]).max(cur[2]),
        cur[0].min(cur[1]).min(cur[2]),
    );
    let sm = if hls {
        // Barycentric weights: hue = max − min, white = min, black = 1 − max.
        let (wh, ww, wb) = (mx - mn, mn, 1.0 - mx);
        pos2(
            tri[0].x * wh + tri[1].x * ww + tri[2].x * wb,
            tri[0].y * wh + tri[1].y * ww + tri[2].y * wb,
        )
    } else {
        sq.min + vec2(st.hsv[1] * side, (1.0 - st.hsv[2]) * side)
    };
    let light = (mx + mn) / 2.0;
    ui.painter().circle_stroke(
        sm,
        4.0,
        Stroke::new(
            1.0,
            if light > 0.5 {
                Color32::BLACK
            } else {
                Color32::WHITE
            },
        ),
    );
    // Interaction.
    let wr = Rect::from_center_size(center, vec2(d, d));
    let resp = ui.interact(wr, ui.scope_id().with("wheel"), Sense::click_and_drag());
    let drag_id = ui.scope_id().with("wheel-part");
    let in_inner = |q: Pos2| {
        if hls {
            (q - center).length() < tri_r
        } else {
            sq.contains(q)
        }
    };
    if (resp.drag_started() || resp.clicked())
        && let Some(q) = resp.interact_pointer_pos()
    {
        let part = if in_inner(q) {
            1u8
        } else if (q - center).length() >= r_in - 2.0 {
            0
        } else {
            2
        };
        ui.data_mut(|dd| dd.insert_temp(drag_id, part));
    }
    if (resp.dragged() || resp.clicked())
        && let (Some(q), Some(part)) = (
            resp.interact_pointer_pos(),
            ui.data(|dd| dd.get_temp::<u8>(drag_id)),
        )
    {
        let mut hsv = st.hsv;
        if part == 0 {
            let v = q - center;
            let ang = v.y.atan2(v.x).to_degrees();
            hsv[0] = (ang - 180.0).rem_euclid(360.0);
            st.set_hsv(hsv);
        } else if part == 1 && hls {
            // Barycentric coordinates of q, clamped into the triangle.
            let (p0, p1, p2) = (tri[0], tri[1], tri[2]);
            let den = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
            let wh = ((p1.y - p2.y) * (q.x - p2.x) + (p2.x - p1.x) * (q.y - p2.y)) / den;
            let ww = ((p2.y - p0.y) * (q.x - p2.x) + (p0.x - p2.x) * (q.y - p2.y)) / den;
            let (wh, ww) = (wh.clamp(0.0, 1.0), ww.clamp(0.0, 1.0));
            let (wh, ww) = if wh + ww > 1.0 {
                (wh / (wh + ww), ww / (wh + ww))
            } else {
                (wh, ww)
            };
            let rgb = hue_rgb.map(|h| {
                ((f32::from(h) / 255.0 * wh + ww) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            });
            let keep = st.hsv[0];
            st.set(rgb);
            st.hsv[0] = keep;
        } else if part == 1 {
            hsv[1] = ((q.x - sq.left()) / side).clamp(0.0, 1.0);
            hsv[2] = 1.0 - ((q.y - sq.top()) / side).clamp(0.0, 1.0);
            st.set_hsv(hsv);
        }
    }
    // Chips bottom-left, H/S/V readout at the bottom.
    chips(
        ui,
        c,
        st,
        pos2(body.left() + 6.0, body.bottom() - 46.0),
        18.0,
    );
    let readout = if hls {
        let s_hls = if (mx - mn).abs() < 1e-6 {
            0.0
        } else {
            (mx - mn) / (1.0 - (2.0 * light - 1.0).abs())
        };
        format!(
            "H {:.0}  L {:.0}  S {:.0}",
            st.hsv[0],
            light * 100.0,
            s_hls * 100.0
        )
    } else {
        format!(
            "H {:.0}  S {:.0}  V {:.0}",
            st.hsv[0],
            st.hsv[1] * 100.0,
            st.hsv[2] * 100.0
        )
    };
    ui.painter().text(
        pos2(body.right() - 8.0, body.bottom() - 8.0),
        egui::Align2::RIGHT_CENTER,
        readout,
        egui::FontId::proportional(10.0),
        c.text,
    );
}

/// Brushye's own default colour set: a grey row, then twelve hues at four lightness levels.
fn default_set() -> Vec<Option<[u8; 3]>> {
    let mut v: Vec<Option<[u8; 3]>> = (0..11).map(|i| Some([(i * 255 / 10) as u8; 3])).collect();
    v.push(None);
    for (s, val) in [(0.35, 1.0), (0.7, 1.0), (1.0, 0.9), (1.0, 0.55)] {
        for i in 0..12 {
            v.push(Some(hsv_to_rgb([i as f32 * 30.0, s, val])));
        }
    }
    v
}

fn set(ui: &mut Ui, c: &Colors, st: &mut ColourState, body: Rect) {
    let cell = 16.0;
    let cols = ((body.width() - 8.0) / (cell + 1.0)).floor().max(1.0) as usize;
    for (i, sw) in default_set().iter().enumerate() {
        let (x, y) = ((i % cols) as f32, (i / cols) as f32);
        let r = Rect::from_min_size(
            body.min + vec2(4.0 + x * (cell + 1.0), 4.0 + y * (cell + 1.0)),
            vec2(cell, cell),
        );
        match sw {
            Some(col) => {
                ui.painter().rect_filled(r, 0.0, c32(*col));
            }
            None => checker(ui, r, 4.0),
        }
        let resp = ui.interact(r, ui.scope_id().with(("swatch", i)), Sense::click());
        if resp.hovered() {
            ui.painter()
                .rect_stroke(r, 0.0, Stroke::new(1.0, c.text), egui::StrokeKind::Inside);
        }
        if resp.clicked() {
            match sw {
                Some(col) => st.set(*col),
                None => st.select(Chip::Transparent),
            }
        }
    }
}

fn sliders(ui: &mut Ui, c: &Colors, st: &mut ColourState, body: Rect) {
    // Vertical mode tabs at the left edge (U-005).
    let modes = [
        (SliderMode::Rgb, "RGB"),
        (SliderMode::Hsv, "HSV"),
        (SliderMode::Cmy, "CMY"),
    ];
    for (i, (m, name)) in modes.iter().enumerate() {
        let r = Rect::from_min_size(
            body.min + vec2(2.0, 4.0 + i as f32 * 34.0),
            vec2(16.0, 32.0),
        );
        ui.painter().rect_filled(
            r,
            0.0,
            if st.slider_mode == *m {
                c.accent
            } else {
                c.header
            },
        );
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            &name[..1],
            egui::FontId::proportional(10.0),
            c.text,
        );
        if ui
            .interact(r, ui.scope_id().with(("mode", i)), Sense::click())
            .on_hover_text(*name)
            .clicked()
        {
            st.slider_mode = *m;
        }
    }
    let area = Rect::from_min_max(
        body.min + vec2(24.0, 6.0),
        pos2(body.right() - 6.0, body.bottom() - 50.0),
    );
    widgets::area(ui, area, |ui| {
        let rgb = st.active_rgb();
        match st.slider_mode {
            SliderMode::Rgb => {
                let mut v = rgb.map(i32::from);
                let mut changed = false;
                for (i, l) in ["R", "G", "B"].iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(*l);
                        changed |= ui.add(egui::Slider::new(&mut v[i], 0..=255)).changed();
                    });
                }
                if changed {
                    st.set(v.map(|x| x as u8));
                }
            }
            SliderMode::Hsv => {
                let mut h = st.hsv;
                let (mut s, mut val) = (h[1] * 100.0, h[2] * 100.0);
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("H");
                    changed |= ui
                        .add(egui::Slider::new(&mut h[0], 0.0..=359.0).fixed_decimals(0))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("S");
                    changed |= ui
                        .add(egui::Slider::new(&mut s, 0.0..=100.0).fixed_decimals(0))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("V");
                    changed |= ui
                        .add(egui::Slider::new(&mut val, 0.0..=100.0).fixed_decimals(0))
                        .changed();
                });
                if changed {
                    st.set_hsv([h[0], s / 100.0, val / 100.0]);
                }
            }
            SliderMode::Cmy => {
                let mut v = rgb.map(|x| 255 - i32::from(x));
                let mut changed = false;
                for (i, l) in ["C", "M", "Y"].iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(*l);
                        changed |= ui.add(egui::Slider::new(&mut v[i], 0..=255)).changed();
                    });
                }
                if changed {
                    st.set(v.map(|x| (255 - x) as u8));
                }
            }
        }
    });
    chips(
        ui,
        c,
        st,
        pos2(body.left() + 26.0, body.bottom() - 44.0),
        22.0,
    );
}

fn history(ui: &mut Ui, c: &Colors, st: &mut ColourState, body: Rect) {
    let cell = 14.0;
    let cols = ((body.width() - 8.0) / (cell + 1.0)).floor().max(1.0) as usize;
    let hist = st.history.clone();
    if hist.is_empty() {
        ui.painter().text(
            body.center(),
            egui::Align2::CENTER_CENTER,
            "No colours used yet",
            egui::FontId::proportional(11.0),
            c.text_dim,
        );
    }
    for (i, col) in hist.iter().enumerate() {
        let r = Rect::from_min_size(
            body.min
                + vec2(
                    4.0 + (i % cols) as f32 * (cell + 1.0),
                    4.0 + (i / cols) as f32 * (cell + 1.0),
                ),
            vec2(cell, cell),
        );
        ui.painter().rect_filled(r, 0.0, c32(*col));
        if st.drawing() == Some(*col) {
            ui.painter().rect_stroke(
                r,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(230, 50, 50)),
                egui::StrokeKind::Inside,
            );
        }
        if ui
            .interact(r, ui.scope_id().with(("hist", i)), Sense::click())
            .clicked()
        {
            st.set(*col);
        }
    }
}

fn approximate(ui: &mut Ui, c: &Colors, st: &mut ColourState, body: Rect) {
    // A grid of value (columns) and saturation (rows) steps around the current colour.
    let n = 13;
    let side = ((body.width() - 30.0).min(body.height() - 14.0) / n as f32).floor();
    let origin = body.min + vec2(24.0, 6.0);
    let base = st.hsv;
    let mut pick = None;
    for row in 0..n {
        for col in 0..n {
            let dv = (col as f32 - 6.0) * 0.06;
            let ds = (row as f32 - 6.0) * 0.06;
            let hsv = [
                base[0],
                (base[1] + ds).clamp(0.0, 1.0),
                (base[2] + dv).clamp(0.0, 1.0),
            ];
            let rgb = hsv_to_rgb(hsv);
            let r = Rect::from_min_size(
                origin + vec2(col as f32 * side, row as f32 * side),
                vec2(side, side),
            );
            ui.painter().rect_filled(r, 0.0, c32(rgb));
            if row == 6 && col == 6 {
                ui.painter().rect_stroke(
                    r,
                    0.0,
                    Stroke::new(1.0, c.text),
                    egui::StrokeKind::Inside,
                );
            }
            if ui
                .interact(r, ui.scope_id().with(("approx", row, col)), Sense::click())
                .clicked()
            {
                pick = Some(rgb);
            }
        }
    }
    ui.painter().text(
        body.min + vec2(4.0, 10.0),
        egui::Align2::LEFT_CENTER,
        "S",
        egui::FontId::proportional(10.0),
        c.text,
    );
    ui.painter().text(
        origin + vec2(side * n as f32 / 2.0, side * n as f32 + 6.0),
        egui::Align2::CENTER_CENTER,
        "V",
        egui::FontId::proportional(10.0),
        c.text,
    );
    if let Some(p) = pick {
        st.set(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips() {
        for c in [
            [255, 0, 0],
            [0, 255, 0],
            [12, 34, 56],
            [200, 200, 10],
            [0, 0, 0],
            [255, 255, 255],
        ] {
            assert_eq!(hsv_to_rgb(rgb_to_hsv(c)), c);
        }
        // Red sits at the left of the wheel, cyan at the right (U-005).
        assert!((hue_angle(0.0).cos() + 1.0).abs() < 1e-6);
        assert!((hue_angle(180.0).cos() - 1.0).abs() < 1e-6);
        // Blue (240) at the bottom: +y on screen.
        assert!((hue_angle(240.0 + 30.0).sin() - 1.0).abs() < 1e-6);
    }
}
