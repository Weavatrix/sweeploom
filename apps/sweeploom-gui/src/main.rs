//! SweepLoom desktop UI.

mod app;
mod autostart;
mod chrome;
mod format;
mod live;
mod nav;
mod prefs;
mod review_extra;
mod scan_job;
mod screens;
mod sort;
mod theme;
mod tray;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let start_hidden = std::env::args().any(|item| item == "--tray");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([880.0, 600.0])
            .with_title("SweepLoom")
            .with_visible(!start_hidden),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "SweepLoom",
        options,
        Box::new(move |cc| Ok(Box::new(app::SweepLoomApp::new(cc, start_hidden)))),
    )
}
