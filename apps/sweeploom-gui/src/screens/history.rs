//! Observed process history. Never invented from before SweepLoom started.

use eframe::egui::RichText;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::page_title;

pub fn ui_history(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "History",
        "Rings start when SweepLoom first sees a process. Nothing is back-filled.",
    );
    ui.label(format!("Tracked processes: {}", app.history.len()));
    ui.add_space(8.0);
    let Some(snapshot) = &app.snapshot else {
        ui.label("No live snapshot yet.");
        return;
    };
    let mut ranked: Vec<_> = snapshot.processes.iter().collect();
    ranked.sort_by_key(|item| std::cmp::Reverse(item.rss_bytes));
    for process in ranked.into_iter().take(12) {
        let Some(hist) = app.history.get(process.key) else {
            continue;
        };
        let samples = hist.fast.chrono();
        if samples.is_empty() {
            continue;
        }
        let peak = samples.iter().map(|item| item.rss_bytes).max().unwrap_or(0);
        let cpu = samples
            .iter()
            .map(|item| item.cpu_percent)
            .fold(0.0_f32, f32::max);
        ui.label(
            RichText::new(format!(
                "{}  pid {}  now {}  peak {}  cpu max {:.1}%  samples {}",
                process.name,
                process.pid,
                format_bytes(process.rss_bytes),
                format_bytes(peak),
                cpu,
                samples.len()
            ))
            .size(16.0),
        );
    }
}
