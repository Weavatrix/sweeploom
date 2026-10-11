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

#[path = "app_build.rs"]
mod build;
#[path = "app_live.rs"]
mod live_state;

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
        let mut app = build::build(build::Boot {
            sampler,
            snapshot,
            sessions,
            saved,
            cached_projects,
            scan_history,
            locations,
            prefs,
            start_hidden,
        });
        live::stamp_first(&mut app);
        app.rebuild_review();
        // Startup Review refresh must not erase persisted folder measurements.
        app.project_sizes = cached_sizes;
        app
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
