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

#[test]
fn nested_package_json_inside_node_modules_is_not_a_project() {
    let root = std::env::temp_dir().join(format!("sweeploom-nested-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let nested = root.join("node_modules").join("left-pad");
    fs::create_dir_all(&nested).unwrap();
    fs::write(root.join("package.json"), "{}\n").unwrap();
    fs::write(nested.join("package.json"), "{}\n").unwrap();
    let report = scan_inventory(&root, InventoryLimits::default()).expect("scan");
    assert!(report.projects.iter().any(|item| paths_match(item, &root)));
    assert!(
        report
            .projects
            .iter()
            .all(|item| !item.ends_with("left-pad")),
        "nested deps must not become projects: {:?}",
        report.projects
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn discover_finds_cargo_and_skips_nested_node_modules() {
    let root = std::env::temp_dir().join(format!("sweeploom-discover-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("app").join("target").join("debug")).unwrap();
    fs::create_dir_all(root.join("app").join("node_modules").join("pkg")).unwrap();
    fs::write(
        root.join("app").join("Cargo.toml"),
        "[package]\nname=\"a\"\n",
    )
    .unwrap();
    fs::write(
        root.join("app")
            .join("node_modules")
            .join("pkg")
            .join("package.json"),
        "{}\n",
    )
    .unwrap();
    let found = super::discover_projects(&root, 32);
    assert!(
        found.iter().any(|item| item.ends_with("app")),
        "expected app project, got {found:?}"
    );
    assert!(
        found.iter().all(|item| !item.ends_with("pkg")),
        "node_modules must not be a project: {found:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn developer_roots_prefers_github_over_documents() {
    let home = std::env::temp_dir().join(format!("sweeploom-roots-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join("Documents").join("GitHub")).unwrap();
    fs::create_dir_all(home.join("Desktop")).unwrap();
    let roots = super::developer_roots(&home);
    assert!(
        roots.iter().any(|item| item.ends_with("GitHub")),
        "expected Documents/GitHub, got {roots:?}"
    );
    assert!(roots.iter().any(|item| item.ends_with("Desktop")));
    assert!(
        roots
            .iter()
            .all(|item| item.file_name().and_then(|name| name.to_str()) != Some("Documents")),
        "Documents itself must not be walked when GitHub exists: {roots:?}"
    );
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn review_scan_roots_uses_developer_folders_for_home() {
    let home = std::env::temp_dir().join(format!("sweeploom-review-roots-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join("src")).unwrap();
    let roots = super::review_scan_roots(&home, &home);
    assert_eq!(roots, vec![home.join("src")]);
    let other = home.join("work");
    fs::create_dir_all(&other).unwrap();
    let scoped = super::review_scan_roots(&other, &home);
    assert_eq!(scoped, vec![other]);
    let _ = fs::remove_dir_all(&home);
}
