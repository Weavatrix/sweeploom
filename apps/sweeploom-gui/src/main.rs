//! SweepLoom desktop UI.

mod app;
mod app_tray;
mod autostart;
mod brand;
mod chrome;
mod format;
mod icons;
mod live;
mod mark;
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
            .with_icon(window_icon())
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

fn window_icon() -> egui::IconData {
    let (rgba, width, height) = mark::rgba(32);
    egui::IconData {
        rgba,
        width,
        height,
    }
}
