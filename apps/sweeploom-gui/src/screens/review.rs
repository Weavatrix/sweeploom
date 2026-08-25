//! Review generated cleanup, then apply after revalidation.

use crate::review_extra;
use eframe::egui::{self, Color32, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_exec::{apply_plan, build_plan};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::page_title;
use egui_extras::{Column, TableBuilder};
use sweeploom_dev::ReviewRow;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Storage review",
        "Cargo/Node/Python generated output is discovered without walking target. Dirty Git blocks auto-select, not listing.",
    );
    ui.horizontal(|ui| {
        if ui.button("Rebuild review").clicked() {
            app.rebuild_review();
        }
        if ui.button("Clean selected").clicked() {
            app.apply_review();
        }
        ui.label("Free at least");
        ui.add(egui::TextEdit::singleline(&mut app.free_gb).desired_width(48.0));
        ui.label("GB");
        if ui.button("Select to free").clicked() {
            app.select_to_free();
        }
    });
    if let Some(message) = &app.action_message {
        ui.label(message);
    }
    if let Some(receipt) = &app.last_receipt {
        ui.label(format!(
            "Receipt {}  deleted={} skipped_changed={} failed={} planned={}",
            receipt.plan.0,
            receipt.counts.deleted,
            receipt.counts.skipped_changed,
            receipt.counts.failed,
            format_bytes(receipt.estimated_physical_bytes)
        ));
    }
    ui.add_space(8.0);
    if app.review.is_empty() {
        ui.label("Rebuild review for temp/Downloads, or scan Explorer for project artifacts.");
        return;
    }
    let selected: u64 = app
        .review
        .iter()
        .filter(|row| row.selected)
        .map(|row| row.candidate.logical_bytes)
        .sum();
    ui.label(format!(
        "{} candidates · {} selected",
        app.review.len(),
        format_bytes(selected)
    ));
    draw_review_table(app, ui);
}

fn draw_review_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.review_sort;
    let order = review_order(&app.review, sort);
    let row_count = order.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(36.0))
        .column(Column::remainder().at_least(280.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(110.0))
        .column(Column::auto().at_least(140.0))
        .header(32.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Rebuild"));
            header.col(|ui| {
                ui.strong("Safety");
            });
        })
        .body(|body| {
            body.rows(28.0, row_count, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(0);
                fill_review_row(&mut app.review, &mut row, index);
            });
        });
    app.review_sort = sort;
}

fn review_order(rows: &[ReviewRow], sort: Sort) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&left, &right| {
        let a = &rows[left];
        let b = &rows[right];
        match sort.col {
            Col::Name => a.title.cmp(&b.title),
            Col::Status => a.candidate.rebuild.cost.cmp(&b.candidate.rebuild.cost),
            _ => a.candidate.logical_bytes.cmp(&b.candidate.logical_bytes),
        }
    });
    if sort.desc {
        order.reverse();
    }
    order
}

fn fill_review_row(rows: &mut [ReviewRow], row: &mut egui_extras::TableRow<'_, '_>, index: usize) {
    let Some(item) = rows.get(index) else {
        return;
    };
    let blocked = item.candidate.safety.is_blocked();
    let title = item.title.clone();
    let size = format_bytes(item.candidate.logical_bytes);
    let rebuild = format!("{:?}", item.candidate.rebuild.cost);
    let safety = if blocked {
        item.candidate
            .safety
            .blockers
            .first()
            .map(|item| format!("BLOCKED {item:?}"))
            .unwrap_or_else(|| "BLOCKED".to_owned())
    } else {
        format!("{:?}", item.candidate.safety.level)
    };
    let mut selected = item.selected;
    row.col(|ui| {
        if blocked {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, ""));
        } else if ui.checkbox(&mut selected, "").changed()
            && let Some(item) = rows.get_mut(index)
        {
            item.selected = selected;
        }
    });
    row.col(|ui| {
        ui.label(RichText::new(title).size(16.0));
    });
    row.col(|ui| {
        ui.label(&size);
    });
    row.col(|ui| {
        ui.label(&rebuild);
    });
    row.col(|ui| {
        if blocked {
            ui.colored_label(Color32::from_rgb(240, 160, 80), safety);
        } else {
            ui.label(safety);
        }
    });
}

impl SweepLoomApp {
    /// Fill review from project discovery plus temp / Downloads / AI.
    pub fn rebuild_review(&mut self) {
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.as_slice())
            .unwrap_or(&[]);
        let root = std::path::PathBuf::from(self.scan_root.trim());
        let inventory = self
            .inventory
            .as_ref()
            .map(|item| item.projects.as_slice())
            .unwrap_or(&[]);
        self.review = review_extra::all_rows(&root, &self.locations, inventory, processes);
        self.action_message = Some(format!("{} candidates", self.review.len()));
    }

    /// Pre-select the cheapest SAFE generated rows until `free_gb` is reached.
    pub fn select_to_free(&mut self) {
        let gb: f64 = self.free_gb.trim().parse().unwrap_or(0.0);
        let target = (gb * 1_000_000_000.0) as u64;
        if target == 0 {
            self.action_message = Some("Enter a size greater than 0 GB.".to_owned());
            return;
        }
        for row in &mut self.review {
            row.selected = false;
        }
        let mut order: Vec<usize> = self
            .review
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                !row.candidate.safety.is_blocked()
                    && row.candidate.deletion != DeletionStrategy::InspectOnly
            })
            .map(|(index, _)| index)
            .collect();
        order.sort_by_key(|&index| {
            (
                self.review[index].candidate.rebuild.cost,
                std::cmp::Reverse(self.review[index].candidate.logical_bytes),
            )
        });
        let mut acc = 0_u64;
        for index in order {
            if acc >= target {
                break;
            }
            self.review[index].selected = true;
            acc = acc.saturating_add(self.review[index].candidate.logical_bytes);
        }
        self.action_message = Some(format!(
            "selected {} toward {}",
            format_bytes(acc),
            format_bytes(target)
        ));
    }

    /// Apply selected unblocked rows through CleanPlan revalidation.
    pub fn apply_review(&mut self) {
        let selected: Vec<_> = self
            .review
            .iter()
            .filter(|row| row.selected && !row.candidate.safety.is_blocked())
            .map(|row| row.candidate.clone())
            .collect();
        if selected.is_empty() {
            self.action_message = Some("Nothing selected.".to_owned());
            return;
        }
        let plan = build_plan(&selected, None);
        let (report, receipt) = apply_plan(&plan);
        let summary = format!(
            "deleted={} skipped_changed={} failed={} planned={}",
            report.counts.deleted,
            report.counts.skipped_changed,
            report.counts.failed,
            format_bytes(receipt.estimated_physical_bytes)
        );
        self.last_receipt = Some(receipt);
        self.rebuild_review();
        self.action_message = Some(summary);
    }
}
