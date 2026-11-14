//! A pressure curve editor drawn like CSP's dynamics graph (spec U-010): a 4×4 grid, control
//! points as small blue squares, the curve (as the engine evaluates it) in white, and the
//! control polygon when the curve isn't linear. Click empty space to add a point, drag to move
//! it, drag it out of the graph (or right-click it) to remove it. The ends (0,0) and (1,1) are
//! fixed.

use brushye_command::core::brush::Curve;
use eframe::egui;

/// Edit `points` in place; returns true when they changed. Invalid edits (points crossing
/// in x) are prevented by clamping each point between its neighbours.
pub fn curve_editor(
    ui: &mut egui::Ui,
    id: egui::Id,
    points: &mut Vec<[f64; 2]>,
    side: f32,
) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    painter.rect_filled(rect, 0.0, visuals.extreme_bg_color);
    let grid = egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
    for i in 1..4 {
        let t = i as f32 / 4.0;
        painter.line_segment(
            [
                egui::pos2(rect.left() + t * side, rect.top()),
                egui::pos2(rect.left() + t * side, rect.bottom()),
            ],
            grid,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top() + t * side),
                egui::pos2(rect.right(), rect.top() + t * side),
            ],
            grid,
        );
    }
    let to_screen = |p: [f64; 2]| {
        egui::pos2(
            rect.left() + p[0] as f32 * side,
            rect.bottom() - p[1] as f32 * side,
        )
    };
    let to_curve = |q: egui::Pos2| {
        [
            f64::from(((q.x - rect.left()) / side).clamp(0.0, 1.0)),
            f64::from(((rect.bottom() - q.y) / side).clamp(0.0, 1.0)),
        ]
    };
    let mut changed = false;
    let dragging_id = id.with("drag");
    let mut dragging: Option<usize> = ui.data(|d| d.get_temp(dragging_id));
    let hit =
        |q: egui::Pos2, pts: &[[f64; 2]]| pts.iter().position(|&p| to_screen(p).distance(q) < 8.0);
    if let Some(q) = response.interact_pointer_pos() {
        if response.drag_started() {
            dragging = hit(q, points);
        }
        if response.secondary_clicked() {
            if let Some(i) = hit(q, points) {
                points.remove(i);
                changed = true;
            }
        } else if response.clicked() && hit(q, points).is_none() {
            let p = to_curve(q);
            if p[0] > 0.0 && p[0] < 1.0 && !points.iter().any(|x| (x[0] - p[0]).abs() < 0.01) {
                points.push(p);
                points.sort_by(|a, b| a[0].total_cmp(&b[0]));
                changed = true;
            }
        }
        // Dragged out of the graph: the point is removed (U-010).
        if let (Some(i), Some(q)) = (dragging, response.interact_pointer_pos())
            && i < points.len()
            && !rect.expand(6.0).contains(q)
        {
            points.remove(i);
            dragging = None;
            changed = true;
        }
        if let Some(i) = dragging.filter(|&i| i < points.len()) {
            let p = to_curve(q);
            let lo = if i == 0 { 0.0 } else { points[i - 1][0] } + 0.005;
            let hi = if i + 1 == points.len() {
                1.0
            } else {
                points[i + 1][0]
            } - 0.005;
            let np = [p[0].clamp(lo, hi), p[1]];
            if np != points[i] {
                points[i] = np;
                changed = true;
            }
        }
    }
    if response.drag_stopped() {
        dragging = None;
    }
    ui.data_mut(|d| {
        if let Some(i) = dragging {
            d.insert_temp(dragging_id, i);
        } else {
            d.remove::<usize>(dragging_id);
        }
    });
    // The curve, as the engine evaluates it.
    let curve = Curve::new(points.clone()).unwrap_or_default();
    let line: Vec<egui::Pos2> = (0..=100)
        .map(|i| {
            let x = f64::from(i) / 100.0;
            to_screen([x, curve.eval(x)])
        })
        .collect();
    let blue = egui::Color32::from_rgb(80, 140, 230);
    if !points.is_empty() {
        let mut poly = vec![to_screen([0.0, 0.0])];
        poly.extend(points.iter().map(|&p| to_screen(p)));
        poly.push(to_screen([1.0, 1.0]));
        painter.add(egui::Shape::line(poly, egui::Stroke::new(1.0, blue)));
    }
    painter.add(egui::Shape::line(
        line,
        egui::Stroke::new(1.5, egui::Color32::WHITE),
    ));
    for (i, &p) in points.iter().enumerate() {
        let col = if dragging == Some(i) {
            egui::Color32::from_rgb(230, 60, 60)
        } else {
            blue
        };
        painter.rect_filled(
            egui::Rect::from_center_size(to_screen(p), egui::vec2(6.0, 6.0)),
            0.0,
            col,
        );
    }
    response.on_hover_text("Specify output amount");
    changed
}
