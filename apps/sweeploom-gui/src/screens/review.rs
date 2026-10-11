//! Review generated cleanup, then apply after revalidation.

use eframe::egui::{self, RichText};
use sweeploom_core::DeletionStrategy;
use sweeploom_core::auto_eligible;

use crate::app::SweepLoomApp;
use crate::format::{candidate_caption, format_bytes, safety_text, short_path};
use crate::sort::{Col, Sort, header_cell};
use crate::theme;
use crate::widgets::{self, Segment, table_scroll_height};
use egui_extras::Column;
use sweeploom_dev::ReviewRow;

/// Cleanup → Project output: generated build output, dependencies, caches and temp files.
pub fn ui_review(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    summary(app, ui);
    widgets::toolbar(ui, |ui| {
        let rebuild = if app.scanning {
            "Rebuilding…"
        } else {
            "Rebuild"
        };
        if widgets::button(ui, rebuild, !app.scanning).clicked() {
            app.rebuild_review();
        }
        let paths = app
            .review
            .iter()
            .filter(|row| row.selected)
            .map(|row| row.candidate.path.clone())
            .collect::<Vec<_>>();
        crate::disk_actions::toolbar(app, ui, &paths);
        ui.add(egui::Separator::default().vertical().spacing(theme::MD));
        if widgets::input_group(
            ui,
            "Free up",
            Some((&mut app.free_gb, "GB")),
            "Select",
            "Select the cheapest auto-eligible rows until this many GB",
        ) {
            app.select_to_free();
        }
        if !app.review.is_empty() {
            widgets::caption(
                ui,
                format!("{} selected", format_bytes(selected_bytes(&app.review))),
            );
        }
    });
    widgets::action_note(ui, app.action_message.as_deref());
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
        .column(Column::exact(34.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(80.0).clip(true))
        .column(Column::exact(168.0).clip(true))
        .header(30.0, |mut header| {
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
            body.rows(widgets::TABLE_ROW_TALL, row_count, |mut row| {
                let index = order.get(row.index()).copied().unwrap_or(0);
                row::fill_review_row(
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

/// Unique reclaimable estimate across Project output rows (nested paths count once).
pub(crate) fn review_bytes(app: &SweepLoomApp) -> Option<(u64, bool)> {
    if app.review.is_empty() {
        return None;
    }
    let unique = sweeploom_core::reclaim_set(
        &app.review
            .iter()
            .map(|row| row.candidate.clone())
            .collect::<Vec<_>>(),
    );
    Some((
        unique.unique_estimate.bytes,
        unique.unique_estimate.complete,
    ))
}

/// Breakdown card: build output, dependencies, caches, AI stores, temp.
fn summary(app: &SweepLoomApp, ui: &mut egui::Ui) {
    const GROUPS: [&str; 5] = [
        "Build output",
        "Dependencies",
        "Caches",
        "AI stores",
        "Temp, logs & downloads",
    ];
    let mut sums = [(0_u64, 0_usize); GROUPS.len()];
    for row in &app.review {
        let slot = &mut sums[group(row.candidate.kind)];
        slot.0 = slot.0.saturating_add(row.candidate.logical_bytes);
        slot.1 += 1;
    }
    let segments: Vec<Segment> = GROUPS
        .iter()
        .zip(sums)
        .enumerate()
        .filter(|(_, (_, (bytes, _)))| *bytes > 0)
        .map(|(index, (label, (bytes, count)))| Segment {
            label: format!("{label} · {count}"),
            value: bytes as f64,
            detail: format_bytes(bytes),
            color: if index == 4 {
                theme::series_rest(ui)
            } else {
                theme::series(ui, index)
            },
        })
        .collect();
    let listed: u64 = sums.iter().map(|(bytes, _)| *bytes).sum();
    let total = review_bytes(app).map_or_else(
        || "Rebuild to measure".to_owned(),
        |(bytes, complete)| {
            format!(
                "{} reclaimable · {} listed in {} rows",
                crate::format::format_bytes_bound(bytes, complete),
                format_bytes(listed),
                app.review.len()
            )
        },
    );
    widgets::card(ui, |ui| {
        widgets::breakdown(ui, "Project output", &total, &segments);
        if app.scanning {
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                ui.spinner();
                widgets::caption(
                    ui,
                    "Rebuilding in the background; rows already shown stay until fresh sizes land.",
                );
            });
        }
    });
    ui.add_space(theme::MD);
}

fn group(kind: sweeploom_core::CandidateKind) -> usize {
    use sweeploom_core::CandidateKind as K;
    match kind {
        K::BuildArtifact => 0,
        K::DependencyTree => 1,
        K::PackageCache | K::ToolCache | K::IdeCache | K::BrowserCache => 2,
        K::AiSession | K::AiCache | K::AgentWorktree => 3,
        _ => 4,
    }
}

#[path = "review_ops.rs"]
mod ops;
#[path = "review_row.rs"]
mod row;
#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
