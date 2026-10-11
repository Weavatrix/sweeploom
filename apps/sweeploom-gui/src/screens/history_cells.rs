//! History table rows (groups and member processes) and their cells.

use sweeploom_core::SessionId;

use super::super::history_rows::{HistRow, Line};
use crate::format::format_bytes;
use crate::theme;
use crate::widgets::{disclose, sparkline_max};
use eframe::egui::{self, RichText};

pub(super) fn fill_group(
    row: &mut egui_extras::TableRow<'_, '_>,
    line: &Line,
    toggle: &mut Option<String>,
) {
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

pub(super) fn fill_item(
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
    sparkline_max(
        ui,
        spark,
        egui::vec2(width, 20.0),
        theme::series(ui, 0),
        None,
    );
}
