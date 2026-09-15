//! `sweeploom clean` — review, optionally apply.

use std::path::Path;

use crate::api::{ApplyRequest, apply_cleanup, cleanup_candidates};
use crate::bytes::format_bytes;

pub fn run(root: &Path, apply: bool) {
    let rows = cleanup_candidates(root);
    if rows.is_empty() {
        println!("no generated candidates");
        return;
    }
    for row in &rows {
        println!(
            "{}\t{}\t{}{}",
            if row.selected { "[x]" } else { "[ ]" },
            format_bytes(row.logical_bytes),
            row.title,
            if row.blocked { "\tBLOCKED" } else { "" }
        );
    }
    if !apply {
        println!("dry-run; pass --apply to delete pre-selected SAFE rows after revalidation");
        return;
    }
    let report = apply_cleanup(ApplyRequest {
        confirm: true,
        root: root.to_path_buf(),
        ids: Vec::new(),
    });
    if !report.ok {
        println!(
            "{}",
            report.error.unwrap_or_else(|| "apply refused".to_owned())
        );
        return;
    }
    println!(
        "receipt={}\tdeleted={}\tskipped_changed={}\tfailed={}",
        report.receipt.unwrap_or(0),
        report.deleted,
        report.skipped_changed,
        report.failed
    );
}
