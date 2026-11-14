//! `brushye-app [FILE.brsh]` — the Brushye desktop app.

use brushye_app::app::App;
use eframe::egui;

fn main() -> eframe::Result {
    let open = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    // BRUSHYE_WINDOW_SIZE=1920x1080 sets the initial size (screenshots, layout checks).
    let size = std::env::var("BRUSHYE_WINDOW_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some([w.parse().ok()?, h.parse().ok()?])
        })
        .unwrap_or([1600.0, 1000.0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_title("Brushye"),
        ..Default::default()
    };
    eframe::run_native(
        "Brushye",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, open)))),
    )
}
