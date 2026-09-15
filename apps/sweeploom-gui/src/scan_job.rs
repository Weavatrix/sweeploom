//! Folder scans and Review rebuilds run off the UI thread.

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

/// Progress or finished Explorer walk.
pub enum ScanMsg {
    /// Counters (and maybe a shallow folder preview) from the walker.
    Progress(ScanTick),
    /// Full inspector tree. Review rows follow later.
    Tree(InventoryReport),
    /// Review list after the tree, or a walk/review error.
    Finished(Result<Vec<ReviewRow>, String>),
}

/// Start a home/folder walk. The UI polls [`Receiver::try_recv`].
#[must_use]
pub fn spawn(
    root: PathBuf,
    processes: Vec<ProcessSnapshot>,
    locations: UserLocations,
) -> Receiver<ScanMsg> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let tick_tx = tx.clone();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let report = scan_inventory_with(&root, InventoryLimits::gui(), |tick| {
                let _ = tick_tx.send(ScanMsg::Progress(tick));
            })
            .map_err(|error| error.to_string())?;
            let projects = report.projects.clone();
            let scan_root = report.root.clone();
            let _ = tick_tx.send(ScanMsg::Tree(report));
            Ok(review_extra::all_rows(
                &scan_root, &locations, &projects, &processes,
            ))
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
#[must_use]
pub fn spawn_review(
    scan_root: PathBuf,
    inventory_projects: Vec<PathBuf>,
    current_project: Option<PathBuf>,
    processes: Vec<ProcessSnapshot>,
    locations: UserLocations,
) -> Receiver<RebuildOutcome> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let built = review_extra::assemble(
                &scan_root,
                &locations,
                &inventory_projects,
                current_project.as_deref(),
                &processes,
            );
            (built.projects, built.rows)
        }))
        .map_err(|_| "rebuild review panicked".to_owned());
        let _ = tx.send(outcome);
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
            let summary = format!(
                "deleted={} skipped_changed={} failed={} planned={}",
                report.counts.deleted,
                report.counts.skipped_changed,
                report.counts.failed,
                crate::format::format_bytes(receipt.estimated_physical_bytes)
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
    pub(crate) fn run_scan(&mut self) {
        if self.scanning {
            return;
        }
        let root = PathBuf::from(self.scan_root.trim());
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        self.scanning = true;
        self.scan_entries = 0;
        self.scan_bytes = 0;
        self.scan_hint.clear();
        self.inventory_error = None;
        self.action_message = Some(format!("Scanning {}…", root.display()));
        self.scan_rx = Some(spawn(root, processes, self.locations.clone()));
    }

    pub(crate) fn run_ai_listing(&mut self) {
        if self.ai_listing {
            return;
        }
        self.ai_listing = true;
        self.ai_offers = None;
        self.action_message = Some("Sizing AI stores…".to_owned());
        self.ai_rx = Some(spawn_ai(self.locations.clone()));
    }

    pub(crate) fn poll_disk(&mut self) {
        if take_scan(self) {
            return;
        }
        take_ai(self);
        take_rebuild(self);
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
    app.action_message = Some(summary);
    app.rebuild_review();
}

fn take_scan(app: &mut SweepLoomApp) -> bool {
    let message = {
        let Some(rx) = &app.scan_rx else {
            return false;
        };
        match rx.try_recv() {
            Ok(message) => message,
            Err(_) => return false,
        }
    };
    match message {
        ScanMsg::Progress(tick) => apply_tick(app, tick),
        ScanMsg::Tree(report) => apply_tree(app, report),
        ScanMsg::Finished(outcome) => apply_finished(app, outcome),
    }
    true
}

fn apply_finished(app: &mut SweepLoomApp, outcome: Result<Vec<ReviewRow>, String>) {
    app.scan_rx = None;
    app.scanning = false;
    match outcome {
        Ok(rows) => {
            let n = rows.len();
            app.review = rows;
            app.inventory_error = None;
            app.action_message = Some(format!("{n} candidates"));
        }
        Err(error) => app.inventory_error = Some(error),
    }
}

fn apply_tree(app: &mut SweepLoomApp, report: InventoryReport) {
    app.scan_entries = report.entries;
    app.scan_bytes = report.tree.logical_bytes;
    app.project_roots = report.projects.clone();
    app.inventory = Some(report);
    app.expanded_explorer.clear();
    app.scanning = false;
    app.action_message = Some("Folders ready. Building Review in the background…".to_owned());
}

fn apply_tick(app: &mut SweepLoomApp, tick: ScanTick) {
    app.scan_entries = tick.entries;
    app.scan_bytes = tick.logical_bytes;
    app.scan_hint = tick.hint;
    if let Some(tree) = tick.root {
        let root = PathBuf::from(app.scan_root.trim());
        app.inventory = Some(InventoryReport {
            root,
            tree,
            projects: app.project_roots.clone(),
            project_bytes: app
                .inventory
                .as_ref()
                .map(|item| item.project_bytes.clone())
                .unwrap_or_default(),
            entries: tick.entries,
            errors: 0,
            capped: false,
        });
    }
}

fn take_rebuild(app: &mut SweepLoomApp) {
    let Some(rx) = &app.rebuild_rx else {
        return;
    };
    let Ok(outcome) = rx.try_recv() else {
        return;
    };
    app.rebuild_rx = None;
    app.scanning = false;
    match outcome {
        Ok((projects, rows)) => {
            let n = rows.len();
            app.project_roots = projects;
            app.review = rows;
            app.action_message = Some(format!("{n} candidates"));
        }
        Err(error) => app.action_message = Some(error),
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
            app.ai_offers = Some(offers);
            app.action_message = Some(format!("{n} AI store(s) sized"));
        }
        Err(error) => app.action_message = Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn isolated_locations(root: PathBuf) -> UserLocations {
        UserLocations {
            downloads: Some(root.join("Downloads")),
            temp: root.join("tmp"),
            cache: None,
            app_config: root.join("cfg"),
            app_data: root.join("data"),
            home: root,
        }
    }

    #[test]
    fn review_thread_does_not_panic_on_an_empty_tree() {
        let root = std::env::temp_dir().join(format!(
            "sweeploom-rebuild-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|item| item.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(root.join("tmp")).unwrap();
        let rx = spawn_review(
            root.clone(),
            Vec::new(),
            None,
            Vec::new(),
            isolated_locations(root.clone()),
        );
        let outcome = rx
            .recv_timeout(Duration::from_secs(30))
            .expect("rebuild finished");
        let _ = fs::remove_dir_all(&root);
        assert!(outcome.is_ok(), "{outcome:?}");
    }
}
