use super::*;
use std::fs;

#[test]
fn cargo_light_is_preselected_node_is_not() {
    let root = std::env::temp_dir().join(format!("sweeploom-review-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("target").join("incremental")).unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();
    fs::write(root.join("package.json"), "{}\n").unwrap();
    fs::write(
        root.join("target").join("incremental").join("a"),
        vec![0_u8; 512],
    )
    .unwrap();
    fs::write(root.join("node_modules").join("pkg.js"), vec![0_u8; 512]).unwrap();
    let rows = collect_review(&[root.as_path()], &[]);
    let light = rows
        .iter()
        .find(|row| row.title.contains("Light"))
        .expect("light cargo row");
    assert!(light.selected);
    assert!(!light.candidate.safety.is_blocked());
    let node = rows
        .iter()
        .find(|row| row.title.contains("node_modules"))
        .expect("node row");
    assert!(!node.selected);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn vite_cache_is_preselected() {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-review-vite-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".vite")).unwrap();
    fs::write(root.join("package.json"), "{}\n").unwrap();
    fs::write(root.join(".vite").join("chunk"), vec![0_u8; 64]).unwrap();
    let rows = collect_review(&[root.as_path()], &[]);
    let vite = rows
        .iter()
        .find(|row| row.title.contains("Vite"))
        .expect("vite row");
    assert!(vite.selected);
    assert!(!vite.candidate.safety.is_blocked());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn member_review_sizes_the_workspace_target() {
    let root = std::env::temp_dir().join(format!(
        "sweeploom-review-ws-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|item| item.as_nanos())
            .unwrap_or(0)
    ));
    let member = root.join("api");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::create_dir_all(root.join("target").join("debug")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"api\"]\n",
    )
    .unwrap();
    fs::write(member.join("Cargo.toml"), "[package]\nname = \"api\"\n").unwrap();
    fs::write(
        root.join("target").join("debug").join("a.rlib"),
        vec![0_u8; 2048],
    )
    .unwrap();
    let rows = collect_review(&[member.as_path()], &[]);
    let full = rows
        .iter()
        .find(|row| row.title.contains("Full"))
        .expect("workspace target");
    assert_eq!(full.candidate.logical_bytes, 2048);
    match &full.candidate.owner {
        CandidateOwner::Project(id) => assert_eq!(id.0, root),
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&root);
}
