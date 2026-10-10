//! Folder scans and Review rebuilds run off the UI thread.
//!
//! Results already on screen stay there while a rescan runs; each finished
//! piece replaces only the rows it covers.

#[path = "scan_merge.rs"]
mod merge;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::thread;

use crossbeam_channel::{Receiver, unbounded};
use sweeploom_core::{Candidate, ExecutionContext, ProcessSnapshot, Receipt};
use sweeploom_dev::ReviewRow;
use sweeploom_exec::{apply_plan_with, build_plan_with};
use sweeploom_platform::UserLocations;
use sweeploom_storage::{InventoryLimits, InventoryReport, ScanTick, scan_inventory_with};

use crate::app::SweepLoomApp;
use crate::review_extra;

/// Result of a Review-only rebuild. Inventory is left untouched.
pub type RebuildOutcome = Result<(Vec<PathBuf>, Vec<ReviewRow>), String>;

/// Project discovery is available while cleanup candidates are still being sized.
#[derive(Debug)]
pub enum RebuildMsg {
    /// Discovered roots, before artifact and AI-store measurements.
    Projects(Vec<PathBuf>),
    /// One project's (or the extra stores') rows, as soon as they are sized.
    Rows(Vec<ReviewRow>),
    /// Final cleanup listing or error.
    Finished(RebuildOutcome),
}

/// Progress or finished Explorer walk.
pub enum ScanMsg {
    /// Counters (and maybe a shallow folder preview) from the walker.
    Progress(ScanTick),
    /// Full inspector tree. Review rows follow later.
    Tree(InventoryReport),
    /// The folder walk finished, independently of the Review rebuild.
    Finished(Result<(), String>),
}

/// Start a home/folder walk. The UI polls [`Receiver::try_recv`].
#[must_use]
pub fn spawn(root: PathBuf) -> Receiver<ScanMsg> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let tick_tx = tx.clone();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let report = scan_inventory_with(&root, InventoryLimits::gui(), |tick| {
                let _ = tick_tx.send(ScanMsg::Progress(tick));
            })
            .map_err(|error| error.to_string())?;
            let _ = tick_tx.send(ScanMsg::Tree(report));
            Ok(())
        }))
        .unwrap_or_else(|_| Err("scan panicked".to_owned()));
        let _ = tx.send(ScanMsg::Finished(outcome));
    });
    rx
}

/// Inspect AI stores off the UI thread.
#[must_use]
pub fn spawn_ai(locations: UserLocations) -> Receiver<Result<Vec<sweeploom_ai::AiOffer>, String>> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            sweeploom_ai::inspect_offers(&locations)
        }))
        .map_err(|_| "AI listing panicked".to_owned());
        let _ = tx.send(outcome);
    });
    rx
}

/// Discover projects and size generated trees without blocking the window.
/// Rows stream per project; temp / Downloads / AI stores are sized alongside.
#[must_use]
pub fn spawn_review(
    inventory_root: Option<PathBuf>,
    inventory_projects: Vec<PathBuf>,
    current_project: Option<PathBuf>,
    processes: Vec<ProcessSnapshot>,
    locations: UserLocations,
) -> Receiver<RebuildMsg> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let projects = review_extra::project_roots(
                inventory_root.as_deref(),
                &locations,
                &inventory_projects,
                current_project.as_deref(),
            );
            let _ = tx.send(RebuildMsg::Projects(projects.clone()));
            let (mut rows, extra) = thread::scope(|scope| {
                let extra = scope.spawn(|| {
                    let rows = review_extra::extra_rows(&locations);
                    let _ = tx.send(RebuildMsg::Rows(rows.clone()));
                    rows
                });
                let rows = sweeploom_dev::collect_review_with(&projects, &processes, |rows| {
                    let _ = tx.send(RebuildMsg::Rows(rows.to_vec()));
                });
                (rows, extra.join().unwrap_or_default())
            });
            rows.extend(extra);
            (projects, rows)
        }))
        .map_err(|_| "rebuild review panicked".to_owned());
        let _ = tx.send(RebuildMsg::Finished(outcome));
    });
    rx
}

/// Apply a cleanup plan off the UI thread.
#[must_use]
pub fn spawn_apply(
    candidates: Vec<Candidate>,
    processes: Vec<ProcessSnapshot>,
) -> Receiver<(String, Receipt)> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let ctx = ExecutionContext::observed(&processes);
            let plan = build_plan_with(&candidates, None, &ctx);
            let (report, receipt) = apply_plan_with(&plan, &ctx);
            let details = report
                .skipped
                .iter()
                .map(|(id, reason)| format!("candidate {}: {reason:?}", id.0))
                .chain(
                    report
                        .failures
                        .iter()
                        .map(|(id, reason)| format!("candidate {}: {reason}", id.0)),
                )
                .collect::<Vec<_>>()
                .join("; ");
            let summary = format!(
                "deleted={} skipped={} failed={} planned={} {}",
                report.counts.deleted,
                report.skipped.len(),
                report.counts.failed,
                crate::format::format_bytes(receipt.estimated_physical_bytes),
                details
            );
            (summary, receipt)
        }))
        .unwrap_or_else(|_| {
            (
                "apply panicked".to_owned(),
                Receipt {
                    plan: sweeploom_core::PlanId(0),
                    started: std::time::SystemTime::now(),
                    finished: std::time::SystemTime::now(),
                    selected_logical_bytes: 0,
                    estimated_physical_bytes: 0,
                    actual_free_space_delta: None,
                    counts: sweeploom_core::ReceiptCounts::default(),
                },
            )
        });
        let _ = tx.send(outcome);
    });
    rx
}

