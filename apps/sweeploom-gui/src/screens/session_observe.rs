//! Observed RAM/CPU/disk for a selected session. Never back-filled.

use eframe::egui::{self, RichText};
use sweeploom_core::LiveSession;
use sweeploom_history::{fold_recent, summarize_cpu, summarize_rss};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

use super::session_pressure;

pub fn draw(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    draw_share(app, ui, session);
    draw_history(app, ui, session);
    ui.label(format!(
        "Disk this interval  read {}  write {}",
        format_bytes(session.disk.read_bytes),
        format_bytes(session.disk.write_bytes)
    ));
    draw_network(ui, session);
}

fn draw_share(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let Some(snapshot) = &app.snapshot else {
        return;
    };
    ui.add(
        egui::Label::new(
            RichText::new(session_pressure::share_line(
                session,
                snapshot.memory.used_bytes,
                snapshot.memory.total_bytes,
                snapshot.cpu.usage_percent,
            ))
            .color(crate::theme::muted(ui)),
        )
        .wrap(),
    );
}

fn draw_history(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let series: Vec<_> = session
        .processes
        .iter()
        .filter_map(|key| app.history.get(*key).map(|hist| hist.fast.chrono()))
        .collect();
    if series.is_empty() {
        ui.label("CPU/RAM history starts when SweepLoom first sees these processes.");
        return;
    }
    let folded = fold_recent(&series, 40);
    let Some(last) = folded.last() else {
        return;
    };
    let rss = summarize_rss(&folded);
    let cpu = summarize_cpu(&folded, &[], last.at_unix_ms);
    ui.label(format!(
        "Session RAM observed {} → {} (peak {}) over {} samples · now {}",
        format_bytes(rss.first),
        format_bytes(rss.now),
        format_bytes(rss.peak),
        rss.fast_samples,
        format_bytes(session.rss_bytes)
    ));
    ui.horizontal(|ui| {
        ui.label(format!(
            "Session CPU now {:.1}%  peak {:.1}%  {}",
            session.cpu_percent,
            cpu.peak,
            avg_label("5m", cpu.avg_5m)
        ));
        spark(ui, &folded, |item| item.cpu_percent);
    });
    ui.horizontal(|ui| {
        ui.label("RAM");
        spark(ui, &folded, |item| item.rss_bytes as f32);
        ui.label(avg_label("1h", cpu.avg_1h));
    });
}

fn spark(
    ui: &mut egui::Ui,
    folded: &[sweeploom_history::Sample],
    pick: fn(&sweeploom_history::Sample) -> f32,
) {
    let values: Vec<f32> = folded.iter().map(pick).collect();
    crate::widgets::sparkline(ui, &values, egui::vec2(140.0, 20.0), crate::theme::accent());
}

fn avg_label(window: &str, value: Option<f32>) -> String {
    match value {
        Some(cpu) => format!("CPU {window} avg {cpu:.1}%"),
        None => {
            format!("CPU {window} avg unavailable (not watched long enough; not shown as zero)")
        }
    }
}

fn draw_network(ui: &mut egui::Ui, session: &LiveSession) {
    if session.network.connections_available {
        if session.network.listening_ports.is_empty() {
            ui.label("Listening ports: none observed");
        } else {
            ui.label(format!(
                "Listening ports: {}",
                session
                    .network
                    .listening_ports
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    } else {
        ui.label("Listening ports unavailable on this OS (not shown as zero).");
    }
    if session.network.byte_rate_available {
        ui.label(format!(
            "Observed TCP  rx {}  tx {}  since SweepLoom started watching",
            format_bytes(session.network.observed_rx_bytes),
            format_bytes(session.network.observed_tx_bytes)
        ));
    } else {
        ui.label("Per-process TCP bytes unavailable (not shown as zero).");
    }
}
