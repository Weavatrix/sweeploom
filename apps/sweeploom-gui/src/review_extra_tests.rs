use super::*;
use std::fs;

fn temp_home(name: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!(
        "sweeploom-roots-{name}-{}-{}",
        std::process::id(),
        crate::scan_history::now_ms()
    ));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    fs::canonicalize(&home).unwrap()
}

fn project(path: &Path, marker: &str) {
    fs::create_dir_all(path).unwrap();
    if marker.ends_with(".xcodeproj") {
        fs::create_dir_all(path.join(marker)).unwrap();
    } else {
        fs::write(path.join(marker), "").unwrap();
    }
}

fn locations(home: &Path) -> UserLocations {
    UserLocations {
        downloads: None,
        temp: home.join("tmp"),
        cache: None,
        app_config: home.join("cfg"),
        app_data: home.join("data"),
        home: home.to_path_buf(),
    }
}

#[cfg(unix)]
#[test]
fn launch_directory_and_home_are_not_discovery_roots() {
    let home = PathBuf::from("/Users/example");
    let developer = home.join("dev");
    let mut roots = vec![developer.clone()];
    prepend_unique(&mut roots, Path::new("/"), &home);
    prepend_unique(&mut roots, &home, &home);
    assert_eq!(roots, vec![developer]);
}

#[test]
fn saved_explorer_scan_of_one_repo_does_not_hide_other_developer_roots() {
    let home = temp_home("merge");
    let own = home.join("dev").join("sweeploom");
    project(&own, "Cargo.toml");
    project(&own.join("apps").join("gui"), "Cargo.toml");
    project(&home.join("dev").join("web"), "package.json");
    project(&home.join("dev").join("ios-app"), "App.xcodeproj");
    project(&home.join("dev").join("swift-kit"), "Package.swift");
    project(
        &home.join("Documents").join("GitHub").join("tool"),
        "go.mod",
    );
    project(&home.join("Projects").join("api"), "pom.xml");
    // Hidden tool homes found by an Explorer scan never become projects.
    project(
        &own.join(".cargo-home").join("registry").join("dep"),
        "Cargo.toml",
    );
    let inventory = vec![own.clone(), own.join(".cargo-home/registry/dep")];
    let found = project_roots(Some(&own), &locations(&home), &inventory, None);
    for expected in [
        own.clone(),
        own.join("apps").join("gui"),
        home.join("dev").join("web"),
        home.join("dev").join("ios-app"),
        home.join("dev").join("swift-kit"),
        home.join("Documents").join("GitHub").join("tool"),
        home.join("Projects").join("api"),
    ] {
        assert!(
            found.contains(&expected),
            "{expected:?} missing from {found:?}"
        );
    }
    assert!(
        found
            .iter()
            .all(|path| !path.to_string_lossy().contains(".cargo-home"))
    );
    let unique: std::collections::HashSet<_> = found.iter().collect();
    assert_eq!(unique.len(), found.len(), "duplicates in {found:?}");
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn inventory_projects_outside_developer_roots_are_merged() {
    let home = temp_home("external");
    project(&home.join("dev").join("one"), "Cargo.toml");
    let external = home.join("Volumes").join("ssd").join("work").join("two");
    project(&external, "package.json");
    let scan_root = home.join("Volumes").join("ssd");
    let found = project_roots(
        Some(&scan_root),
        &locations(&home),
        &[external.clone()],
        None,
    );
    assert!(found.contains(&home.join("dev").join("one")));
    assert!(found.contains(&external));
    let _ = fs::remove_dir_all(&home);
}
