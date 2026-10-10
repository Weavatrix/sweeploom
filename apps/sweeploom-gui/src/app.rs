//! Application state. Screens live in `screens/`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, ViewportCommand};

use crossbeam_channel::Receiver;
use sweeploom_ai::AiOffer;
use sweeploom_core::{
    LiveSession, ObservationTracker, PendingTermination, ProcessKey, ProjectId, Receipt, SessionId,
};
use sweeploom_dev::ReviewRow;
use sweeploom_history::HistoryStore;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet, volume_space};
use sweeploom_storage::InventoryReport;

use crate::chrome;
use crate::live;
use crate::nav::Nav;
use crate::prefs::Prefs;
use crate::scan_job::{RebuildMsg, ScanMsg};
use crate::screens::{AiGroup, BrowserUi, ProjectCard, ProjectGroup};
use crate::sort::Sort;
use crate::theme;
use crate::tray::{self, TrayIconHandle};

/// Live UI application.
pub struct SweepLoomApp {
    pub(crate) nav: Nav,
    pub(crate) sampler: ProcessSampler,
    pub(crate) last_sample: Instant,
    pub(crate) last_quiet: Instant,
    pub(crate) snapshot: Option<ProcessSnapshotSet>,
    pub(crate) sessions: Vec<LiveSession>,
    pub(crate) node_versions: crate::node_versions::NodeVersions,
    pub(crate) selected_session: Option<SessionId>,
    pub(crate) group_raw: bool,
    pub(crate) session_search: String,
    pub(crate) show_all_apps: bool,
    pub(crate) session_sort: Sort,
    pub(crate) review_sort: Sort,
    pub(crate) project_sort: Sort,
    pub(crate) project_group: ProjectGroup,
    pub(crate) collapsed_project_groups: HashSet<String>,
    pub(crate) project_cards: Vec<ProjectCard>,
    pub(crate) project_card_stamp: u64,
    pub(crate) project_sizes: crate::project_sizes::ProjectSizes,
    pub(crate) ai_sort: Sort,
    pub(crate) ai_group: AiGroup,
    pub(crate) collapsed_ai_groups: HashSet<String>,
    pub(crate) expanded_history: HashSet<String>,
    pub(crate) expanded_sessions: HashSet<String>,
    pub(crate) explorer_sort: Sort,
    pub(crate) selected_explorer: HashSet<PathBuf>,
    pub(crate) selected_projects: HashSet<PathBuf>,
    pub(crate) native_cleanup: crate::native_cleanup::NativeCleanup,
    pub(crate) disk_actions: crate::disk_actions::DiskActions,
    pub(crate) expanded_explorer: HashSet<String>,
    pub(crate) process_sort: Sort,
    pub(crate) history_sort: Sort,
    pub(crate) history: HistoryStore,
    pub(crate) inventory: Option<InventoryReport>,
    pub(crate) inventory_error: Option<String>,
    pub(crate) scan_history: crate::scan_history::ScanHistory,
    pub(crate) inventory_at: Option<u64>,
    pub(crate) inventory_cached: bool,
    pub(crate) disk_history_filter: String,
    pub(crate) disk_history_selected: Option<(crate::scan_history::Source, PathBuf)>,
    pub(crate) scan_root: String,
    pub(crate) locations: UserLocations,
    pub(crate) confirm_terminate: bool,
    pub(crate) pending_stop: Option<PendingTermination>,
    pub(crate) observation: ObservationTracker,
    pub(crate) confirm_force: bool,
    pub(crate) pending_force: Option<Vec<ProcessKey>>,
    pub(crate) action_message: Option<String>,
    pub(crate) review: Vec<ReviewRow>,
    pub(crate) last_receipt: Option<Receipt>,
    pub(crate) last_cleanup_summary: Option<String>,
    pub(crate) free_gb: String,
    pub(crate) free_ram_gb: String,
    pub(crate) reduce_cpu: String,
    pub(crate) planned_keys: HashSet<ProcessKey>,
    pub(crate) helper_keys: HashSet<ProcessKey>,
    pub(crate) confirm_planned: bool,
    pub(crate) confirm_helpers: bool,
    pub(crate) browser: BrowserUi,
    pub(crate) current_project: Option<ProjectId>,
    pub(crate) project_roots: Vec<PathBuf>,
    pub(crate) last_busy: HashMap<ProcessKey, SystemTime>,
    pub(crate) volumes: Vec<(PathBuf, u64, u64)>,
    pub(crate) scanning: bool,
    pub(crate) scan_entries: u64,
    pub(crate) scan_bytes: u64,
    pub(crate) scan_hint: String,
    pub(crate) ai_offers: Option<Vec<AiOffer>>,
    pub(crate) ai_listing: bool,
    pub(crate) prefs: Prefs,
    pub(crate) hidden: bool,
    pub(crate) hid_at: Instant,
    pub(crate) tray: Option<TrayIconHandle>,
    pub(crate) force_quit: bool,
    start_hidden: bool,
    pub(crate) scan_rx: Option<Receiver<ScanMsg>>,
    pub(crate) rebuild_rx: Option<Receiver<RebuildMsg>>,
    pub(crate) review_after_scan: bool,
    pub(crate) apply_rx: Option<Receiver<(String, Receipt)>>,
    pub(crate) ai_rx: Option<Receiver<Result<Vec<AiOffer>, String>>>,
}

