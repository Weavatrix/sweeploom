//! Logical session table. Details sit in reserved space under the table.

use super::session_detail;
use super::session_fills;
use super::session_label;
use super::session_plan;
use super::session_raw;
use super::session_rows::{self, SessionRow};
use crate::app::SweepLoomApp;
use crate::sort::{Col, header_cell};
use crate::widgets;
use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_core::ProcessSnapshot;

const HELP: &str = "Logical sessions sit on the OS process tree. Click a row for members. A forgotten shell under Claude can be stopped without ending the agent.";

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.heading(RichText::new("Sessions").size(22.0).strong());
        widgets::help_mark(ui, HELP);
        session_plan::controls(app, ui);
        ui.checkbox(&mut app.group_raw, "Raw")
            .on_hover_text("Show the raw OS process tree");
        if !app.group_raw {
            let hidden = hidden_count(app);
            ui.checkbox(&mut app.show_all_apps, format!("{hidden} leftover"))
                .on_hover_text("Show leftover apps hidden from the spotlight list");
        }
    });
    session_plan::extras(app, ui);
    ui.add_space(6.0);
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
    let rows = collect_rows(app);
    let expanded = app.expanded_sessions.clone();
    let lines = session_rows::table_lines(&rows, sort, &expanded);
    let mut planned = std::mem::take(&mut app.planned_keys);
    let mut toggle = None;
    let reserve = if selected.is_some() {
        (ui.available_height() * 0.45).clamp(200.0, 420.0)
    } else {
        40.0
    };
    let height = (ui.available_height() - reserve).max(140.0);
    let row_count = lines.len();
    TableBuilder::new(ui)
        .id_salt("sessions-grouped")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(36.0).clip(true).resizable(false))
        .column(Column::initial(220.0).at_least(140.0).clip(true))
        .column(Column::exact(52.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(64.0).clip(true))
        .column(Column::exact(136.0).clip(true))
        .column(Column::remainder().at_least(80.0).clip(true))
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
            body.rows(crate::widgets::TABLE_ROW, row_count, |mut row| {
                session_fills::fill_line(
                    &lines,
                    &app.sessions,
                    &mut planned,
                    selected,
                    &mut row,
                    &mut selected,
                    &mut toggle,
                );
            });
        });
    app.planned_keys = planned;
    app.session_sort = sort;
    if let Some(key) = toggle
        && !app.expanded_sessions.remove(&key)
    {
        app.expanded_sessions.insert(key);
    }
    if app.selected_session != selected {
        app.helper_keys.clear();
        app.confirm_helpers = false;
    }
    app.selected_session = selected;
    session_detail::draw_details(app, ui);
}

fn collect_rows(app: &SweepLoomApp) -> Vec<SessionRow> {
    let processes = processes_of(app);
    app.sessions
        .iter()
        .enumerate()
        .filter(|(_, session)| app.show_all_apps || session_label::is_spotlight(session, processes))
        .map(|(index, session)| SessionRow {
            index,
            title: session_label::title(session, processes),
            kind: session.kind,
            rss: session.rss_bytes,
            cpu: session.cpu_percent,
            procs: session.processes.len(),
            status: session.activity.label().to_owned(),
            project: session
                .project
                .as_ref()
                .map(|item| session_rows::project_name(&item.0))
                .unwrap_or_else(|| "Unknown".to_owned()),
        })
        .collect()
}
