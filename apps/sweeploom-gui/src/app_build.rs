//! Initial application state. Split from `app.rs` to keep the constructor short.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use sweeploom_core::{LiveSession, ObservationTracker};
use sweeploom_history::HistoryStore;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet, volume_space};

use super::SweepLoomApp;
use crate::live;
use crate::nav::Nav;
use crate::prefs::Prefs;
use crate::scan_history::{SavedScan, ScanHistory};
use crate::screens::{AiGroup, BrowserUi, ProjectGroup};
use crate::sort::Sort;

/// Everything loaded before the first frame.
pub(super) struct Boot {
    pub sampler: ProcessSampler,
    pub snapshot: ProcessSnapshotSet,
    pub sessions: Vec<LiveSession>,
    pub saved: Option<SavedScan>,
    pub cached_projects: Vec<std::path::PathBuf>,
    pub scan_history: ScanHistory,
    pub locations: UserLocations,
    pub prefs: Prefs,
    pub start_hidden: bool,
}

pub(super) fn build(b: Boot) -> SweepLoomApp {
    SweepLoomApp {
        nav: Nav::Overview,
        history_view: Default::default(),
        sampler: b.sampler,
        last_sample: Instant::now(),
        last_quiet: Instant::now(),
        snapshot: Some(b.snapshot),
        sessions: b.sessions,
        node_versions: Default::default(),
        selected_session: None,
        group_raw: false,
        session_search: String::new(),
        show_all_apps: false,
        session_sort: Sort::size_desc(),
        review_sort: Sort::size_desc(),
        project_sort: Sort::size_desc(),
        project_group: ProjectGroup::Parent,
        collapsed_project_groups: HashSet::new(),
        project_cards: Vec::new(),
        project_card_stamp: u64::MAX,
        project_sizes: Default::default(),
        ai_sort: Sort::size_desc(),
        ai_group: AiGroup::Tool,
        collapsed_ai_groups: HashSet::new(),
        expanded_history: HashSet::new(),
        expanded_sessions: HashSet::new(),
        explorer_sort: Sort::size_desc(),
        selected_explorer: HashSet::new(),
        selected_projects: HashSet::new(),
        disk_actions: Default::default(),
        native_cleanup: Default::default(),
        expanded_explorer: HashSet::new(),
        process_sort: Sort::size_desc(),
        history_sort: Sort::size_desc(),
        history: HistoryStore::default(),
        inventory: b.saved.as_ref().map(|saved| saved.report.clone()),
        inventory_error: None,
        inventory_at: b.saved.as_ref().map(|saved| saved.at),
        inventory_cached: b.saved.is_some(),
        disk_history_filter: String::new(),
        disk_history_selected: None,
        scan_root: b.saved.as_ref().map_or_else(
            || b.locations.home.display().to_string(),
            |saved| saved.report.root.display().to_string(),
        ),
        scan_history: b.scan_history,
        locations: b.locations,
        confirm_terminate: false,
        pending_stop: None,
        observation: ObservationTracker::default(),
        confirm_force: false,
        pending_force: None,
        action_message: None,
        review: Vec::new(),
        last_receipt: None,
        last_cleanup_summary: None,
        free_gb: "1".to_owned(),
        free_ram_gb: "2".to_owned(),
        reduce_cpu: "10".to_owned(),
        planned_keys: HashSet::new(),
        helper_keys: HashSet::new(),
        confirm_planned: false,
        confirm_helpers: false,
        browser: BrowserUi::default(),
        current_project: live::current_project(),
        project_roots: b.cached_projects,
        last_busy: HashMap::new(),
        volumes: volume_space(),
        scanning: false,
        scan_entries: 0,
        scan_bytes: 0,
        scan_hint: String::new(),
        ai_offers: None,
        ai_listing: false,
        prefs: b.prefs,
        hidden: false,
        hid_at: Instant::now(),
        tray: None,
        force_quit: false,
        start_hidden: b.start_hidden,
        scan_rx: None,
        rebuild_rx: None,
        review_after_scan: false,
        apply_rx: None,
        ai_rx: None,
    }
}