impl SweepLoomApp {
    /// Construct and take the first process sample.
    pub fn new(cc: &eframe::CreationContext<'_>, start_hidden: bool) -> Self {
        let locations = UserLocations::current();
        let prefs = Prefs::load(&locations.app_config.join("prefs.json"));
        let scan_history =
            crate::scan_history::ScanHistory::load(locations.app_data.join("scan-history.json"));
        let saved = scan_history.scans().last().cloned();
        let cached_projects = scan_history.projects().to_vec();
        let cached_sizes = crate::project_sizes::ProjectSizes::restore(&scan_history);
        theme::apply(&cc.egui_ctx, prefs.theme, prefs.ui_scale);
        tray::install_wake(cc.egui_ctx.clone());
        let mut sampler = ProcessSampler::new();
        let (snapshot, sessions) = live::sample_with(&mut sampler, &locations);
        let mut app = Self {
            nav: Nav::Overview,
            sampler,
            last_sample: Instant::now(),
            last_quiet: Instant::now(),
            snapshot: Some(snapshot),
            sessions,
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
            inventory: saved.as_ref().map(|saved| saved.report.clone()),
            inventory_error: None,
            inventory_at: saved.as_ref().map(|saved| saved.at),
            inventory_cached: saved.is_some(),
            disk_history_filter: String::new(),
            disk_history_selected: None,
            scan_root: saved.as_ref().map_or_else(
                || locations.home.display().to_string(),
                |saved| saved.report.root.display().to_string(),
            ),
            scan_history,
            locations,
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
            project_roots: cached_projects,
            last_busy: HashMap::new(),
            volumes: volume_space(),
            scanning: false,
            scan_entries: 0,
            scan_bytes: 0,
            scan_hint: String::new(),
            ai_offers: None,
            ai_listing: false,
            prefs,
            hidden: false,
            hid_at: Instant::now(),
            tray: None,
            force_quit: false,
            start_hidden,
            scan_rx: None,
            rebuild_rx: None,
            review_after_scan: false,
            apply_rx: None,
            ai_rx: None,
        };
        live::stamp_first(&mut app);
        app.rebuild_review();
        // Startup Review refresh must not erase persisted folder measurements.
        app.project_sizes = cached_sizes;
        app
    }

    pub(crate) fn observation_gap(&self) -> bool {
        self.observation.has_gap()
    }

    pub(crate) fn persist_prefs(&self) {
        self.prefs
            .save(&self.locations.app_config.join("prefs.json"));
    }

    pub(crate) fn sync_tray(&mut self) {
        if self.prefs.tray_enabled && tray::is_supported() {
            if self.tray.is_none() {
                self.tray = tray::create();
            }
        } else {
            self.tray = None;
        }
    }

    pub(crate) fn leave_background(&mut self, ctx: &egui::Context) {
        self.hidden = false;
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(1360.0, 860.0)));
        // Visible(false) stops Windows redraws, so restore both flags.
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
        self.refresh_now();
    }

    pub(crate) fn enter_background(&mut self, ctx: &egui::Context) {
        self.hidden = true;
        self.hid_at = Instant::now();
        self.history = HistoryStore::default();
        // Disk results (AI stores, Review, Projects, Explorer) survive the tray;
        // only live process state is dropped.
        self.snapshot = None;
        self.sessions.clear();
        self.planned_keys.clear();
        self.selected_session = None;
        self.pending_stop = None;
        self.observation.mark_gap();
        self.sampler.enter_quiet();
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        // Keep the window visible to winit so tray clicks still wake a frame.
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        ctx.request_repaint();
    }

    pub(crate) fn refresh_live(&mut self) {
        if self.last_sample.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.refresh_now();
    }

    fn refresh_now(&mut self) {
        let mut snapshot = self.sampler.refresh(Duration::ZERO);
        snapshot.resolve_parents();
        let _ = enrich_network(&mut snapshot.processes);
        self.history
            .record(&snapshot.processes, snapshot.captured_at);
        let roots = live::session_roots(self);
        live::rescore(
            &mut self.sessions,
            &mut self.last_busy,
            &mut snapshot,
            self.current_project.as_ref(),
            &roots,
            &mut self.observation,
        );
        let live_keys: HashSet<ProcessKey> = self
            .sessions
            .iter()
            .flat_map(|session| session.processes.iter().copied())
            .collect();
        self.planned_keys.retain(|key| live_keys.contains(key));
        self.helper_keys.retain(|key| live_keys.contains(key));
        let live_sessions: HashSet<SessionId> = self.sessions.iter().map(|item| item.id).collect();
        self.browser
            .tree_ids
            .retain(|id| live_sessions.contains(id));
        self.snapshot = Some(snapshot);
        self.last_sample = Instant::now();
    }
}

impl eframe::App for SweepLoomApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.poll_native();
        self.project_sizes
            .poll(&[], &egui::Context::default(), &mut self.scan_history);
        self.scan_history.flush();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.scan_history.poll_save();
        self.ensure_tray();
        if self.start_hidden {
            self.start_hidden = false;
            self.enter_background(ctx);
        }
        self.handle_tray(ctx);
        if self.stay_background(ctx) {
            self.poll_native();
            self.poll_disk();
            return;
        }
        theme::apply(ctx, self.prefs.theme, self.prefs.ui_scale);
        if self.scanning || self.ai_listing || self.scan_rx.is_some() || self.apply_rx.is_some() {
            self.refresh_live();
            ctx.request_repaint_after(Duration::from_millis(200));
        } else {
            self.refresh_live();
            ctx.request_repaint_after(if self.tray.is_some() {
                Duration::from_millis(200)
            } else {
                Duration::from_secs(1)
            });
        }
        chrome::draw(ctx, self);
        self.disk_dialog(ctx);
        self.native_dialog(ctx);
        self.poll_native();
        self.poll_trash();
        self.poll_disk();
        self.project_sizes
            .poll(&self.project_roots, ctx, &mut self.scan_history);
    }
}
