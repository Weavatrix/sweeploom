//! Observed process history. Never invented from before SweepLoom started.

use sweeploom_history::summarize_cpu;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::{list_row, page_title};

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
        let fast = hist.fast.chrono();
        let Some(last) = fast.last() else {
            continue;
        };
        let peak = fast.iter().map(|item| item.rss_bytes).max().unwrap_or(0);
        let cpu = summarize_cpu(&fast, &hist.slow.chrono(), last.at_unix_ms);
        list_row(
            ui,
            &process.name,
            &format!(
                "pid {} · {} · peak {}",
                process.pid,
                format_bytes(process.rss_bytes),
                format_bytes(peak)
            ),
            &format!(
                "cpu {:.1}% · {} · {} samples",
                cpu.now,
                avg_short(cpu.avg_5m),
                fast.len()
            ),
        );
    }
}

fn avg_short(value: Option<f32>) -> String {
    match value {
        Some(cpu) => format!("5m {cpu:.1}%"),
        None => "5m unavailable".to_owned(),
    }
}
