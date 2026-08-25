//! Observed RAM/CPU/disk for a selected session. Never back-filled.

use eframe::egui;
use sweeploom_core::LiveSession;
use sweeploom_history::summarize_cpu;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn draw(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    draw_history(app, ui, session);
    ui.label(format!(
        "Disk this interval  read {}  write {}",
        format_bytes(session.disk.read_bytes),
        format_bytes(session.disk.write_bytes)
    ));
    draw_network(ui, session);
}

fn draw_history(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let Some(key) = session.processes.first() else {
        return;
    };
    let Some(hist) = app.history.get(*key) else {
        ui.label("CPU/RAM history starts when SweepLoom first sees this process.");
        return;
    };
    let fast = hist.fast.chrono();
    let Some(last) = fast.last() else {
        return;
    };
    if let Some(first) = fast.first() {
        ui.label(format!(
            "Observed {} → {} RSS over {} samples",
            format_bytes(first.rss_bytes),
            format_bytes(last.rss_bytes),
            fast.len()
        ));
    }
    let cpu = summarize_cpu(&fast, &hist.slow.chrono(), last.at_unix_ms);
    ui.label(format!(
        "CPU now {:.1}%  peak {:.1}%  {}",
        cpu.now,
        cpu.peak,
        avg_label("5m", cpu.avg_5m)
    ));
    ui.label(avg_label("1h", cpu.avg_1h));
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
