use super::*;
use crate::sort::Col;
use crossbeam_channel::Sender;

fn item(id: &str, bytes: Option<u64>) -> Item {
    Item {
        id: id.into(),
        name: id.into(),
        kind: Kind::Cache,
        bytes,
        usage: None,
        status: String::new(),
        scope: None,
        path: Some(id.into()),
        selected: false,
        enabled: false,
    }
}

fn worker(cleanup: &mut NativeCleanup, pane: Pane) -> Sender<ListingMsg> {
    let (tx, rx) = crossbeam_channel::unbounded();
    cleanup.panes.get_mut(&pane).unwrap().start(rx);
    tx
}

fn initial(tx: &Sender<ListingMsg>, id: &str, bytes: Option<u64>) {
    tx.send(ListingMsg::Initial(Listing {
        items: vec![item(id, bytes)],
        ..Default::default()
    }))
    .unwrap();
}

fn history(name: &str) -> ScanHistory {
    ScanHistory::load(std::env::temp_dir().join(format!(
        "sweeploom-pane-{name}-{}-{}.json",
        std::process::id(),
        now_ms()
    )))
}

#[test]
fn navigation_keeps_partial_scan_and_finishes_hidden_pane_without_restarting() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("navigation");
    cleanup.pane = Pane::Caches;
    let tx = worker(&mut cleanup, Pane::Caches);
    initial(&tx, "/cache", None);
    cleanup.poll_listings(&mut history);
    let state = cleanup.active_mut();
    state.listing.as_mut().unwrap().items[0].selected = true;
    state.filter = "cache".into();
    state.sort = Sort {
        col: Col::Name,
        desc: false,
    };
    for pane in Pane::ALL {
        cleanup.pane = pane;
    }
    tx.send(ListingMsg::Measured(item("/cache", Some(512))))
        .unwrap();
    cleanup.poll_listings(&mut history);
    let state = &cleanup.panes[&Pane::Caches];
    assert!(state.rx.is_some());
    assert!(!state.listing.as_ref().unwrap().complete);
    assert!(history.series().is_empty());
    tx.send(ListingMsg::Done).unwrap();
    drop(tx);
    cleanup.poll_listings(&mut history);
    cleanup.poll_listings(&mut history);
    cleanup.pane = Pane::Caches;
    let state = cleanup.active();
    assert!(state.rx.is_none());
    assert!(!state.needs_scan());
    assert_eq!(state.filter, "cache");
    assert_eq!(state.sort.col, Col::Name);
    assert!(!state.sort.desc);
    let listing = state.listing.as_ref().unwrap();
    assert!(listing.complete);
    assert_eq!(listing.items[0].bytes, Some(512));
    assert!(listing.items[0].selected);
    let series = history
        .get(Source::Native, &PathBuf::from("/cache"))
        .unwrap();
    assert_eq!(series.points.len(), 1);
    assert_eq!(series.latest().usage.bytes, 512);
}

#[test]
fn simultaneous_workers_with_identical_ids_update_only_their_own_panes() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("routing");
    let caches = worker(&mut cleanup, Pane::Caches);
    let apps = worker(&mut cleanup, Pane::Apps);
    initial(&caches, "/shared", None);
    initial(&apps, "/shared", None);
    cleanup.poll_listings(&mut history);
    cleanup.pane = Pane::Apps;
    cleanup.active_mut().listing.as_mut().unwrap().items[0].selected = true;
    caches
        .send(ListingMsg::Measured(item("/shared", Some(100))))
        .unwrap();
    caches.send(ListingMsg::Done).unwrap();
    apps.send(ListingMsg::Measured(item("/shared", Some(200))))
        .unwrap();
    cleanup.poll_listings(&mut history);
    let cache_state = &cleanup.panes[&Pane::Caches];
    let app_state = cleanup.active();
    assert!(cache_state.rx.is_none());
    assert!(app_state.rx.is_some());
    let cache_item = &cache_state.listing.as_ref().unwrap().items[0];
    let app_item = &app_state.listing.as_ref().unwrap().items[0];
    assert_eq!(cache_item.bytes, Some(100));
    assert!(!cache_item.selected);
    assert_eq!(app_item.bytes, Some(200));
    assert!(app_item.selected);
}

