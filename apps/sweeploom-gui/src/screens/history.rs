//! Observed process history. Never invented from before SweepLoom started.

use sweeploom_core::SessionId;
use sweeploom_history::{Sample, summarize_cpu};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{disclose, page_title, sparkline_max, table_scroll_height};
use eframe::egui::{self, RichText};
use egui_extras::Column;

use super::history_rows::{self, HistRow, Line};

pub fn ui_history(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "History",
        "Groups start collapsed. Click a group to expand it. Click a member to open its session.",
    );
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
                        fill_group(&mut row, line, toggle);
                        if row.response().clicked() {
                            *toggle = Some(key.clone());
                        }
                    }
                    Some(Line::Item(item)) => fill_item(&mut row, item, go, raw),
                    None => {}
                }
            });
        });
}

fn fill_group(row: &mut egui_extras::TableRow<'_, '_>, line: &Line, toggle: &mut Option<String>) {
    let Line::Group {
        key,
        title,
        kind,
        rss,
        peak,
        cpu,
        avg_5m,
        spark,
        samples,
        count,
        expanded,
    } = line
    else {
        return;
    };
    row.col(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if disclose(ui, *expanded, theme::muted(ui)) {
            *toggle = Some(key.clone());
        }
        crate::brand::show_group(ui, title, *kind, 16.0);
        ui.add(
            egui::Label::new(RichText::new(title.as_str()).strong())
                .truncate()
                .selectable(false),
        );
        ui.label(
            RichText::new(if *count == 1 {
                "1 process".to_owned()
            } else {
                format!("{count} processes")
            })
                .size(12.0)
                .color(theme::muted(ui)),
        );
    });
    row.col(|ui| {
        ui.label(format_bytes(*rss));
    });
    row.col(|ui| {
        ui.label(format_bytes(*peak));
    });
    row.col(|ui| {
        cpu_cell(ui, *cpu);
    });
    row.col(|ui| {
        avg_cell(ui, avg_5m);
    });
    row.col(|ui| {
        trend_cell(ui, spark);
    });
    row.col(|ui| {
        ui.label(samples.to_string());
    });
}

fn fill_item(
    row: &mut egui_extras::TableRow<'_, '_>,
    item: &HistRow,
    go: &mut Option<SessionId>,
    raw: &mut bool,
) {
    row.col(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(44.0);
        ui.add(egui::Label::new(&item.name).truncate().selectable(false));
        ui.label(
            RichText::new(format!("pid {}", item.pid))
                .size(12.0)
                .color(theme::muted(ui)),
        );
    });
    row.col(|ui| {
        ui.label(format_bytes(item.rss));
    });
    row.col(|ui| {
        ui.label(format_bytes(item.peak));
    });
    row.col(|ui| {
        cpu_cell(ui, item.cpu);
    });
    row.col(|ui| {
        avg_cell(ui, &item.avg_5m);
    });
    row.col(|ui| {
        trend_cell(ui, &item.spark);
    });
    row.col(|ui| {
        ui.label(item.samples.to_string());
    });
    if row.response().clicked() {
        *go = item.session;
        *raw = item.session.is_none();
    }
}

fn avg_short(value: Option<f32>) -> String {
    match value {
        Some(cpu) => format!("{cpu:.1}%"),
        None => "—".to_owned(),
    }
}

fn cpu_cell(ui: &mut egui::Ui, cpu: f32) {
    let tone = if cpu >= 80.0 {
        theme::Tone::Warn
    } else {
        theme::Tone::Neutral
    };
    ui.label(RichText::new(format!("{cpu:.1}%")).color(theme::tone(ui, tone)));
}

fn avg_cell(ui: &mut egui::Ui, avg: &str) {
    if avg == "—" {
        ui.label(RichText::new(avg).color(theme::muted(ui)))
            .on_hover_text("Not enough history for a 5-minute average yet.");
    } else {
        ui.label(avg);
    }
}

fn trend_cell(ui: &mut egui::Ui, spark: &[f32]) {
    let width = (ui.available_width() - 4.0).max(40.0);
    sparkline_max(ui, spark, egui::vec2(width, 20.0), theme::series(ui, 0), None);
}
