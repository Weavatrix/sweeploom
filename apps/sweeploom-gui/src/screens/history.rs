//! Observed process history. Never invented from before SweepLoom started.

use sweeploom_core::SessionId;
use sweeploom_history::{Sample, summarize_cpu};

use crate::app::SweepLoomApp;
use crate::nav::Nav;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::{page_title, table_scroll_height};
use eframe::egui;
use egui_extras::Column;

use super::history_rows::{self, HistRow, Line};

#[path = "history_cells.rs"]
mod cells;

/// History sub-view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoryView {
    /// Observed process CPU/RAM.
    #[default]
    Activity,
    /// Saved Explorer scans and folder measurements.
    DiskScans,
}

pub fn ui_history(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let hint = hint_for(app.history_view);
    if crate::edition::STORE {
        page_title(ui, "History", hint_for(HistoryView::DiskScans));
        crate::edition::full_only_note(
            ui,
            "Process activity history needs process access the App Sandbox does not give.",
        );
        ui.add_space(crate::theme::SM);
        crate::disk_history::ui(app, ui);
        return;
    }
    page_title(ui, "History", hint);
    let scans = app.scan_history.scans().len();
    let mut view = app.history_view;
    if crate::widgets::tabs(
        ui,
        &mut view,
        &[
            (HistoryView::Activity, "Process activity".to_owned()),
            (HistoryView::DiskScans, format!("Disk scans · {scans}")),
        ],
    ) {
        app.history_view = view;
    }
    match view {
        HistoryView::Activity => activity(app, ui),
        HistoryView::DiskScans => crate::disk_history::ui(app, ui),
    }
}

fn hint_for(view: HistoryView) -> &'static str {
    match view {
        HistoryView::Activity => {
            "Process CPU and memory observed since SweepLoom started. Groups start collapsed; click a member to open its session."
        }
        HistoryView::DiskScans => {
            "Saved Explorer scans and folder measurements survive restarts. Nothing is invented for time before history was enabled."
        }
    }
}

fn activity(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let Some(snapshot) = &app.snapshot else {
        ui.label("No live snapshot yet.");
        return;
    };
    let rows = collect_rows(app, snapshot.processes.as_slice());
    let mut sort = app.history_sort;
    let expanded = app.expanded_history.clone();
    let lines = history_rows::table_lines(
        &rows,
        &app.sessions,
        snapshot.processes.as_slice(),
        sort,
        &expanded,
    );
    let mut go = None;
    let mut raw = false;
    let mut toggle = None;
    draw_table(ui, &lines, &mut sort, &mut go, &mut raw, &mut toggle);
    app.history_sort = sort;
    if let Some(key) = toggle
        && !app.expanded_history.remove(&key)
    {
        app.expanded_history.insert(key);
    }
    if go.is_some() || raw {
        app.selected_session = go;
        app.group_raw = raw;
        app.nav = Nav::Sessions;
    }
}

fn collect_rows(app: &SweepLoomApp, processes: &[sweeploom_core::ProcessSnapshot]) -> Vec<HistRow> {
    let mut rows = Vec::new();
    for process in processes {
        let Some(hist) = app.history.get(process.key) else {
            continue;
        };
        let fast = hist.fast.chrono();
        let Some(last) = fast.last() else {
            continue;
        };
        let cpu = summarize_cpu(&fast, &hist.slow.chrono(), last.at_unix_ms);
        rows.push(HistRow {
            name: process.name.clone(),
            pid: process.pid,
            rss: process.rss_bytes,
            peak: fast.iter().map(|item| item.rss_bytes).max().unwrap_or(0),
            cpu: cpu.now,
            avg_5m: avg_short(cpu.avg_5m),
            samples: fast.len(),
            spark: spark_values(&fast),
            session: process.session,
        });
    }
    rows
}

fn spark_values(fast: &[Sample]) -> Vec<f32> {
    let take = 40.min(fast.len());
    let start = fast.len().saturating_sub(take);
    fast[start..].iter().map(|item| item.cpu_percent).collect()
}

fn draw_table(
    ui: &mut egui::Ui,
    lines: &[Line],
    sort: &mut Sort,
    go: &mut Option<SessionId>,
    raw: &mut bool,
    toggle: &mut Option<String>,
) {
    let height = table_scroll_height(ui);
    let count = lines.len();
    crate::widgets::table(ui, "history-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(200.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(76.0).clip(true))
        .column(Column::exact(140.0).clip(true))
        .column(Column::exact(76.0).clip(true))
        .header(30.0, |mut header| {
            header.col(|ui| header_cell(ui, sort, Col::Name, "Process"));
            header.col(|ui| header_cell(ui, sort, Col::Size, "RSS"));
            header.col(|ui| header_cell(ui, sort, Col::Procs, "Peak"));
            header.col(|ui| header_cell(ui, sort, Col::Cpu, "CPU"));
            header.col(|ui| {
                ui.strong("5m avg");
            });
            header.col(|ui| {
                ui.strong("CPU trend");
            });
            header.col(|ui| header_cell(ui, sort, Col::Status, "Samples"));
        })
        .body(|body| {
            body.rows(crate::widgets::TABLE_ROW, count, |mut row| {
                match lines.get(row.index()) {
                    Some(line @ Line::Group { key, .. }) => {
                        cells::fill_group(&mut row, line, toggle);
                        if row.response().clicked() {
                            *toggle = Some(key.clone());
                        }
                    }
                    Some(Line::Item(item)) => cells::fill_item(&mut row, item, go, raw),
                    None => {}
                }
            });
        });
}

fn avg_short(value: Option<f32>) -> String {
    match value {
        Some(cpu) => format!("{cpu:.1}%"),
        None => "—".to_owned(),
    }
}
