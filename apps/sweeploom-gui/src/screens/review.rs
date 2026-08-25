//! Review generated cleanup, then apply after revalidation.

use eframe::egui::{self, Color32, RichText};
use sweeploom_exec::{apply_plan, build_plan};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.heading("Storage review");
    ui.label(
        RichText::new("SAFE generated items can be pre-selected. BLOCKED rows cannot be cleaned.")
            .weak(),
    );
    ui.horizontal(|ui| {
        if ui.button("Rebuild from last scan").clicked() {
            app.rebuild_review();
        }
        if ui.button("Clean selected").clicked() {
            app.apply_review();
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
        ui.label("Scan Explorer first, then rebuild the review.");
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
    let count = app.review.len();
    for index in 0..count {
        draw_row(app, ui, index);
    }
}

fn draw_row(app: &mut SweepLoomApp, ui: &mut egui::Ui, index: usize) {
    let blocked = app.review[index].candidate.safety.is_blocked();
    let title = app.review[index].title.clone();
    let size = format_bytes(app.review[index].candidate.logical_bytes);
    let rebuild = app.review[index].candidate.rebuild.cost;
    let blocker = app.review[index]
        .candidate
        .safety
        .blockers
        .first()
        .map(|item| format!("  BLOCKED {item:?}"))
        .unwrap_or_default();
    let mut selected = app.review[index].selected;
    ui.horizontal(|ui| {
        if blocked {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, ""));
            ui.colored_label(
                Color32::from_rgb(240, 160, 80),
                format!("{title}  {size}  rebuild={rebuild:?}{blocker}"),
            );
        } else {
            if ui.checkbox(&mut selected, "").changed() {
                app.review[index].selected = selected;
            }
            ui.label(format!("{title}  {size}  rebuild={rebuild:?}"));
        }
    });
}

impl SweepLoomApp {
    /// Fill review from the last inventory.
    pub fn rebuild_review(&mut self) {
        let Some(report) = &self.inventory else {
            self.action_message = Some("Scan Explorer first.".to_owned());
            return;
        };
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.as_slice())
            .unwrap_or(&[]);
        self.review = sweeploom_dev::collect_review(&report.projects, processes);
        self.action_message = Some(format!("{} candidates", self.review.len()));
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
