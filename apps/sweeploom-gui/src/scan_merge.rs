//! Fold background results into what is already on screen.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use sweeploom_core::CandidateId;
use sweeploom_dev::ReviewRow;
use sweeploom_storage::{InventoryReport, ScanTick, is_discoverable_below};

use super::RebuildMsg;
use crate::app::SweepLoomApp;

pub(super) fn take_rebuild(app: &mut SweepLoomApp) {
    let Some(rx) = app.rebuild_rx.clone() else {
        return;
    };
    while let Ok(message) = rx.try_recv() {
        let outcome = match message {
            RebuildMsg::Projects(projects) => {
                app.scan_history.remember_projects(&projects);
                app.project_roots = projects;
                app.action_message =
                    Some("Projects found. Sizing cleanup candidates in the background…".into());
                continue;
            }
            RebuildMsg::Rows(rows) => {
                let rows = with_disk_bytes(app.inventory.as_ref(), rows);
                merge_rows(&mut app.review, rows);
                continue;
            }
            RebuildMsg::Finished(outcome) => outcome,
        };
        app.rebuild_rx = None;
        app.scanning = app.scan_rx.is_some();
        app.project_sizes.forget_facts();
        match outcome {
            Ok((projects, rows)) => {
                let rows = with_disk_bytes(app.inventory.as_ref(), rows);
                let n = rows.len();
                app.scan_history.record_review(&rows);
                app.project_roots = projects;
                replace_rows(&mut app.review, rows);
                app.action_message = Some(format!("{n} candidates"));
            }
            Err(error) => app.action_message = Some(error),
        }
        if app.review_after_scan && app.scan_rx.is_none() {
            app.review_after_scan = false;
            app.rebuild_review();
        }
    }
}

fn with_disk_bytes(report: Option<&InventoryReport>, mut rows: Vec<ReviewRow>) -> Vec<ReviewRow> {
    if let Some(report) = report {
        for row in &mut rows {
            row.candidate.allocated_bytes = report
                .folder_disk_bytes(&row.candidate.path)
                .or(row.candidate.allocated_bytes);
        }
    }
    rows
}

/// A streamed batch replaces rows for the same paths and keeps every other row.
pub(crate) fn merge_rows(review: &mut Vec<ReviewRow>, rows: Vec<ReviewRow>) {
    let mut fresh: HashMap<PathBuf, ReviewRow> = rows
        .into_iter()
        .map(|row| (row.candidate.path.clone(), row))
        .collect();
    for row in review.iter_mut() {
        if let Some(mut update) = fresh.remove(&row.candidate.path) {
            keep_choice(&mut update, row);
            *row = update;
        }
    }
    let mut added: Vec<ReviewRow> = fresh.into_values().collect();
    added.sort_by(|left, right| left.candidate.path.cmp(&right.candidate.path));
    review.extend(added);
    renumber(review);
}

/// The final list drops vanished rows; the user's choices survive by path.
pub(crate) fn replace_rows(review: &mut Vec<ReviewRow>, mut rows: Vec<ReviewRow>) {
    let previous: HashMap<PathBuf, &ReviewRow> = review
        .iter()
        .map(|row| (row.candidate.path.clone(), row))
        .collect();
    for row in &mut rows {
        if let Some(old) = previous.get(&row.candidate.path) {
            keep_choice(row, old);
        }
    }
    *review = rows;
    renumber(review);
}

fn keep_choice(update: &mut ReviewRow, old: &ReviewRow) {
    // A row that became blocked is never kept selected.
    if !update.candidate.safety.is_blocked() {
        update.selected = old.selected;
    }
}

fn renumber(review: &mut [ReviewRow]) {
    for (index, row) in review.iter_mut().enumerate() {
        row.candidate.id = CandidateId(index as u64 + 1);
    }
}

/// A finished walk adds its projects to Projects; it never replaces discovery.
pub(super) fn apply_tree(app: &mut SweepLoomApp, report: InventoryReport) {
    let at = crate::scan_history::now_ms();
    app.scan_history.record_scan(&report, at);
    app.inventory_at = Some(at);
    app.inventory_cached = false;
    app.scan_entries = report.entries;
    app.scan_bytes = report.tree.disk_bytes();
    let known: HashSet<&PathBuf> = app.project_roots.iter().collect();
    let added: Vec<PathBuf> = report
        .projects
        .iter()
        .filter(|path| !known.contains(path) && is_discoverable_below(&report.root, path))
        .cloned()
        .collect();
    app.review_after_scan |= !added.is_empty();
    app.project_roots.extend(added);
    for row in &mut app.review {
        if row.candidate.path.starts_with(&report.root) {
            row.candidate.allocated_bytes = report
                .folder_disk_bytes(&row.candidate.path)
                .or(row.candidate.allocated_bytes);
        }
    }
    let same_root = app
        .inventory
        .as_ref()
        .is_some_and(|item| item.root == report.root);
    if !same_root {
        app.expanded_explorer.clear();
    }
    app.inventory = Some(report);
    app.action_message = Some("Folders ready.".to_owned());
}

/// Stream the root preview, unless a finished tree of the same root is on screen.
pub(super) fn apply_tick(app: &mut SweepLoomApp, tick: ScanTick) {
    app.scan_entries = tick.entries;
    app.scan_bytes = tick.allocated_bytes.unwrap_or(tick.logical_bytes);
    app.scan_hint = tick.hint;
    let Some(tree) = tick.root else {
        return;
    };
    let keep = app.inventory_at.is_some()
        && app
            .inventory
            .as_ref()
            .is_some_and(|report| report.root == tree.path);
    if keep {
        return;
    }
    app.inventory = Some(InventoryReport {
        root: tree.path.clone(),
        tree,
        projects: Vec::new(),
        project_bytes: Vec::new(),
        project_disk_bytes: Vec::new(),
        entries: tick.entries,
        errors: 0,
        capped: false,
    });
}
