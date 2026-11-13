//! M1 tablet spike app: logs the pen event stream and draws pressure-weighted strokes.
//!
//! Run: `cargo run -p brushye-tablet --example pen_log`
//! - Every pen event is printed to stdout as `PEN <event>` (for automated checks) and shown in
//!   the window's log.
//! - With delivery `Full` (Wayland), the pen is also injected into egui as a pointer, so it can
//!   press the window's buttons.
//! - Set `PEN_LOG_SECONDS=<n>` to exit automatically after n seconds.

use std::sync::Arc;
use std::time::Instant;

use brushye_tablet::{Delivery, PenEvent, PenPose, Tablet};
use eframe::egui;

struct App {
    tablet: Result<Tablet, String>,
    log: Vec<String>,
    strokes: Vec<Vec<PenPose>>,
    current: Option<Vec<PenPose>>,
    last: Option<PenPose>,
    started: Instant,
    exit_after: Option<f64>,
    events: usize,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let tablet = Tablet::new(cc, cc).map_err(|e| e.to_string());
        match &tablet {
            Ok(t) => {
                println!("PEN backend={} delivery={:?}", t.backend(), t.delivery());
                let ctx = cc.egui_ctx.clone();
                t.set_waker(Arc::new(move || ctx.request_repaint()));
            }
            Err(e) => println!("PEN error={e}"),
        }
        let exit_after = std::env::var("PEN_LOG_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok());
        Self {
            tablet,
            log: Vec::new(),
            strokes: Vec::new(),
            current: None,
            last: None,
            started: Instant::now(),
            exit_after,
            events: 0,
        }
    }

    fn handle(&mut self, ev: PenEvent) {
        self.events += 1;
        println!("PEN {ev:?}");
        self.log.push(format!("{ev:?}"));
        if self.log.len() > 200 {
            self.log.remove(0);
        }
        match ev {
            PenEvent::Pose(p) => {
                self.last = Some(p);
                if let Some(s) = self.current.as_mut() {
                    s.push(p);
                }
            }
            PenEvent::Down => self.current = Some(self.last.into_iter().collect()),
            PenEvent::Up | PenEvent::Out => {
                if let Some(s) = self.current.take() {
                    self.strokes.push(s);
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let Ok(tablet) = &self.tablet else { return };
        tablet.set_scale_factor(f64::from(ctx.pixels_per_point()));
        let mut evs = Vec::new();
        tablet.poll(&mut evs);
        let full = tablet.delivery() == Delivery::Full;
        for ev in evs {
            if full {
                // Feed the pen into the UI: it no longer arrives as a mouse.
                match ev {
                    PenEvent::Pose(p) => raw_input.events.push(egui::Event::PointerMoved(
                        egui::pos2(p.x as f32, p.y as f32),
                    )),
                    PenEvent::Down | PenEvent::Up => {
                        if let Some(p) = self.last {
                            raw_input.events.push(egui::Event::PointerButton {
                                pos: egui::pos2(p.x as f32, p.y as f32),
                                button: egui::PointerButton::Primary,
                                pressed: ev == PenEvent::Down,
                                modifiers: egui::Modifiers::default(),
                            });
                        }
                    }
                    PenEvent::Out => raw_input.events.push(egui::Event::PointerGone),
                    _ => {}
                }
            }
            self.handle(ev);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Some(secs) = self.exit_after {
            if self.started.elapsed().as_secs_f64() > secs {
                println!("PEN done events={}", self.events);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        egui::Panel::left("log").min_size(360.0).show(ui, |ui| {
            match &self.tablet {
                Ok(t) => ui.label(format!(
                    "backend: {}   delivery: {:?}",
                    t.backend(),
                    t.delivery()
                )),
                Err(e) => ui.colored_label(egui::Color32::RED, e),
            };
            if let Some(p) = self.last {
                ui.label(format!(
                    "x {:.2}  y {:.2}  pressure {:?}  tilt {:?}  twist {:?}  t {} µs",
                    p.x, p.y, p.pressure, p.tilt, p.twist, p.time_us
                ));
            }
            if ui.button("Clear").clicked() {
                self.strokes.clear();
                self.log.clear();
            }
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for l in &self.log {
                        ui.monospace(l);
                    }
                });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let painter = ui.painter();
            let all = self.strokes.iter().chain(self.current.iter());
            for s in all {
                // Each segment is drawn with a round cap at its end (a filled circle), so
                // thick segments meeting at an angle leave no wedge-shaped gaps at bends.
                // (A trace of the input, not Brushye's brush engine.)
                let width = |p: &PenPose| 1.0 + 24.0 * p.pressure.unwrap_or(1.0);
                if let Some(first) = s.first() {
                    painter.circle_filled(
                        egui::pos2(first.x as f32, first.y as f32),
                        width(first) / 2.0,
                        egui::Color32::WHITE,
                    );
                }
                for w in s.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let pb = egui::pos2(b.x as f32, b.y as f32);
                    painter.line_segment(
                        [egui::pos2(a.x as f32, a.y as f32), pb],
                        egui::Stroke::new(width(&b), egui::Color32::WHITE),
                    );
                    painter.circle_filled(pb, width(&b) / 2.0, egui::Color32::WHITE);
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Brushye pen log (M1 spike)",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
