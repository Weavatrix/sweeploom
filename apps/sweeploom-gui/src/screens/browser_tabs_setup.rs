//! Empty / error companion state: Edge needs an unpacked extension + native host.

use sweeploom_browser::{
    install_native_host, is_chromium_extension_id, load_chromium_id, sibling_host_exe,
};

use crate::app::SweepLoomApp;
use crate::theme;
use crate::widgets::pointer;
use eframe::egui::RichText;

pub fn draw_missing(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, path: &std::path::Path) {
    fill_saved_id(app);
    ui.label(
        RichText::new("Tabs need URLs from the SweepLoom companion. Process trees are not tabs.")
            .strong(),
    );
    ui.label(
        RichText::new(format!("No snapshot yet at {}", path.display())).color(theme::muted(ui)),
    );
    draw_steps(app, ui);
}

pub fn draw_error(
    app: &mut SweepLoomApp,
    ui: &mut eframe::egui::Ui,
    path: &std::path::Path,
    error: &std::io::Error,
) {
    fill_saved_id(app);
    ui.label(RichText::new(format!("Could not read companion snapshot: {error}")).strong());
    ui.label(RichText::new(path.display().to_string()).color(theme::muted(ui)));
    draw_steps(app, ui);
}

fn fill_saved_id(app: &mut SweepLoomApp) {
    if !app.browser.chromium_id.is_empty() {
        return;
    }
    if let Ok(Some(id)) = load_chromium_id(&app.locations.app_data) {
        app.browser.chromium_id = id;
    }
}

fn draw_steps(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    ui.add_space(8.0);
    ui.label("1. Edge > Extensions > Developer mode > Load unpacked > browser/chromium-extension");
    ui.label("2. Copy the extension ID (32 letters a–p) and register the native host for Edge.");
    ui.label("3. Reload the extension. Close is never sent.");
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label("Extension ID");
        ui.add(
            eframe::egui::TextEdit::singleline(&mut app.browser.chromium_id)
                .desired_width(280.0)
                .hint_text("abcdefghijklmnopabcdefghijklmnop"),
        );
    });
    ui.horizontal_wrapped(|ui| {
        if pointer(ui.button("Register Edge host")).clicked() {
            register_host(app);
        }
    });
    if let Some(message) = &app.action_message {
        ui.label(message.clone());
    }
}

fn register_host(app: &mut SweepLoomApp) {
    let id = app.browser.chromium_id.trim().to_owned();
    if !is_chromium_extension_id(&id) {
        app.action_message = Some(
            "Extension ID is 32 letters a–p. Turn on Developer mode on edge://extensions.".into(),
        );
        return;
    }
    let host = match sibling_host_exe() {
        Ok(path) => path,
        Err(error) => {
            app.action_message = Some(format!("Could not locate companion host: {error}"));
            return;
        }
    };
    if !host.is_file() {
        app.action_message = Some(format!(
            "Missing {}. Build it with: cargo build -p sweeploom-cli --bins",
            host.display()
        ));
        return;
    }
    app.action_message = Some(
        match install_native_host(&app.locations.app_data, &host, Some(&id)) {
            Ok(_) => {
                "Edge native host registered. Reload SweepLoom Companion on edge://extensions."
                    .into()
            }
            Err(error) => format!("Could not register native host: {error}"),
        },
    );
}
