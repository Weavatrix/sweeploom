//! Ungrouped process table.

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::Col;
use crate::sort::header_cell;
use crate::widgets::table_scroll_height;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn draw(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
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
