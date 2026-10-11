//! Parallel discovery and inventory: budgets, skips and determinism.

use std::fs;
use std::path::{Path, PathBuf};

use super::{InventoryLimits, discover_projects, scan_inventory};

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-parallel-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::canonicalize(&root).unwrap()
}

fn marker(dir: &Path, name: &str) {
    fs::create_dir_all(dir).unwrap();
    if name.contains(".xc") || name == ".git" {
        fs::create_dir_all(dir.join(name)).unwrap();
    } else {
        fs::write(dir.join(name), "").unwrap();
    }
}

#[test]
fn one_huge_tree_cannot_crowd_out_its_siblings() {
    let root = temp_root("fair");
    for index in 0..80 {
        marker(
            &root.join("a-mono").join(format!("pkg{index:02}")),
            "package.json",
        );
    }
    marker(&root.join("a-mono"), "package.json");
    marker(&root.join("b-ios"), "App.xcodeproj");
    marker(&root.join("c-swift"), "Package.swift");
    marker(&root.join("d-repo"), ".git");
    let found = discover_projects(&root, 10);
    fs::remove_dir_all(&root).ok();
    assert_eq!(found.len(), 10, "{found:?}");
    for name in ["a-mono", "b-ios", "c-swift", "d-repo"] {
        assert!(
            found.contains(&root.join(name)),
            "{name} missing: {found:?}"
        );
    }
}

#[test]
fn discovery_skips_hidden_generated_and_bundle_folders() {
    let root = temp_root("skips");
    let app = root.join("app");
    marker(&app, "Cargo.toml");
    marker(
        &app.join(".claude").join("worktrees").join("copy"),
        "Cargo.toml",
    );
    marker(
        &app.join("DerivedDataUI").join("SourcePackages").join("dep"),
        "Package.swift",
    );
    marker(
        &app.join(".build").join("checkouts").join("dep"),
        "Package.swift",
    );
    marker(&app.join("Pods").join("Lib"), "Podfile");
    marker(&app.join("App.xcodeproj").join("inner"), "Cargo.toml");
    marker(&app.join("ios"), "App.xcworkspace");
    let found = discover_projects(&root, 64);
    fs::remove_dir_all(&root).ok();
    assert_eq!(found, vec![app.clone(), app.join("ios")], "{found:?}");
}

fn build_tree(root: &Path) -> (u64, u64) {
    let mut files = 0;
    let mut bytes = 0;
    for top in 0..50 {
        for nested in 0..3 {
            let dir = root.join(format!("top{top:02}")).join(format!("n{nested}"));
            fs::create_dir_all(&dir).unwrap();
            let size = 10 + top * 3 + nested;
            fs::write(dir.join("file.rs"), vec![b'x'; size]).unwrap();
            files += 1;
            bytes += size as u64;
        }
    }
    marker(&root.join("top07"), "go.mod");
    fs::write(root.join("README.md"), b"root").unwrap();
    (files + 2, bytes + 4)
}

#[test]
fn parallel_inventory_matches_the_tree_and_is_deterministic() {
    let root = temp_root("inventory");
    let (files, bytes) = build_tree(&root);
    let first = scan_inventory(&root, InventoryLimits::gui()).unwrap();
    let second = scan_inventory(&root, InventoryLimits::gui()).unwrap();
    fs::remove_dir_all(&root).ok();
    assert_eq!(first.tree.files, files);
    assert_eq!(first.tree.logical_bytes, bytes);
    assert_eq!(first.tree.directories, 50);
    assert_eq!(first.tree.children.len(), 50);
    assert_eq!(first.entries, 1 + 50 + 150 + files);
    assert!(!first.tree.incomplete);
    assert_eq!(first.projects, vec![root.join("top07")]);
    assert!(first.folder_bytes(&root.join("top07")).is_some());
    assert_eq!(first.tree, second.tree);
    assert_eq!(first.projects, second.projects);
    let sizes: Vec<u64> = first
        .tree
        .children
        .iter()
        .map(|child| child.disk_bytes())
        .collect();
    assert!(sizes.windows(2).all(|pair| pair[0] >= pair[1]), "{sizes:?}");
    assert!(
        first
            .tree
            .children
            .iter()
            .all(|child| child.children.len() == 3)
    );
}

#[test]
fn streamed_preview_lists_top_level_folders_while_walking() {
    let root = temp_root("preview");
    build_tree(&root);
    let mut previews = Vec::new();
    let report = super::scan_inventory_with(&root, InventoryLimits::gui(), |tick| {
        previews.extend(tick.root);
    })
    .unwrap();
    fs::remove_dir_all(&root).ok();
    for preview in previews {
        assert!(
            preview
                .children
                .iter()
                .all(|child| child.children.is_empty())
        );
        assert!(preview.logical_bytes <= report.tree.logical_bytes);
    }
}