#[test]
fn refresh_keeps_old_results_and_view_state_then_reconciles_selection_by_id() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("refresh");
    let tx = worker(&mut cleanup, Pane::Docker);
    initial(&tx, "/retained", Some(10));
    tx.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    let state = cleanup.active_mut();
    state.filter = "retained".into();
    state.sort = Sort {
        col: Col::Status,
        desc: false,
    };
    state.listing.as_mut().unwrap().items[0].selected = true;
    let other = worker(&mut cleanup, Pane::Ios);
    let refresh = worker(&mut cleanup, Pane::Docker);
    assert_eq!(
        cleanup.active().listing.as_ref().unwrap().items[0].bytes,
        Some(10)
    );
    initial(&refresh, "/retained", None);
    cleanup.poll_listings(&mut history);
    refresh
        .send(ListingMsg::Measured(item("/retained", Some(20))))
        .unwrap();
    refresh.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    let state = cleanup.active();
    let current = &state.listing.as_ref().unwrap().items[0];
    assert!(current.selected);
    assert_eq!(current.bytes, Some(20));
    assert_eq!(state.filter, "retained");
    assert_eq!(state.sort.col, Col::Status);
    assert!(cleanup.panes[&Pane::Ios].rx.is_some());
    initial(&other, "/other", Some(30));
    other.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    assert_eq!(
        cleanup.active().listing.as_ref().unwrap().items[0].bytes,
        Some(20)
    );
}

#[test]
fn cleanup_invalidation_retains_results_and_does_not_interrupt_other_scans() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("invalidate");
    let cache = worker(&mut cleanup, Pane::Caches);
    let docker = worker(&mut cleanup, Pane::Docker);
    initial(&cache, "/cache", Some(10));
    initial(&docker, "/docker", Some(20));
    docker.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    cleanup.pane = Pane::Caches;
    cleanup.active_mut().filter = "cache".into();
    cleanup.invalidate_files();
    assert!(cleanup.active().rx.is_some());
    assert!(!cleanup.active().needs_scan());
    assert!(!cleanup.panes[&Pane::Docker].needs_scan());
    cache.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    assert!(cleanup.active().needs_scan());
    assert_eq!(cleanup.active().filter, "cache");
    assert_eq!(
        cleanup.active().listing.as_ref().unwrap().items[0].bytes,
        Some(10)
    );
    cleanup.invalidate(Pane::Docker);
    assert!(cleanup.panes[&Pane::Docker].needs_scan());
    assert_eq!(
        cleanup.panes[&Pane::Docker].listing.as_ref().unwrap().items[0].bytes,
        Some(20)
    );
}

#[test]
fn interrupted_hidden_worker_keeps_partial_rows_and_does_not_auto_retry_or_archive() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("interrupted");
    let tx = worker(&mut cleanup, Pane::Apps);
    initial(&tx, "/cache", Some(5));
    drop(tx);
    cleanup.poll_listings(&mut history);
    let state = &cleanup.panes[&Pane::Apps];
    assert!(state.rx.is_none());
    assert!(!state.needs_scan());
    let listing = state.listing.as_ref().unwrap();
    assert!(!listing.complete);
    assert_eq!(listing.items[0].bytes, Some(5));
    assert!(listing.note.contains("interrupted"));
    assert!(history.series().is_empty());
    assert!(cleanup.active().listing.is_none());
}

#[test]
fn rescan_keeps_previous_sizes_on_screen_but_records_only_fresh_ones() {
    let mut cleanup = NativeCleanup::default();
    let mut history = history("carry");
    let first = worker(&mut cleanup, Pane::Caches);
    initial(&first, "/cache", None);
    first
        .send(ListingMsg::Measured(item("/cache", Some(700))))
        .unwrap();
    first.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    cleanup.pane = Pane::Caches;
    cleanup.active_mut().listing.as_mut().unwrap().items[0].selected = true;
    cleanup.invalidate(Pane::Caches);
    let second = worker(&mut cleanup, Pane::Caches);
    initial(&second, "/cache", None);
    // A verdict-only update must not blank the carried size either.
    let mut verdict = item("/cache", None);
    verdict.status = "Project still exists".into();
    second.send(ListingMsg::Measured(verdict)).unwrap();
    cleanup.poll_listings(&mut history);
    let shown = &cleanup.active().listing.as_ref().unwrap().items[0];
    assert_eq!(
        shown.bytes,
        Some(700),
        "previous size stays until re-measured"
    );
    assert!(shown.selected);
    assert_eq!(shown.status, "Project still exists");
    std::thread::sleep(std::time::Duration::from_millis(5));
    second.send(ListingMsg::Done).unwrap();
    cleanup.poll_listings(&mut history);
    let series = history
        .get(Source::Native, &PathBuf::from("/cache"))
        .unwrap();
    assert_eq!(
        series.points.len(),
        1,
        "carried sizes are not new measurements"
    );
}
