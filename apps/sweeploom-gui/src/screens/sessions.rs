//! Logical session table. Details sit in reserved space under the table.

use super::session_detail;
use super::session_fills;
use super::session_label;
use super::session_plan;
use super::session_raw;
use super::session_rows::{self, SessionRow};
use crate::app::SweepLoomApp;
use crate::sort::{Col, header_cell};
use crate::theme;
use crate::widgets;
use eframe::egui::{self, RichText};
use egui_extras::Column;
use sweeploom_core::ProcessSnapshot;

const HELP: &str = "Logical sessions sit on the OS process tree. Click a row for members. A forgotten shell under Claude can be stopped without ending the agent.";

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Spotlight,
    All,
    Raw,
}

pub fn ui_sessions(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    if let Some(snapshot) = &app.snapshot {
        app.node_versions.poll(&snapshot.processes, ui.ctx());
    }
    ui.horizontal(|ui| {
        ui.label(RichText::new("Sessions").size(22.0).strong());
        widgets::help_mark(ui, HELP);
    });
    ui.add_space(2.0);
    widgets::caption(ui, node_caption(app));
    ui.add_space(theme::LG);
    widgets::toolbar(ui, |ui| {
        let hidden = hidden_count(app);
        let mut view = match (app.group_raw, app.show_all_apps) {
            (true, _) => View::Raw,
            (false, true) => View::All,
            (false, false) => View::Spotlight,
        };
        let all = format!("All apps +{hidden}");
        if widgets::segmented(
            ui,
            &mut view,
            &[
                (View::Spotlight, "Dev sessions"),
                (View::All, all.as_str()),
                (View::Raw, "Process tree"),
            ],
        ) {
            app.group_raw = view == View::Raw;
            app.show_all_apps = view == View::All;
        }
        if !app.group_raw {
            widgets::search_field(
                ui,
                &mut app.session_search,
                "Filter: tool, Node version, PID or folder",
                300.0,
            );
            if !app.session_search.is_empty() && widgets::button(ui, "Clear", true).clicked() {
                app.session_search.clear();
            }
        }
        ui.end_row();
        ui.add_space(theme::XS);
        session_plan::controls(app, ui);
    });
    session_plan::extras(app, ui);
    if app.group_raw {
        session_raw::draw(app, ui);
        return;
    }
    draw_session_table(app, ui);
}

fn node_caption(app: &SweepLoomApp) -> String {
    let Some(snapshot) = &app.snapshot else {
        return "Waiting for the first process sample…".into();
    };
    let is_node = |key: &sweeploom_core::ProcessKey| {
        snapshot
            .processes
            .iter()
            .any(|p| p.key == *key && sweeploom_session::is_node(p))
    };
    let nodes = snapshot
        .processes
        .iter()
        .filter(|p| sweeploom_session::is_node(p))
        .count();
    let launches = app
        .sessions
        .iter()
        .filter(|s| s.processes.iter().any(is_node))
        .count();
    format!(
        "{} sessions · {nodes} Node processes in {launches} trees · counts include children. Select a row for script, runtime, parent and idle evidence.",
        app.sessions.len()
    )
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
    if selected.is_some_and(|id| {
        !rows
            .iter()
            .any(|row| app.sessions.get(row.index).is_some_and(|s| s.id == id))
    }) {
        selected = None;
    }
    let expanded = app.expanded_sessions.clone();
    let lines = session_rows::table_lines(&rows, sort, &expanded);
    let mut planned = std::mem::take(&mut app.planned_keys);
    let mut toggle = None;
    let reserve = if selected.is_some() {
        (ui.available_height() * 0.55).clamp(240.0, 520.0)
    } else {
        40.0
    };
    let header_height = 30.0 + ui.spacing().item_spacing.y;
    let height = (ui.available_height() - reserve - header_height).max(140.0);
    let row_count = lines.len();
    ui.allocate_ui(
        egui::vec2(ui.available_width(), height + header_height),
        |ui| {
            // TableBuilder paints resize guides using the containing UI's bottom.
            // Keep both their paint and hit area inside the reserved table region.
            ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
            crate::widgets::table(ui, "sessions-workloads-v3")
                .striped(true)
                .resizable(true)
                .sense(egui::Sense::click())
                .min_scrolled_height(height)
                .max_scroll_height(height)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(34.0).clip(true).resizable(false))
                .column(Column::remainder().at_least(220.0).clip(true))
                .column(Column::exact(104.0).clip(true))
                .column(Column::exact(84.0).clip(true))
                .column(Column::exact(72.0).clip(true))
                .column(Column::exact(168.0).clip(true))
                .column(Column::initial(240.0).at_least(120.0).clip(true))
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Session"));
                    header.col(|ui| header_cell(ui, &mut sort, Col::Procs, "Processes"));
                    header.col(|ui| header_cell(ui, &mut sort, Col::Size, "RSS"));
                    header.col(|ui| header_cell(ui, &mut sort, Col::Cpu, "CPU"));
                    header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Status"));
                    header.col(|ui| {
                        ui.strong("Project / cwd");
                    });
                })
                .body(|body| {
                    body.rows(widgets::TABLE_ROW_TALL, row_count, |mut row| {
                        session_fills::fill_line(
                            &lines,
                            &app.sessions,
                            &rows,
                            &mut planned,
                            selected,
                            &mut row,
                            &mut selected,
                            &mut toggle,
                        );
                    });
                });
        },
    );
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
    let query = app.session_search.trim().to_lowercase();
    app.sessions
        .iter()
        .enumerate()
        .filter(|(_, session)| app.show_all_apps || session_label::is_spotlight(session, processes))
        .map(|(index, session)| {
            session_label::row(
                index,
                session,
                processes,
                &app.node_versions,
                app.snapshot
                    .as_ref()
                    .map(|s| s.captured_at)
                    .unwrap_or_else(std::time::SystemTime::now),
            )
        })
        .filter(|row| {
            query.is_empty()
                || format!(
                    "{} {} {} {}",
                    row.title, row.subtitle, row.project, row.source_tip
                )
                .to_lowercase()
                .contains(&query)
        })
        .collect()
}