impl SweepLoomApp {
    /// Walk `scan_root`. Rescanning the shown root keeps its tree, expansion and
    /// selection on screen until the new tree replaces them.
    pub(crate) fn run_scan(&mut self) {
        if self.scan_rx.is_some() {
            return;
        }
        let root = PathBuf::from(self.scan_root.trim());
        let same_root = self.inventory.as_ref().is_some_and(|report| {
            report.root == root || std::fs::canonicalize(&root).is_ok_and(|path| path == report.root)
        });
        if !same_root {
            self.inventory = None;
            self.inventory_at = None;
            self.inventory_cached = false;
            self.selected_explorer.clear();
            self.expanded_explorer.clear();
        }
        self.scanning = true;
        self.scan_entries = 0;
        self.scan_bytes = 0;
        self.scan_hint.clear();
        self.inventory_error = None;
        self.action_message = Some(format!("Scanning {}…", root.display()));
        self.scan_rx = Some(spawn(root));
    }

    /// Re-size AI stores. Earlier offers stay visible until the new listing lands.
    pub(crate) fn run_ai_listing(&mut self) {
        if self.ai_listing {
            return;
        }
        self.ai_listing = true;
        self.action_message = Some("Sizing AI stores…".to_owned());
        self.ai_rx = Some(spawn_ai(self.locations.clone()));
    }

    pub(crate) fn poll_disk(&mut self) {
        take_scan(self);
        take_ai(self);
        merge::take_rebuild(self);
        take_apply_job(self);
    }
}

fn take_apply_job(app: &mut SweepLoomApp) {
    let Some(rx) = &app.apply_rx else {
        return;
    };
    let Ok((summary, receipt)) = rx.try_recv() else {
        return;
    };
    app.apply_rx = None;
    app.last_receipt = Some(receipt);
    app.last_cleanup_summary = Some(summary.clone());
    app.action_message = Some(summary);
    app.rebuild_review();
}

fn take_scan(app: &mut SweepLoomApp) {
    let Some(rx) = app.scan_rx.clone() else {
        return;
    };
    let mut latest_tick = None;
    while let Ok(message) = rx.try_recv() {
        match message {
            ScanMsg::Progress(tick) => latest_tick = Some(tick),
            ScanMsg::Tree(report) => {
                latest_tick = None;
                merge::apply_tree(app, report);
            }
            ScanMsg::Finished(outcome) => {
                latest_tick = None;
                apply_finished(app, outcome);
                break;
            }
        }
    }
    if let Some(tick) = latest_tick {
        merge::apply_tick(app, tick);
    }
}

fn apply_finished(app: &mut SweepLoomApp, outcome: Result<(), String>) {
    app.scan_rx = None;
    app.scanning = app.rebuild_rx.is_some();
    match outcome {
        Ok(()) => {
            app.inventory_error = None;
            // Review only reruns when the walk found projects discovery did not know.
            if app.review_after_scan && app.rebuild_rx.is_none() {
                app.review_after_scan = false;
                app.rebuild_review();
            }
        }
        Err(error) => app.inventory_error = Some(error),
    }
}

fn take_ai(app: &mut SweepLoomApp) {
    let outcome = {
        let Some(rx) = &app.ai_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(outcome) => outcome,
            Err(_) => return,
        }
    };
    app.ai_rx = None;
    app.ai_listing = false;
    match outcome {
        Ok(offers) => {
            let n = offers.len();
            let values = offers
                .iter()
                .flat_map(|offer| {
                    std::iter::once(crate::scan_history::candidate_value(
                        &offer.candidate,
                        !offer.capped,
                    ))
                    .chain(offer.entries.iter().map(|entry| {
                        crate::scan_history::candidate_value(&entry.candidate, !offer.capped)
                    }))
                })
                .collect();
            app.scan_history.record_batch(
                crate::scan_history::Source::Ai,
                values,
                crate::scan_history::now_ms(),
            );
            app.ai_offers = Some(offers);
            app.action_message = Some(format!("{n} AI store(s) sized"));
        }
        Err(error) => app.action_message = Some(error),
    }
}

#[cfg(test)]
#[path = "scan_job_tests.rs"]
mod tests;
