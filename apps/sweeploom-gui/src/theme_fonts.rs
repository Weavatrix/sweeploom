//! Prefer the platform UI face (San Francisco on macOS); egui's bundled fonts stay as fallback.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

const FACES: [(&str, &str, FontFamily); 2] = [
    (
        "system-ui",
        "/System/Library/Fonts/SFNS.ttf",
        FontFamily::Proportional,
    ),
    (
        "system-mono",
        "/System/Library/Fonts/SFNSMono.ttf",
        FontFamily::Monospace,
    ),
];

/// Install once per context. Missing files leave egui's defaults untouched.
pub(super) fn install(ctx: &egui::Context) {
    let id = egui::Id::new("sweeploom-fonts-installed");
    if ctx.data(|data| data.get_temp::<bool>(id)).unwrap_or(false) {
        return;
    }
    ctx.data_mut(|data| data.insert_temp(id, true));
    let mut fonts = FontDefinitions::default();
    let mut any = false;
    for (name, path, family) in FACES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.to_owned());
        any = true;
    }
    if any {
        ctx.set_fonts(fonts);
    }
}
