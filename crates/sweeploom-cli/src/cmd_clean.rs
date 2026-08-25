//! `sweeploom clean` — review generated offers, optionally apply.

use std::path::Path;

use sweeploom_dev::collect_review;
use sweeploom_exec::{apply_plan, build_plan};
use sweeploom_storage::{InventoryLimits, scan_inventory};

use crate::bytes::format_bytes;

pub fn run(root: &Path, apply: bool) {
    let report = match scan_inventory(root, InventoryLimits::default()) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("scan failed: {error}");
            std::process::exit(1);
        }
    };
    let rows = collect_review(&report.projects, &[]);
    if rows.is_empty() {
        println!("no generated candidates");
        return;
    }
    for row in &rows {
        println!(
            "{}\t{}\t{}{}",
            if row.selected { "[x]" } else { "[ ]" },
            format_bytes(row.candidate.logical_bytes),
            row.title,
            if row.candidate.safety.is_blocked() {
                "\tBLOCKED"
            } else {
                ""
            }
        );
    }
    if !apply {
        println!("dry-run; pass --apply to delete pre-selected SAFE rows after revalidation");
        return;
    }
    let selected: Vec<_> = rows
        .into_iter()
        .filter(|row| row.selected && !row.candidate.safety.is_blocked())
        .map(|row| row.candidate)
        .collect();
    let plan = build_plan(&selected, None);
    let (report, receipt) = apply_plan(&plan);
    println!(
        "receipt={}\tdeleted={}\tskipped_changed={}\tfailed={}\tplanned={}",
        receipt.plan.0,
        report.counts.deleted,
        report.counts.skipped_changed,
        report.counts.failed,
        format_bytes(receipt.estimated_physical_bytes)
    );
}
