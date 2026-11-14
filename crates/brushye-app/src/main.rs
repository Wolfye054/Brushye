//! `brushye-app [FILE.brsh]` — the Brushye desktop app.

use brushye_app::app::App;
use eframe::egui;

fn main() -> eframe::Result {
    let open = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Brushye"),
        ..Default::default()
    };
    eframe::run_native(
        "Brushye",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, open)))),
    )
}
