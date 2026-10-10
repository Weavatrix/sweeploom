//! Observed RAM/CPU/disk for a selected session. Never back-filled.

use eframe::egui::{self, RichText};
use sweeploom_core::LiveSession;
use sweeploom_history::{fold_recent, summarize_cpu, summarize_rss};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::theme;
use crate::widgets::Segment;

pub fn draw(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    draw_history(app, ui, session);
    if let Some(snapshot) = &app.snapshot {
        let cores = crate::widgets::cpu_cores::history(ui.ctx());
        crate::widgets::pair(
            ui,
            ("session-host", session.id.0),
            crate::widgets::panel_frame,
            |ui| host_memory(ui, session, snapshot),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Host CPU").size(13.5).strong());
                    ui.label(
                        RichText::new(format!("{:.0}%", snapshot.cpu.usage_percent))
                            .size(13.5)
                            .strong()
                            .color(theme::load(ui, snapshot.cpu.usage_percent)),
                    )
                    .on_hover_text(super::session_pressure::share_line(
                        session,
                        snapshot.memory.used_bytes,
                        snapshot.memory.total_bytes,
                        snapshot.cpu.usage_percent,
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        crate::widgets::caption(
                            ui,
                            format!("{} · last {} s", cores.summary(), cores.total.len()),
                        );
                    });
                });
                ui.add_space(theme::SM);
                crate::widgets::cpu_cores::heat_strip(ui, &cores);
            },
        );
    }
    crate::widgets::panel(ui, "Disk & network", |ui| {
        egui::Grid::new(("session-activity", session.id.0))
            .num_columns(2)
            .spacing([theme::LG, 6.0])
            .show(ui, |ui| {
                super::session_detail::field(
                    ui,
                    "Disk this interval",
                    format!(
                        "Read {} · Write {}",
                        format_bytes(session.disk.read_bytes),
                        format_bytes(session.disk.write_bytes)
                    ),
                );
                draw_network(ui, session);
            });
    });
}

fn host_memory(
    ui: &mut egui::Ui,
    session: &LiveSession,
    snapshot: &sweeploom_process::ProcessSnapshotSet,
) {
    let memory = snapshot.memory;
    let available = memory
        .available_bytes
        .max(memory.total_bytes.saturating_sub(memory.used_bytes));
    let in_use = memory.total_bytes.saturating_sub(available);
    let mine = session.rss_bytes.min(in_use);
    let segments = [
        Segment {
            label: "This session".into(),
            value: mine as f64,
            detail: format_bytes(session.rss_bytes),
            color: theme::series(ui, 0),
        },
        Segment {
            label: "Other in use".into(),
            value: in_use.saturating_sub(mine) as f64,
            detail: format_bytes(in_use.saturating_sub(mine)),
            color: theme::series_rest(ui),
        },
        Segment {
            label: "Available".into(),
            value: available as f64,
            detail: format_bytes(available),
            color: theme::inset(ui),
        },
    ];
    crate::widgets::breakdown(
        ui,
        "Host memory",
        &format!(
            "{} of {}",
            format_bytes(in_use),
            format_bytes(memory.total_bytes)
        ),
        &segments,
    );
}

fn draw_history(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let series: Vec<_> = session
        .processes
        .iter()
        .filter_map(|key| app.history.get(*key).map(|hist| hist.fast.chrono()))
        .collect();
    if series.len() != session.processes.len() {
        ui.label(
            RichText::new(format!(
                "History incomplete: {} of {} processes tracked. Live totals are shown in the table.",
                series.len(), session.processes.len()
            )).size(12.5).color(theme::muted(ui)),
        );
        ui.add_space(theme::SM);
        return;
    }
    let folded = fold_recent(&series, 40);
    let Some(last) = folded.last() else {
        ui.label(
            RichText::new("Collecting CPU and memory history…").color(crate::theme::muted(ui)),
        );
        return;
    };
    let rss = summarize_rss(&folded);
    let cpu = summarize_cpu(&folded, &[], last.at_unix_ms);
    crate::widgets::pair(
        ui,
        ("session-trends", session.id.0),
        crate::widgets::panel_frame,
        |ui| {
            trend(
                ui,
                "CPU",
                &format!("peak {:.1}%", cpu.peak),
                &folded,
                |item| item.cpu_percent,
                theme::series(ui, 0),
            );
            crate::widgets::caption(
                ui,
                format!("5m avg {} · 1h avg {}", avg_label(cpu.avg_5m), avg_label(cpu.avg_1h)),
            )
            .on_hover_text("An average stays unavailable until enough activity has been observed. Missing history is not zero usage.");
        },
        |ui| {
            trend(
                ui,
                "Memory",
                &format!("peak {}", format_bytes(rss.peak)),
                &folded,
                |item| item.rss_bytes as f32,
                theme::series(ui, 2),
            );
            crate::widgets::caption(
                ui,
                format!(
                    "Started at {} · {} observed samples",
                    format_bytes(rss.first),
                    rss.fast_samples
                ),
            );
        },
    );
}

fn trend(
    ui: &mut egui::Ui,
    title: &str,
    peak: &str,
    folded: &[sweeploom_history::Sample],
    pick: fn(&sweeploom_history::Sample) -> f32,
    color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{title} trend")).size(13.5).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            crate::widgets::caption(ui, peak);
        });
    });
    ui.add_space(theme::SM);
    let values: Vec<f32> = folded.iter().map(pick).collect();
    crate::widgets::sparkline_max(
        ui,
        &values,
        egui::vec2(ui.available_width(), 44.0),
        color,
        None,
    );
    ui.add_space(theme::XS);
}

fn avg_label(value: Option<f32>) -> String {
    value
        .map(|cpu| format!("{cpu:.1}%"))
        .unwrap_or_else(|| "Not enough history".into())
}

fn draw_network(ui: &mut egui::Ui, session: &LiveSession) {
    let ports = if !session.network.connections_available {
        "Unavailable on this OS".into()
    } else if session.network.listening_ports.is_empty() {
        "None observed".into()
    } else {
        session
            .network
            .listening_ports
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    super::session_detail::field(ui, "Listening ports", ports);
    let traffic = if session.network.byte_rate_available {
        format!(
            "Received {} · Sent {} (since observation began)",
            format_bytes(session.network.observed_rx_bytes),
            format_bytes(session.network.observed_tx_bytes)
        )
    } else {
        "Unavailable on this OS".into()
    };
    super::session_detail::field(ui, "TCP traffic", traffic);
}
