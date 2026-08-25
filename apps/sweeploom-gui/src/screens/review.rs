//! Review generated cleanup, then apply after revalidation.

use crate::review_extra;
use eframe::egui::{self, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_exec::{apply_plan, build_plan};

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, row_caption, safety_text, short_path};
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{page_title, table_scroll_height};
use egui_extras::{Column, TableBuilder};
use sweeploom_dev::ReviewRow;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Storage review",
        "Cargo/Node/Python generated output. Stripes are not selection. npm is here, not Browser.",
    );
    ui.horizontal_wrapped(|ui| {
        if crate::widgets::pointer(ui.button("Rebuild review")).clicked() {
            app.rebuild_review();
        }
        if crate::widgets::pointer(ui.button("Clean selected")).clicked() {
            app.apply_review();
        }
        ui.label("Free at least");
        ui.add(egui::TextEdit::singleline(&mut app.free_gb).desired_width(48.0));
        ui.label("GB");
        if crate::widgets::pointer(ui.button("Select to free")).clicked() {
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
        "{} candidates · {} selected (checkbox, not row stripe)",
        app.review.len(),
        format_bytes(selected)
    ));
    draw_review_table(app, ui);
}

fn can_select(row: &ReviewRow) -> bool {
    !row.candidate.safety.is_blocked() && row.candidate.deletion != DeletionStrategy::InspectOnly
}

fn draw_review_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.review_sort;
    let order = review_order(&app.review, sort);
    let selectable = app.review.iter().filter(|row| can_select(row)).count();
    let chosen = app
        .review
        .iter()
        .filter(|row| row.selected && can_select(row))
        .count();
    let mut all = selectable > 0 && chosen == selectable;
    let row_count = order.len();
    let height = table_scroll_height(ui);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(36.0))
        .column(Column::remainder().at_least(280.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(110.0))
        .column(Column::auto().at_least(160.0))
        .header(32.0, |mut header| {
            header.col(|ui| {
                if ui.checkbox(&mut all, "").changed() {
                    for row in &mut app.review {
                        if can_select(row) {
                            row.selected = all;
                        }
                    }
                }
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Rebuild"));
            header.col(|ui| {
                ui.strong("Safety");
            });
        })
        .body(|body| {
            body.rows(40.0, row_count, |mut row| {
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
    let inspect_only = item.candidate.deletion == DeletionStrategy::InspectOnly;
    let name = row_caption(&item.title);
    let path = short_path(&item.candidate.path);
    let size = format_bytes(item.candidate.logical_bytes);
    let rebuild = item.candidate.rebuild.cost.label().to_owned();
    let safety = safety_text(&item.candidate.safety);
    let mut selected = item.selected;
    row.set_selected(selected);
    row.col(|ui| {
        if blocked || inspect_only {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, ""));
        } else if ui.checkbox(&mut selected, "").changed()
            && let Some(item) = rows.get_mut(index)
        {
            item.selected = selected;
        }
    });
    row.col(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(name).size(15.0).strong());
            ui.label(RichText::new(path).size(12.0).color(theme::muted(ui)));
        });
    });
    row.col(|ui| {
        ui.label(&size);
    });
    row.col(|ui| {
        ui.label(&rebuild);
    });
    row.col(|ui| {
        if blocked {
            ui.colored_label(ui.visuals().error_fg_color, safety);
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
            .map(|item| item.projects.clone())
            .unwrap_or_default();
        let current = self.current_project.as_ref().map(|item| item.0.clone());
        let built = review_extra::assemble(
            &root,
            &self.locations,
            &inventory,
            current.as_deref(),
            processes,
        );
        let n = built.rows.len();
        self.project_roots = built.projects;
        self.review = built.rows;
        self.action_message = Some(format!("{n} candidates"));
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
            .filter(|(_, row)| can_select(row))
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
