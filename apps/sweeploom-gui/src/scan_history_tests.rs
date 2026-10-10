use super::*;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "sweeploom-history-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn value(path: &str, bytes: u64, complete: bool, basis: Basis) -> Vec<(PathBuf, DiskUsage, Basis)> {
    vec![(
        path.into(),
        DiskUsage {
            bytes,
            complete,
            ..Default::default()
        },
        basis,
    )]
}

#[test]
fn restart_restores_explorer_trees_project_sizes_and_growth() {
    let root = scratch("restart");
    let project = root.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("src/main.rs"), b"hello").unwrap();
    let path = root.join("data/history.json");
    let mut history = ScanHistory::load(path.clone());
    let report =
        sweeploom_storage::scan_inventory(&project, sweeploom_storage::InventoryLimits::gui())
            .unwrap();
    history.record_scan(&report, 1000);
    history.record_batch(
        Source::Projects,
        value(project.to_str().unwrap(), 100, true, Basis::Allocated),
        1000,
    );
    history.record_batch(
        Source::Projects,
        value(project.to_str().unwrap(), 350, true, Basis::Allocated),
        2000,
    );
    history.remember_projects(std::slice::from_ref(&project));
    history.flush();
    assert!(history.error.is_none());
    let loaded = ScanHistory::load(path);
    assert!(loaded.error.is_none());
    assert_eq!(loaded.scans()[0].report, report);
    assert_eq!(loaded.scans()[0].at, 1000);
    assert_eq!(loaded.projects(), &[project.clone()]);
    assert_eq!(
        loaded
            .get(Source::Projects, &project)
            .unwrap()
            .delta()
            .unwrap()
            .0,
        250
    );
    let cached = crate::project_sizes::ProjectSizes::restore(&loaded);
    assert_eq!(cached.get(&project).unwrap().as_ref().unwrap().bytes, 350);
    assert!(cached.is_cached(&project));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn partial_scans_never_report_shrinkage_and_do_not_replace_the_baseline() {
    let root = scratch("partial");
    let mut history = ScanHistory::load(root.join("history.json"));
    history.record_batch(
        Source::Explorer,
        value("/folder", 1000, true, Basis::Allocated),
        1000,
    );
    history.record_batch(
        Source::Explorer,
        value("/folder", 0, false, Basis::Allocated),
        2000,
    );
    assert!(
        history
            .get(Source::Explorer, Path::new("/folder"))
            .unwrap()
            .delta()
            .is_none()
    );
    history.record_batch(
        Source::Explorer,
        value("/folder", 700, true, Basis::Allocated),
        3000,
    );
    let series = history.get(Source::Explorer, Path::new("/folder")).unwrap();
    assert_eq!(series.delta().unwrap().0, -300);
    assert_eq!(series.delta().unwrap().1.at, 1000);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sources_and_size_bases_cannot_be_mixed() {
    let root = scratch("basis");
    let mut history = ScanHistory::load(root.join("history.json"));
    history.record_batch(
        Source::Projects,
        value("/folder", 1000, true, Basis::Allocated),
        1000,
    );
    history.record_batch(
        Source::Review,
        value("/folder", 2, true, Basis::Logical),
        2000,
    );
    assert!(
        history
            .get(Source::Review, Path::new("/folder"))
            .unwrap()
            .delta()
            .is_none()
    );
    history.record_batch(
        Source::Projects,
        value("/folder", 1, true, Basis::Logical),
        3000,
    );
    assert!(
        history
            .get(Source::Projects, Path::new("/folder"))
            .unwrap()
            .delta()
            .is_none()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_from_a_later_tree_is_not_invented_as_deleted() {
    let root = scratch("missing");
    let mut history = ScanHistory::load(root.join("history.json"));
    history.record_batch(
        Source::Explorer,
        value("/folder/child", 100, true, Basis::Allocated),
        1000,
    );
    history.record_batch(
        Source::Explorer,
        value("/other-folder", 200, true, Basis::Allocated),
        2000,
    );
    assert_eq!(
        history
            .get(Source::Explorer, Path::new("/folder/child"))
            .unwrap()
            .points
            .len(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retention_is_bounded_and_duplicate_offers_are_one_sample() {
    let root = scratch("retention");
    let mut history = ScanHistory::load(root.join("history.json"));
    for at in 1..=40 {
        history.record_batch(
            Source::Projects,
            value("/p", at, true, Basis::Allocated),
            at,
        );
    }
    let series = history.get(Source::Projects, Path::new("/p")).unwrap();
    assert_eq!(series.points.len(), MAX_POINTS);
    assert_eq!(series.points[0].at, 9);
    let mut values = value("/p", 10, true, Basis::Allocated);
    values.extend(value("/p", 20, true, Basis::Allocated));
    history.record_batch(Source::Projects, values, 41);
    assert_eq!(
        history
            .get(Source::Projects, Path::new("/p"))
            .unwrap()
            .latest()
            .usage
            .bytes,
        20
    );
    assert_eq!(
        history
            .get(Source::Projects, Path::new("/p"))
            .unwrap()
            .delta()
            .unwrap()
            .0,
        -20
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupt_archive_is_reported_and_preserved() {
    let root = scratch("corrupt");
    let path = root.join("history.json");
    fs::write(&path, b"not valid JSON").unwrap();
    let mut history = ScanHistory::load(path.clone());
    assert!(history.error.is_some());
    history.record_batch(
        Source::Projects,
        value("/p", 10, true, Basis::Allocated),
        1000,
    );
    history.flush();
    assert_eq!(fs::read(&path).unwrap(), b"not valid JSON");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn normal_quit_finishes_a_background_write_and_saves_newer_results() {
    let root = scratch("writer");
    let path = root.join("history.json");
    let mut history = ScanHistory::load(path.clone());
    history.record_batch(
        Source::Projects,
        value("/p", 10, true, Basis::Allocated),
        1000,
    );
    history.last_write = Instant::now() - Duration::from_secs(3);
    history.poll_save();
    assert!(history.writer.is_some());
    history.record_batch(
        Source::Projects,
        value("/p", 20, true, Basis::Allocated),
        2000,
    );
    history.flush();
    let loaded = ScanHistory::load(path);
    assert_eq!(
        loaded
            .get(Source::Projects, Path::new("/p"))
            .unwrap()
            .latest()
            .usage
            .bytes,
        20
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dates_and_signed_sizes_do_not_round_small_changes_to_zero() {
    assert_eq!(timestamp(0), "1970-01-01 00:00:00 UTC");
    assert_eq!(timestamp(951_782_400_000), "2000-02-29 00:00:00 UTC");
    assert_eq!(signed_bytes(0), "Unchanged");
    assert_eq!(signed_bytes(1), "+1 B");
    assert_eq!(signed_bytes(-1), "-1 B");
    assert!(signed_bytes(4096).starts_with('+'));
    assert!(signed_bytes(-4096).starts_with('-'));
}
