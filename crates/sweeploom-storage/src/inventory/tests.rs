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
    assert!(
        report
            .project_bytes
            .iter()
            .any(|(path, bytes)| paths_match(path, &root) && *bytes >= 4096),
        "project bytes missing: {:?}",
        report.project_bytes
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
fn cap_keeps_visited_bytes() {
    let root = std::env::temp_dir().join(format!("sweeploom-cap-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let deep = root.join("a").join("b").join("c");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("big.bin"), vec![0_u8; 2048]).unwrap();
    let report = scan_inventory(
        &root,
        InventoryLimits {
            max_entries: Some(1),
            max_children_per_dir: 64,
            max_projects: 8,
            large_file_bytes: 32 * 1024 * 1024,
        },
    )
    .expect("scan");
    assert!(report.capped);
    assert!(report.tree.incomplete);
    assert!(
        report.tree.logical_bytes >= 2048,
        "visited bytes must survive a cap, got {}",
        report.tree.logical_bytes
    );
    let _ = fs::remove_dir_all(&root);
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

#[test]
fn gui_scan_has_no_entry_cap() {
    assert!(super::InventoryLimits::gui().max_entries.is_none());
}

#[test]
fn scan_preview_keeps_immediate_children_only() {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-preview-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(root.join("alpha").join("nested")).unwrap();
    fs::write(root.join("alpha").join("nested").join("a.bin"), [0_u8; 8]).unwrap();
    let report = super::scan_inventory(&root, super::InventoryLimits::gui()).expect("scan");
    let preview = report.tree.preview();
    fs::remove_dir_all(&root).ok();
    assert!(!preview.children.is_empty());
    assert!(
        preview
            .children
            .iter()
            .all(|child| child.children.is_empty())
    );
}

fn child<'a>(node: &'a super::DirectoryNode, name: &str) -> &'a super::DirectoryNode {
    node.children
        .iter()
        .find(|item| {
            item.path
                .file_name()
                .and_then(|file| file.to_str())
                .is_some_and(|file| file.eq_ignore_ascii_case(name))
        })
        .unwrap_or_else(|| panic!("{name} missing under {}", node.path.display()))
}

#[test]
fn bulky_home_trees_do_not_keep_nested_inspector_rows() {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-flat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ));
    let docker = root
        .join("AppData")
        .join("Local")
        .join("Docker")
        .join("overlay2")
        .join("layer");
    let pkg = root
        .join("AppData")
        .join("Local")
        .join("Packages")
        .join("Claude_x")
        .join("LocalState");
    fs::create_dir_all(&docker).unwrap();
    fs::create_dir_all(&pkg).unwrap();
    fs::write(docker.join("blob"), [0_u8; 32]).unwrap();
    fs::write(pkg.join("state"), [0_u8; 16]).unwrap();
    let report = super::scan_inventory(&root, super::InventoryLimits::gui()).expect("scan");
    let local = child(child(&report.tree, "AppData"), "Local");
    assert!(child(local, "Docker").children.is_empty());
    assert!(
        child(child(local, "Packages"), "Claude_x")
            .children
            .is_empty()
    );
    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn sparse_file_reports_allocated_blocks_separately() {
    let root = std::env::temp_dir().join(format!("sweeploom-sparse-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("Docker.raw");
    let file = fs::File::create(&path).unwrap();
    file.set_len(1024 * 1024 * 1024).unwrap();
    let report = scan_inventory(&root, InventoryLimits::default()).unwrap();
    assert_eq!(report.tree.logical_bytes, 1024 * 1024 * 1024);
    assert!(report.tree.disk_bytes() < 1024 * 1024);
    assert!(
        report
            .tree
            .children
            .iter()
            .any(|child| child.logical_bytes == 1024 * 1024 * 1024
                && child.disk_bytes() < 1024 * 1024)
    );
    fs::remove_dir_all(root).unwrap();
}
