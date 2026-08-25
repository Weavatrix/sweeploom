//! Logical session table and raw process tree.

use super::session_actions;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, header_button};
use crate::widgets::page_title;
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_core::LiveSession;

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Sessions",
        "Logical sessions sit on top of the OS process tree. Terminate is never automatic.",
    );
    ui.checkbox(&mut app.group_raw, "Raw process tree");
    ui.add_space(8.0);
    if app.group_raw {
        ui_process_table(app, ui);
        return;
    }
    draw_session_table(app, ui);
}

fn draw_session_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Sort");
        header_button(ui, &mut app.session_sort, Col::Name, "Name");
        header_button(ui, &mut app.session_sort, Col::Size, "RSS");
        header_button(ui, &mut app.session_sort, Col::Cpu, "CPU");
        header_button(ui, &mut app.session_sort, Col::Procs, "Procs");
        header_button(ui, &mut app.session_sort, Col::Status, "Status");
    });
    let order = session_order(app);
    let row_count = order.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(180.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(160.0))
        .column(Column::remainder())
        .header(30.0, |mut header| {
            for title in [
                "Session",
                "Procs",
                "RSS",
                "CPU",
                "Recommendation",
                "Project",
            ] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|body| {
            body.rows(28.0, row_count, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(row.index());
                fill_session_row(app, &mut row, index);
            });
        });
    if let Some(id) = app.selected_session
        && let Some(session) = app.sessions.iter().find(|item| item.id == id).cloned()
    {
        ui.separator();
        session_details(app, ui, &session);
    }
}

fn session_order(app: &SweepLoomApp) -> Vec<usize> {
    let mut order: Vec<usize> = (0..app.sessions.len()).collect();
    order.sort_by(|&left, &right| compare_session(&app.sessions[left], &app.sessions[right], app));
    if app.session_sort.desc {
        order.reverse();
    }
    order
}

fn compare_session(
    left: &LiveSession,
    right: &LiveSession,
    app: &SweepLoomApp,
) -> std::cmp::Ordering {
    match app.session_sort.col {
        Col::Name => left.label().cmp(right.label()),
        Col::Procs => left.processes.len().cmp(&right.processes.len()),
        Col::Cpu => left
            .cpu_percent
            .partial_cmp(&right.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal),
        Col::Status => format!("{:?}", left.recommendation.recommendation)
            .cmp(&format!("{:?}", right.recommendation.recommendation)),
        Col::Size => left.rss_bytes.cmp(&right.rss_bytes),
    }
}

fn fill_session_row(app: &mut SweepLoomApp, row: &mut egui_extras::TableRow<'_, '_>, index: usize) {
    let Some(session) = app.sessions.get(index) else {
        return;
    };
    let selected = app.selected_session == Some(session.id);
    let id = session.id;
    let label = session.label().to_owned();
    let procs = session.processes.len().to_string();
    let rss = format_bytes(session.rss_bytes);
    let cpu = format!("{:.1}%", session.cpu_percent);
    let rec = format!("{:?}", session.recommendation.recommendation);
    let project = session
        .project
        .as_ref()
        .map(|item| item.0.display().to_string())
        .unwrap_or_else(|| "Unknown".to_owned());
    row.col(|ui| {
        if ui
            .selectable_label(selected, RichText::new(label).size(16.0))
            .clicked()
        {
            app.selected_session = Some(id);
        }
    });
    row.col(|ui| {
        ui.label(&procs);
    });
    row.col(|ui| {
        ui.label(&rss);
    });
    row.col(|ui| {
        ui.label(&cpu);
    });
    row.col(|ui| {
        ui.label(&rec);
    });
    row.col(|ui| {
        ui.label(&project);
    });
}

fn ui_process_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Sort");
        header_button(ui, &mut app.process_sort, Col::Name, "Name");
        header_button(ui, &mut app.process_sort, Col::Size, "RSS");
        header_button(ui, &mut app.process_sort, Col::Cpu, "CPU");
    });
    let Some(snapshot) = &app.snapshot else {
        return;
    };
    let mut order: Vec<usize> = (0..snapshot.processes.len()).collect();
    order.sort_by(|&left, &right| {
        let a = &snapshot.processes[left];
        let b = &snapshot.processes[right];
        match app.process_sort.col {
            Col::Name => a.name.cmp(&b.name),
            Col::Cpu => a
                .cpu_percent
                .partial_cmp(&b.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal),
            _ => a.rss_bytes.cmp(&b.rss_bytes),
        }
    });
    if app.process_sort.desc {
        order.reverse();
    }
    let rows = order.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(180.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::remainder())
        .header(30.0, |mut header| {
            for title in ["PID", "Process", "RSS", "CPU", "Command"] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|body| {
            body.rows(26.0, rows, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(0);
                if let Some(process) = snapshot.processes.get(index) {
                    row.col(|ui| {
                        ui.label(process.pid.to_string());
                    });
                    row.col(|ui| {
                        ui.label(&process.name);
                    });
                    row.col(|ui| {
                        ui.label(format_bytes(process.rss_bytes));
                    });
                    row.col(|ui| {
                        ui.label(format!("{:.1}%", process.cpu_percent));
                    });
                    row.col(|ui| {
                        ui.monospace(process.command.join(" "));
                    });
                }
            });
        });
}

fn session_details(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    ui.label(RichText::new(session.label()).size(20.0).strong());
    ui.label(format!(
        "RAM {} · CPU {:.1}% · processes {} · {:?}",
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity
    ));
    if let Some(key) = session.processes.first()
        && let Some(hist) = app.history.get(*key)
    {
        let samples = hist.fast.chrono();
        if let (Some(first), Some(last)) = (samples.first(), samples.last()) {
            ui.label(format!(
                "Observed {} → {} RSS over {} samples",
                format_bytes(first.rss_bytes),
                format_bytes(last.rss_bytes),
                samples.len()
            ));
        }
    }
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
    if session.safety.terminate_disabled {
        ui.colored_label(
            Color32::from_rgb(240, 160, 80),
            "Terminate disabled (system-critical).",
        );
    }
    session_actions::draw(app, ui, session);
}
