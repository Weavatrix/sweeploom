//! Observed process history. Never invented from before SweepLoom started.

use sweeploom_core::SessionId;
use sweeploom_history::{Sample, summarize_cpu};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{page_title, sparkline, table_scroll_height};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

struct HistRow {
    name: String,
    pid: u32,
    rss: u64,
    peak: u64,
    cpu: f32,
    avg_5m: String,
    samples: usize,
    spark: Vec<f32>,
    session: Option<SessionId>,
}

pub fn ui_history(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "History",
        "Rings start when SweepLoom first sees a process. Averages stay unavailable until watched long enough. Click a row to open its session.",
    );
    ui.label(format!("Tracked processes: {}", app.history.len()));
    ui.add_space(8.0);
    let Some(snapshot) = &app.snapshot else {
        ui.label("No live snapshot yet.");
        return;
    };
    let mut rows = collect_rows(app, snapshot.processes.as_slice());
    let mut sort = app.history_sort;
    sort_rows(&mut rows, sort);
    let mut go = None;
    draw_table(ui, &rows, &mut sort, &mut go);
    app.history_sort = sort;
    if let Some(id) = go {
        app.selected_session = Some(id);
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

fn sort_rows(rows: &mut [HistRow], sort: Sort) {
    rows.sort_by(|left, right| match sort.col {
        Col::Name => left.name.cmp(&right.name),
        Col::Cpu => left
            .cpu
            .partial_cmp(&right.cpu)
            .unwrap_or(std::cmp::Ordering::Equal),
        Col::Status | Col::Procs => left.samples.cmp(&right.samples),
        Col::Size => left.rss.cmp(&right.rss),
    });
    if sort.desc {
        rows.reverse();
    }
}

fn draw_table(ui: &mut egui::Ui, rows: &[HistRow], sort: &mut Sort, go: &mut Option<SessionId>) {
    let height = table_scroll_height(ui);
    let count = rows.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(140.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(90.0))
        .column(Column::auto().at_least(88.0))
        .column(Column::auto().at_least(70.0))
        .header(32.0, |mut header| {
            header.col(|ui| header_cell(ui, sort, Col::Name, "Process"));
            header.col(|ui| header_cell(ui, sort, Col::Size, "RSS"));
            header.col(|ui| header_cell(ui, sort, Col::Procs, "Peak"));
            header.col(|ui| header_cell(ui, sort, Col::Cpu, "CPU"));
            header.col(|ui| {
                ui.strong("5m");
            });
            header.col(|ui| {
                ui.strong("Spark");
            });
            header.col(|ui| header_cell(ui, sort, Col::Status, "Samples"));
        })
        .body(|body| {
            body.rows(28.0, count, |mut row| {
                let Some(item) = rows.get(row.index()) else {
                    return;
                };
                row.col(|ui| {
                    ui.label(format!("{}  pid {}", item.name, item.pid));
                });
                row.col(|ui| {
                    ui.label(format_bytes(item.rss));
                });
                row.col(|ui| {
                    ui.label(format_bytes(item.peak));
                });
                row.col(|ui| {
                    ui.label(format!("{:.1}%", item.cpu));
                });
                row.col(|ui| {
                    ui.label(&item.avg_5m);
                });
                row.col(|ui| {
                    sparkline(ui, &item.spark, egui::vec2(80.0, 18.0), theme::accent());
                });
                row.col(|ui| {
                    ui.label(item.samples.to_string());
                });
                if row.response().clicked() {
                    *go = item.session;
                }
            });
        });
}

fn avg_short(value: Option<f32>) -> String {
    match value {
        Some(cpu) => format!("{cpu:.1}%"),
        None => "unavailable".to_owned(),
    }
}
