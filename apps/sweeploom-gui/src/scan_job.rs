//! Folder scans run off the UI thread.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::thread;

use crossbeam_channel::{Receiver, unbounded};
use sweeploom_core::ProcessSnapshot;
use sweeploom_dev::ReviewRow;
use sweeploom_platform::UserLocations;

use crate::review_extra;
use sweeploom_storage::{InventoryLimits, InventoryReport, scan_inventory};

/// Result of a background Explorer scan plus Review rebuild.
pub type ScanOutcome = Result<(InventoryReport, Vec<ReviewRow>), String>;

/// Start a bounded inventory walk. The UI polls [`Receiver::try_recv`].
#[must_use]
pub fn spawn(
    root: PathBuf,
    processes: Vec<ProcessSnapshot>,
    locations: UserLocations,
) -> Receiver<ScanOutcome> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| build(root, &processes, &locations)))
            .unwrap_or_else(|_| Err("scan panicked".to_owned()));
        let _ = tx.send(outcome);
    });
    rx
}

fn build(root: PathBuf, processes: &[ProcessSnapshot], locations: &UserLocations) -> ScanOutcome {
    let report =
        scan_inventory(&root, InventoryLimits::gui()).map_err(|error| error.to_string())?;
    let mut rows = sweeploom_dev::collect_review(&report.projects, processes);
    rows.extend(review_extra::extra_rows(locations));
    Ok((report, rows))
}
