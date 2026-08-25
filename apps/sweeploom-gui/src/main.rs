//! SweepLoom desktop UI.

mod app;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("SweepLoom"),
        ..Default::default()
    };
    eframe::run_native(
        "SweepLoom",
        options,
        Box::new(|cc| Ok(Box::new(app::SweepLoomApp::new(cc)))),
    )
}
