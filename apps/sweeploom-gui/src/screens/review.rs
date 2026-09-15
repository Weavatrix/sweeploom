//! Review generated cleanup, then apply after revalidation.

use eframe::egui::{self, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_core::auto_eligible;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, row_caption, safety_text, short_path};
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{self, page_title, table_scroll_height};
use egui_extras::{Column, TableBuilder};
use sweeploom_dev::ReviewRow;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Review",
        "Cargo/Node/Python generated output. npm is here, not Browser. Checkbox selects; stripes are not selection.",
    );
    widgets::toolbar(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let rebuild = if app.scanning {
                "Rebuilding…"
            } else {
                "Rebuild"
            };
            if crate::widgets::pointer(ui.add_enabled(!app.scanning, egui::Button::new(rebuild)))
                .clicked()
            {
                app.rebuild_review();
            }
            if widgets::apply_button(ui, "Clean").clicked() {
                app.apply_review();
            }
            ui.add(egui::TextEdit::singleline(&mut app.free_gb).desired_width(36.0));
            ui.label("GB");
            if widgets::primary_button(ui, "Select")
                .on_hover_text("Select cheapest auto-eligible rows until this many GB")
                .clicked()
            {
                app.select_to_free();
            }
            if !app.review.is_empty() {
                let selected: u64 = app
                    .review
                    .iter()
                    .filter(|row| row.selected)
                    .map(|row| row.candidate.logical_bytes)
                    .sum();
                ui.label(
                    RichText::new(format!("{} · {}", app.review.len(), format_bytes(selected)))
                        .color(theme::muted(ui)),
                );
            }
        });
    });
    widgets::action_note(ui, app.action_message.as_deref());
    if let Some(receipt) = &app.last_receipt {
        let delta = match receipt.actual_free_space_delta {
            Some(value) => format_bytes(value.unsigned_abs()),
            None => "not measured".to_owned(),
        };
        ui.label(
            RichText::new(format!(
                "deleted {} · skipped {} · failed {} · disk delta {delta}",
                receipt.counts.deleted, receipt.counts.skipped_changed, receipt.counts.failed
            ))
            .color(theme::muted(ui)),
        );
    }
    if app.review.is_empty() {
        if app.scanning {
            ui.label("Rebuilding in the background.");
        } else {
            ui.label("Rebuild for temp/Downloads, or scan Explorer for project artifacts.");
        }
        return;
    }
    draw_review_table(app, ui);
}

fn can_select(row: &ReviewRow) -> bool {
    !row.candidate.safety.is_blocked() && row.candidate.deletion != DeletionStrategy::InspectOnly
}

fn can_auto_select(row: &ReviewRow) -> bool {
    can_select(row) && auto_eligible(&row.candidate, std::time::SystemTime::now())
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
        .id_salt("review-grid")
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(36.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(80.0).clip(true))
        .column(Column::exact(168.0).clip(true))
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
            body.rows(widgets::TABLE_ROW, row_count, |mut row| {
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
        let label = ui.add(
            egui::Label::new(RichText::new(name).size(15.0))
                .truncate()
                .selectable(false),
        );
        label.on_hover_text(path);
    });
    row.col(|ui| {
        ui.label(&size);
    });
    row.col(|ui| {
        ui.label(&rebuild);
    });
    row.col(|ui| {
        if blocked {
            widgets::chip(
                ui,
                &safety,
                if ui.visuals().dark_mode {
                    eframe::egui::Color32::from_rgb(52, 32, 26)
                } else {
                    eframe::egui::Color32::from_rgb(248, 228, 214)
                },
                theme::warn(),
            );
        } else {
            widgets::chip(ui, &safety, ui.visuals().faint_bg_color, theme::muted(ui));
        }
    });
}

impl SweepLoomApp {
    /// Fill review from project discovery plus temp / Downloads / AI.
    ///
    /// Discovery walks generated trees and must not run on the UI thread.
    pub fn rebuild_review(&mut self) {
        if self.scanning {
            return;
        }
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        let root = std::path::PathBuf::from(self.scan_root.trim());
        let inventory = self
            .inventory
            .as_ref()
            .map(|item| item.projects.clone())
            .unwrap_or_default();
        let current = self.current_project.as_ref().map(|item| item.0.clone());
        self.scanning = true;
        self.action_message = Some("Rebuilding review in the background…".to_owned());
        self.rebuild_rx = Some(crate::scan_job::spawn_review(
            root,
            inventory,
            current,
            processes,
            self.locations.clone(),
        ));
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
            .filter(|(_, row)| can_auto_select(row))
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
        if self.apply_rx.is_some() {
            self.action_message = Some("A cleanup is already running.".to_owned());
            return;
        }
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
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        self.action_message = Some("Applying cleanup in the background…".to_owned());
        self.apply_rx = Some(crate::scan_job::spawn_apply(selected, processes));
    }
}
