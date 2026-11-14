//! UI fonts: the Go fonts (BSD-3-Clause) for text and Phosphor (MIT) for icons, all bundled
//! (`assets/fonts`). egui's own default fonts are not used.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// Font family name for the filled Phosphor icons (selected/active states).
pub const ICONS_FILL: &str = "phosphor-fill";

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::empty();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    add("go", include_bytes!("../assets/fonts/Go-Regular.ttf"));
    add("go-medium", include_bytes!("../assets/fonts/Go-Medium.ttf"));
    add("go-mono", include_bytes!("../assets/fonts/Go-Mono.ttf"));
    add("phosphor", include_bytes!("../assets/fonts/Phosphor.ttf"));
    add(
        "phosphor-fill",
        include_bytes!("../assets/fonts/Phosphor-Fill.ttf"),
    );
    fonts.families.insert(
        FontFamily::Proportional,
        vec!["go".into(), "phosphor".into()],
    );
    fonts.families.insert(
        FontFamily::Monospace,
        vec!["go-mono".into(), "phosphor".into()],
    );
    fonts.families.insert(
        FontFamily::Name(ICONS_FILL.into()),
        vec!["phosphor-fill".into(), "go".into()],
    );
    fonts.families.insert(
        FontFamily::Name("medium".into()),
        vec!["go-medium".into(), "phosphor".into()],
    );
    ctx.set_fonts(fonts);
}
