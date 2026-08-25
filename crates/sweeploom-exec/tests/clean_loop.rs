//! Scan → review → apply on a disposable Cargo fixture.

use std::fs;

use sweeploom_dev::collect_review;
use sweeploom_exec::{apply_plan, build_plan};

#[test]
fn scan_review_apply_deletes_stale_incremental() {
    let root = std::env::temp_dir().join(format!("sweeploom-loop-{}", std::process::id()));
    let incremental = root.join("target").join("incremental");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&incremental).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
    let junk = incremental.join("stale.bin");
    fs::write(&junk, vec![0_u8; 2048]).unwrap();
    let rows = collect_review(&[root.as_path()], &[]);
    let selected: Vec<_> = rows
        .into_iter()
        .filter(|row| row.selected && !row.candidate.safety.is_blocked())
        .map(|row| row.candidate)
        .collect();
    assert!(
        !selected.is_empty(),
        "Light incremental must be pre-selected for a non-git fixture"
    );
    let plan = build_plan(&selected, None);
    let (report, receipt) = apply_plan(&plan);
    assert_eq!(report.counts.deleted, 1);
    assert_eq!(receipt.counts.deleted, 1);
    assert!(!junk.exists(), "stale incremental must be removed");
    let _ = fs::remove_dir_all(&root);
}
