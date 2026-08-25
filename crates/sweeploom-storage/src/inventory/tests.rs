use std::fs;
use std::path::Path;

use super::{InventoryLimits, scan_inventory};

fn paths_match(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => left.file_name() == right.file_name(),
    }
}

#[test]
fn inventory_finds_cargo_project_and_target_bytes() {
    let root = std::env::temp_dir().join(format!("sweeploom-inventory-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
    fs::write(root.join("src").join("lib.rs"), "pub fn x() {}\n").unwrap();
    fs::write(root.join("target").join("big.bin"), vec![0_u8; 4096]).unwrap();

    let report = scan_inventory(&root, InventoryLimits::default()).expect("scan");
    assert!(
        report.projects.iter().any(|item| paths_match(item, &root)),
        "expected project root among {:?}",
        report.projects
    );
    assert!(
        report.tree.logical_bytes >= 4096,
        "logical={}",
        report.tree.logical_bytes
    );
    let target = report.tree.children.iter().find(|child| {
        child
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("target"))
    });
    assert!(target.is_some(), "target directory missing");
    let _ = fs::remove_dir_all(&root);
}
