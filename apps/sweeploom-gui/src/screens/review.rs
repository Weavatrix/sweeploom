//! Review generated cleanup, then apply after revalidation.

use eframe::egui::{self, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_core::auto_eligible;

use crate::app::SweepLoomApp;
use crate::format::{candidate_caption, format_bytes, safety_text, short_path};
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{self, page_title, table_scroll_height};
use egui_extras::Column;
use sweeploom_dev::ReviewRow;

pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Review",
        "Select rows to inspect in Finder, clean generated output, or explicitly move files to Trash.",
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
            let paths = app
                .review
                .iter()
                .filter(|row| row.selected)
                .map(|row| row.candidate.path.clone())
                .collect::<Vec<_>>();
            crate::disk_actions::toolbar(app, ui, &paths);
            ui.add(egui::TextEdit::singleline(&mut app.free_gb).desired_width(36.0));
            ui.label("GB");
            if widgets::primary_button(ui, "Select")
                .on_hover_text("Select cheapest auto-eligible rows until this many GB")
                .clicked()
            {
                app.select_to_free();
            }
            if !app.review.is_empty() {
                let selected = selected_bytes(&app.review);
                ui.label(
                    RichText::new(format!("{} · {}", app.review.len(), format_bytes(selected)))
                        .color(theme::muted(ui)),
                );
            }
        });
    });
    widgets::action_note(ui, app.action_message.as_deref());
    crate::disk_history::history_link(app, ui);
    widgets::action_note(ui, app.last_cleanup_summary.as_deref());
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

fn selected_bytes(rows: &[ReviewRow]) -> u64 {
    let paths = sweeploom_core::unique_roots(
        rows.iter()
            .filter(|row| row.selected)
            .map(|row| row.candidate.path.as_path()),
    );
    paths
        .iter()
        .filter_map(|path| {
            rows.iter()
                .find(|row| row.selected && row.candidate.path == *path)
        })
        .map(|row| {
            row.candidate
                .allocated_bytes
                .unwrap_or(row.candidate.logical_bytes)
        })
        .sum()
}

fn can_select(_row: &ReviewRow) -> bool {
    true // Selection is independent of the action allowed by the safety policy.
}

