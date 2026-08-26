//! Folder scans and Review rebuilds run off the UI thread.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::thread;

use crossbeam_channel::{Receiver, unbounded};
use sweeploom_core::ProcessSnapshot;
use sweeploom_dev::ReviewRow;
use sweeploom_platform::UserLocations;
use sweeploom_storage::{InventoryLimits, InventoryReport, scan_inventory};

use crate::app::SweepLoomApp;
use crate::review_extra;

/// Result of a background Explorer scan plus Review rebuild.
pub type ScanOutcome = Result<(InventoryReport, Vec<ReviewRow>), String>;
/// Result of a Review-only rebuild. Inventory is left untouched.
pub type RebuildOutcome = Result<(Vec<PathBuf>, Vec<ReviewRow>), String>;

/// Start a bounded inventory walk. The UI polls [`Receiver::try_recv`].
#[must_use]
pub fn spawn(
    root: PathBuf,
    processes: Vec<ProcessSnapshot>,
    locations: UserLocations,
) -> Receiver<ScanOutcome> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            build_scan(root, &processes, &locations)
        }))
        .unwrap_or_else(|_| Err("scan panicked".to_owned()));
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

fn build_scan(
    root: PathBuf,
    processes: &[ProcessSnapshot],
    locations: &UserLocations,
) -> ScanOutcome {
    let report =
        scan_inventory(&root, InventoryLimits::gui()).map_err(|error| error.to_string())?;
    let rows = review_extra::all_rows(&root, locations, &report.projects, processes);
    Ok((report, rows))
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
        self.inventory_error = None;
        self.action_message = Some(format!("Scanning {}…", root.display()));
        self.scan_rx = Some(spawn(root, processes, self.locations.clone()));
    }

    pub(crate) fn poll_disk(&mut self) {
        if take_scan(self) {
            return;
        }
        take_rebuild(self);
    }
}

fn take_scan(app: &mut SweepLoomApp) -> bool {
    let Some(rx) = &app.scan_rx else {
        return false;
    };
    let Ok(outcome) = rx.try_recv() else {
        return false;
    };
    app.scan_rx = None;
    app.scanning = false;
    match outcome {
        Ok((report, rows)) => {
            let n = rows.len();
            app.project_roots = report.projects.clone();
            app.inventory = Some(report);
            app.review = rows;
            app.inventory_error = None;
            app.action_message = Some(format!("{n} candidates"));
        }
        Err(error) => app.inventory_error = Some(error),
    }
    true
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
