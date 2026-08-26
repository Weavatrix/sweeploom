//! Logical session table. Details sit in reserved space under the table.

use super::session_actions;
use super::session_label;
use super::session_members;
use super::session_observe;
use super::session_plan;
use super::session_raw;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::page_title;
use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_core::{LiveSession, ProcessSnapshot};

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Sessions",
        "Logical sessions sit on top of the OS process tree. Select a row — member processes appear under the table. A forgotten shell under Claude can be stopped without ending the agent.",
    );
    session_plan::draw(app, ui);
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut app.group_raw, "Raw process tree");
        if !app.group_raw {
            let hidden = hidden_count(app);
            ui.checkbox(
                &mut app.show_all_apps,
                format!("Show leftover apps ({hidden} hidden)"),
            );
        }
    });
    ui.add_space(8.0);
    if app.group_raw {
        session_raw::draw(app, ui);
        return;
    }
    draw_session_table(app, ui);
}

fn hidden_count(app: &SweepLoomApp) -> usize {
    let processes = processes_of(app);
    app.sessions
        .iter()
        .filter(|session| !session_label::is_spotlight(session, processes))
        .count()
}

fn processes_of(app: &SweepLoomApp) -> &[ProcessSnapshot] {
    app.snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[])
}

fn draw_session_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.session_sort;
    let mut selected = app.selected_session;
    let show_all = app.show_all_apps;
    let (order, titles) = {
        let processes = processes_of(app);
        let order = session_order(&app.sessions, processes, sort, show_all);
        let titles: Vec<String> = order
            .iter()
            .filter_map(|&index| app.sessions.get(index))
            .map(|session| session_label::title(session, processes))
            .collect();
        (order, titles)
    };
    let mut planned = std::mem::take(&mut app.planned_keys);
    let reserve = if selected.is_some() {
        (ui.available_height() * 0.45).clamp(200.0, 420.0)
    } else {
        40.0
    };
    let height = (ui.available_height() - reserve).max(140.0);
    let row_count = order.len();
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
                let title = titles.get(row.index()).cloned().unwrap_or_default();
                fill_session_row(
                    &app.sessions,
                    &mut planned,
                    selected,
                    &mut row,
                    index,
                    &mut selected,
                    &title,
                );
            });
        });
    app.planned_keys = planned;
    app.session_sort = sort;
    if app.selected_session != selected {
        app.helper_keys.clear();
        app.confirm_helpers = false;
    }
    app.selected_session = selected;
    draw_details(app, ui);
}

fn session_order(
    sessions: &[LiveSession],
    processes: &[ProcessSnapshot],
    sort: Sort,
    show_all: bool,
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sessions.len())
        .filter(|&index| show_all || session_label::is_spotlight(&sessions[index], processes))
        .collect();
    order.sort_by(|&left, &right| {
        compare_session(&sessions[left], &sessions[right], processes, sort)
    });
    if sort.desc {
        order.reverse();
    }
    order
}

fn compare_session(
    left: &LiveSession,
    right: &LiveSession,
    processes: &[ProcessSnapshot],
    sort: Sort,
) -> std::cmp::Ordering {
    match sort.col {
        Col::Name => {
            session_label::title(left, processes).cmp(&session_label::title(right, processes))
        }
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
    title: &str,
) {
    let Some(session) = sessions.get(index) else {
        return;
    };
    let is_selected = selected_id == Some(session.id);
    let id = session.id;
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
            ui.add(egui::Button::new(RichText::new(title).size(16.0)).frame(false)),
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
    if row.response().clicked() {
        *selected = Some(id);
    }
}

fn draw_details(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let Some(id) = app.selected_session else {
        ui.label("Select a session to see member processes that can be stopped on their own.");
        return;
    };
    let Some(session) = app.sessions.iter().find(|item| item.id == id).cloned() else {
        ui.label("Select a session to see member processes that can be stopped on their own.");
        return;
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            session_details(app, ui, &session);
        });
}

fn session_details(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let title = session_label::title(session, processes_of(app));
    ui.separator();
    ui.label(RichText::new(title).size(20.0).strong());
    ui.label(format!(
        "{} · RAM {} · CPU {:.1}% · processes {} · {}",
        session.kind.label(),
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity.label()
    ));
    session_observe::draw(app, ui, session);
    session_members::draw(app, ui, session);
    if session.safety.terminate_disabled {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "Terminate disabled (system-critical).",
        );
    }
    session_actions::draw(app, ui, session);
}