fn can_auto_select(row: &ReviewRow) -> bool {
    crate::disk_actions::can_clean(&row.candidate)
        && auto_eligible(&row.candidate, std::time::SystemTime::now())
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
    let mut action = None;
    crate::widgets::table(ui, "review-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(36.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(80.0).clip(true))
        .column(Column::exact(168.0).clip(true))
        .header(32.0, |mut header| {
            header.col(|ui| {
                if crate::widgets::check(ui, &mut all).changed() {
                    for row in &mut app.review {
                        if can_select(row) {
                            row.selected = all;
                        }
                    }
                }
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| {
                ui.strong("Change");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Rebuild"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Safety, "Safety"));
        })
        .body(|body| {
            body.rows(46.0, row_count, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(0);
                fill_review_row(
                    &mut app.review,
                    &mut row,
                    index,
                    &mut action,
                    &app.scan_history,
                );
            });
        });
    app.review_sort = sort;
    if let Some(action) = action {
        app.path_action(action);
    }
}

fn review_order(rows: &[ReviewRow], sort: Sort) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&left, &right| {
        let a = &rows[left];
        let b = &rows[right];
        match sort.col {
            Col::Name => candidate_caption(&a.title, &a.candidate.path)
                .to_lowercase()
                .cmp(&candidate_caption(&b.title, &b.candidate.path).to_lowercase()),
            Col::Safety => a.candidate.safety.level.cmp(&b.candidate.safety.level),
            Col::Status => a.candidate.rebuild.cost.cmp(&b.candidate.rebuild.cost),
            _ => a
                .candidate
                .allocated_bytes
                .unwrap_or(a.candidate.logical_bytes)
                .cmp(
                    &b.candidate
                        .allocated_bytes
                        .unwrap_or(b.candidate.logical_bytes),
                ),
        }
        .then(a.candidate.path.cmp(&b.candidate.path))
    });
    if sort.desc {
        order.reverse();
    }
    order
}

fn fill_review_row(
    rows: &mut [ReviewRow],
    row: &mut egui_extras::TableRow<'_, '_>,
    index: usize,
    action: &mut Option<crate::disk_actions::PathAction>,
    history: &crate::scan_history::ScanHistory,
) {
    let Some(item) = rows.get(index) else {
        return;
    };
    let blocked = item.candidate.safety.is_blocked();
    let inspect_only = item.candidate.deletion == DeletionStrategy::InspectOnly;
    let name = candidate_caption(&item.title, &item.candidate.path);
    let path = short_path(&item.candidate.path);
    let size = format_bytes(
        item.candidate
            .allocated_bytes
            .unwrap_or(item.candidate.logical_bytes),
    );
    let rebuild = item.candidate.rebuild.cost.label().to_owned();
    let safety = if inspect_only && !blocked {
        if item.candidate.user_policy == sweeploom_core::UserPolicy::NeverClean {
            "Protected store — inspect".to_owned()
        } else {
            "Manual Trash".to_owned()
        }
    } else {
        safety_text(&item.candidate.safety)
    };
    let mut selected = item.selected;
    let candidate_path = item.candidate.path.clone();
    row.set_selected(selected);
    row.col(|ui| {
        crate::widgets::check(ui, &mut selected);
    });
    row.col(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add(
                egui::Label::new(RichText::new(name).size(15.0))
                    .truncate()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(RichText::new(&path).size(12.5).color(theme::muted(ui)))
                    .truncate()
                    .selectable(false),
            );
        })
        .response
        .interact(egui::Sense::click())
        .on_hover_text(path)
        .clicked()
        .then(|| selected = !selected);
    });
    row.col(|ui| {
        ui.label(&size);
    });
    row.col(|ui| {
        crate::scan_history::trend(
            ui,
            history,
            crate::scan_history::Source::Review,
            &candidate_path,
        );
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
    crate::disk_actions::menu(&row.response(), &candidate_path, action);
    if let Some(item) = rows.get_mut(index) {
        item.selected = selected;
    }
}

impl SweepLoomApp {
    /// Fill review from project discovery plus temp / Downloads / AI.
    ///
    /// Discovery walks generated trees and must not run on the UI thread. Rows
    /// and project sizes already shown stay until fresh values replace them.
    pub fn rebuild_review(&mut self) {
        if self.scanning {
            return;
        }
        self.project_sizes.invalidate();
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        let (root, inventory) = self.inventory.as_ref().map_or((None, Vec::new()), |item| {
            (Some(item.root.clone()), item.projects.clone())
        });
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
                std::cmp::Reverse(
                    self.review[index]
                        .candidate
                        .allocated_bytes
                        .unwrap_or(self.review[index].candidate.logical_bytes),
                ),
            )
        });
        let mut acc = 0_u64;
        for index in order {
            if acc >= target {
                break;
            }
            if self.review.iter().any(|row| {
                row.selected
                    && self.review[index]
                        .candidate
                        .path
                        .starts_with(&row.candidate.path)
            }) {
                continue;
            }
            self.review[index].selected = true;
            acc = acc.saturating_add(
                self.review[index]
                    .candidate
                    .allocated_bytes
                    .unwrap_or(self.review[index].candidate.logical_bytes),
            );
        }
        self.action_message = Some(format!(
            "selected {} toward {}",
            format_bytes(acc),
            format_bytes(target)
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_core::*;
    fn row(path: &str, bytes: u64) -> ReviewRow {
        ReviewRow {
            title: "Downloads".into(),
            selected: true,
            candidate: Candidate {
                id: CandidateId(bytes),
                kind: CandidateKind::OldInstaller,
                owner: CandidateOwner::User,
                path: path.into(),
                logical_bytes: bytes,
                allocated_bytes: None,
                file_count: 1,
                activity: ActivityEvidence::default(),
                safety: SafetyAssessment::review(),
                rebuild: RebuildAssessment::default(),
                deletion: DeletionStrategy::InspectOnly,
                evidence: vec![],
                user_policy: UserPolicy::Default,
            },
        }
    }
    #[test]
    fn selection_does_not_authorize_generated_deletion_and_sizes_do_not_overlap() {
        let mut item = row("/home/Downloads/installer.dmg", 10);
        assert!(can_select(&item));
        assert!(!can_auto_select(&item));
        assert!(!crate::disk_actions::can_clean(&item.candidate));
        item.candidate.safety = SafetyAssessment::blocked(Blocker::ProtectedPath);
        assert!(can_select(&item));
        assert!(!crate::disk_actions::can_clean(&item.candidate));
        let rows = [row("/home/target", 100), row("/home/target/debug", 80)];
        assert_eq!(selected_bytes(&rows), 100);
    }
}
