//! A pressure curve editor: a square graph with the curve (as the engine evaluates it) and its
//! interior control points. Drag a point to move it, click empty space to add one, right-click
//! a point to remove it. The end points (0,0) and (1,1) are fixed.

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
    painter.add(egui::Shape::line(
        line,
        egui::Stroke::new(2.0, visuals.selection.bg_fill),
    ));
    for &p in points.iter() {
        painter.circle_filled(to_screen(p), 4.0, visuals.strong_text_color());
    }
    changed
}
