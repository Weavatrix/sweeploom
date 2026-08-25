//! Logical session table and raw process tree.

use super::session_actions;
use super::session_observe;
use super::session_plan;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::{page_title, table_scroll_height};
use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_core::LiveSession;

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Sessions",
        "Logical sessions sit on top of the OS process tree. Keep means leave it running. Idle is only after known quiet time, not uptime.",
    );
    session_plan::draw(app, ui);
    ui.checkbox(&mut app.group_raw, "Raw process tree");
    ui.add_space(8.0);
    if app.group_raw {
        ui_process_table(app, ui);
        return;
    }
    draw_session_table(app, ui);
}

fn draw_session_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.session_sort;
    let mut selected = app.selected_session;
    let mut planned = std::mem::take(&mut app.planned_keys);
    let order = session_order(&app.sessions, sort);
    let row_count = order.len();
    let height = table_scroll_height(ui);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(36.0))
        .column(Column::auto().at_least(180.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(160.0))
        .column(Column::remainder())
        .header(32.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Session"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Procs, "Procs"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "RSS"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Cpu, "CPU"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Status"));
            header.col(|ui| {
                ui.strong("Project");
            });
        })
        .body(|body| {
            body.rows(28.0, row_count, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(row.index());
                fill_session_row(
                    &app.sessions,
                    &mut planned,
                    selected,
                    &mut row,
                    index,
                    &mut selected,
                );
            });
        });
    app.planned_keys = planned;
    app.session_sort = sort;
    app.selected_session = selected;
    if let Some(id) = app.selected_session
        && let Some(session) = app.sessions.iter().find(|item| item.id == id).cloned()
    {
        ui.separator();
        session_details(app, ui, &session);
    }
}

fn session_order(sessions: &[LiveSession], sort: Sort) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sessions.len()).collect();
    order.sort_by(|&left, &right| compare_session(&sessions[left], &sessions[right], sort));
    if sort.desc {
        order.reverse();
    }
    order
}

fn compare_session(left: &LiveSession, right: &LiveSession, sort: Sort) -> std::cmp::Ordering {
    match sort.col {
        Col::Name => left.label().cmp(right.label()),
        Col::Procs => left.processes.len().cmp(&right.processes.len()),
        Col::Cpu => left
            .cpu_percent
            .partial_cmp(&right.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal),
        Col::Status => left
            .recommendation
            .recommendation
            .cmp(&right.recommendation.recommendation),
        Col::Size => left.rss_bytes.cmp(&right.rss_bytes),
    }
}

fn fill_session_row(
    sessions: &[LiveSession],
    planned: &mut std::collections::HashSet<sweeploom_core::ProcessKey>,
    selected_id: Option<sweeploom_core::SessionId>,
    row: &mut egui_extras::TableRow<'_, '_>,
    index: usize,
    selected: &mut Option<sweeploom_core::SessionId>,
) {
    let Some(session) = sessions.get(index) else {
        return;
    };
    let is_selected = selected_id == Some(session.id);
    let id = session.id;
    let label = session.label().to_owned();
    let procs = session.processes.len().to_string();
    let rss = format_bytes(session.rss_bytes);
    let cpu = format!("{:.1}%", session.cpu_percent);
    let rec = session.recommendation.recommendation.label().to_owned();
    let project = session
        .project
        .as_ref()
        .map(|item| item.0.display().to_string())
        .unwrap_or_else(|| "Unknown".to_owned());
    row.set_selected(is_selected);
    row.col(|ui| {
        session_plan::checkbox(ui, session, planned);
    });
    row.col(|ui| {
        if crate::widgets::pointer(
            ui.add(egui::Button::new(RichText::new(label).size(16.0)).frame(false)),
        )
        .clicked()
        {
            *selected = Some(id);
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
    let Some(snapshot) = &app.snapshot else {
        return;
    };
    let mut sort = app.process_sort;
    let mut order: Vec<usize> = (0..snapshot.processes.len()).collect();
    order.sort_by(|&left, &right| {
        let a = &snapshot.processes[left];
        let b = &snapshot.processes[right];
        match sort.col {
            Col::Name => a.name.cmp(&b.name),
            Col::Cpu => a
                .cpu_percent
                .partial_cmp(&b.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal),
            _ => a.rss_bytes.cmp(&b.rss_bytes),
        }
    });
    if sort.desc {
        order.reverse();
    }
    let rows = order.len();
    let height = table_scroll_height(ui);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(180.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::remainder())
        .header(32.0, |mut header| {
            header.col(|ui| {
                ui.strong("PID");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Process"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "RSS"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Cpu, "CPU"));
            header.col(|ui| {
                ui.strong("Command");
            });
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
    app.process_sort = sort;
}

fn session_details(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    ui.label(RichText::new(session.label()).size(20.0).strong());
    ui.label(format!(
        "RAM {} · CPU {:.1}% · processes {} · {}",
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity.label()
    ));
    session_observe::draw(app, ui, session);
    if session.safety.terminate_disabled {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "Terminate disabled (system-critical).",
        );
    }
    session_actions::draw(app, ui, session);
}
