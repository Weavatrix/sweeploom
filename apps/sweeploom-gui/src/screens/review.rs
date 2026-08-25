//! Review generated cleanup, then apply after revalidation.

use eframe::egui::{self, Color32, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_exec::{apply_plan, build_plan};
use sweeploom_general::collect_offers;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, header_button};
use crate::widgets::page_title;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Storage review",
        "SAFE generated/temp can be pre-selected. Downloads stay REVIEW. BLOCKED cannot be cleaned.",
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
    ui.horizontal(|ui| {
        ui.label("Sort");
        header_button(ui, &mut app.review_sort, Col::Name, "Name");
        header_button(ui, &mut app.review_sort, Col::Size, "Size");
        header_button(ui, &mut app.review_sort, Col::Status, "Status");
    });
    let order = review_order(app);
    for index in order {
        draw_row(app, ui, index);
    }
}

fn review_order(app: &SweepLoomApp) -> Vec<usize> {
    let mut order: Vec<usize> = (0..app.review.len()).collect();
    order.sort_by(|&left, &right| {
        let a = &app.review[left];
        let b = &app.review[right];
        match app.review_sort.col {
            Col::Name => a.title.cmp(&b.title),
            Col::Status => a
                .candidate
                .safety
                .is_blocked()
                .cmp(&b.candidate.safety.is_blocked()),
            _ => a.candidate.logical_bytes.cmp(&b.candidate.logical_bytes),
        }
    });
    if app.review_sort.desc {
        order.reverse();
    }
    order
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
                RichText::new(format!("{title}  {size}  rebuild={rebuild:?}{blocker}")).size(16.0),
            );
        } else {
            if ui.checkbox(&mut selected, "").changed() {
                app.review[index].selected = selected;
            }
            ui.label(RichText::new(format!("{title}  {size}  rebuild={rebuild:?}")).size(16.0));
        }
    });
}

impl SweepLoomApp {
    /// Fill review from general roots plus the last project inventory.
    pub fn rebuild_review(&mut self) {
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.as_slice())
            .unwrap_or(&[]);
        let mut rows = match &self.inventory {
            Some(report) => sweeploom_dev::collect_review(&report.projects, processes),
            None => Vec::new(),
        };
        for offer in collect_offers(&self.locations) {
            rows.push(sweeploom_dev::ReviewRow {
                candidate: offer.candidate,
                selected: offer.selected,
                title: offer.title,
            });
        }
        self.review = rows;
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
