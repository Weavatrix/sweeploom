//! Logical session table and raw process tree.

use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_core::LiveSession;
use sweeploom_process::ProcessSnapshotSet;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Sessions");
        ui.separator();
        ui.checkbox(&mut app.group_raw, "Raw process tree");
    });
    ui.label(
        RichText::new(
            "Logical sessions sit on top of the OS process tree. Terminate is never automatic.",
        )
        .weak(),
    );
    ui.add_space(8.0);
    if app.group_raw {
        ui_process_table(app, ui);
        return;
    }
    draw_session_table(app, ui);
}

fn draw_session_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let row_count = app.sessions.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(140.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(140.0))
        .column(Column::remainder())
        .header(22.0, |mut header| {
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
        .body(|mut body| {
            body.rows(20.0, row_count, |mut row| {
                fill_session_row(app, &mut row);
            });
        });
    if let Some(index) = app.selected_session
        && let Some(session) = app.sessions.get(index)
    {
        ui.separator();
        session_details(ui, session, app.snapshot.as_ref());
    }
}

fn fill_session_row(app: &mut SweepLoomApp, row: &mut egui_extras::TableRow<'_, '_>) {
    let index = row.index();
    let Some(session) = app.sessions.get(index) else {
        return;
    };
    let selected = app.selected_session == Some(index);
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
        if ui.selectable_label(selected, label).clicked() {
            app.selected_session = Some(index);
        }
    });
    row.col(|ui| {
        ui.label(procs);
    });
    row.col(|ui| {
        ui.label(rss);
    });
    row.col(|ui| {
        ui.label(cpu);
    });
    row.col(|ui| {
        ui.label(rec);
    });
    row.col(|ui| {
        ui.label(project);
    });
}

pub fn ui_process_table(app: &SweepLoomApp, ui: &mut egui::Ui) {
    let Some(snapshot) = &app.snapshot else {
        return;
    };
    let rows = snapshot.processes.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(160.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::remainder())
        .header(22.0, |mut header| {
            for title in ["PID", "Process", "RSS", "CPU", "Command"] {
                header.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|mut body| {
            body.rows(18.0, rows, |mut row| {
                let index = row.index();
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

fn session_details(
    ui: &mut egui::Ui,
    session: &LiveSession,
    snapshot: Option<&ProcessSnapshotSet>,
) {
    ui.label(RichText::new(session.label()).strong());
    ui.label(format!(
        "RAM {} · CPU {:.1}% · processes {} · {:?}",
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity
    ));
    if session.safety.terminate_disabled {
        ui.colored_label(
            Color32::from_rgb(240, 160, 80),
            "Terminate disabled (system-critical).",
        );
    }
    let Some(snapshot) = snapshot else {
        return;
    };
    for key in &session.processes {
        if let Some(process) = snapshot.processes.iter().find(|item| item.key == *key) {
            ui.monospace(format!(
                "pid {}  {}  {}",
                process.pid,
                format_bytes(process.rss_bytes),
                process.name
            ));
        }
    }
}
