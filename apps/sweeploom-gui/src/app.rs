//! Application state. Screens live in `screens/`.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, RichText};

use crossbeam_channel::Receiver;
use sweeploom_core::{LiveSession, ProcessKey, Receipt, SessionId};
use sweeploom_dev::ReviewRow;
use sweeploom_history::HistoryStore;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet, volume_space};
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};
use sweeploom_storage::InventoryReport;

use crate::nav::Nav;
use crate::scan_job::{self, ScanOutcome};
use crate::screens;
use crate::sort::Sort;
use crate::theme;
use crate::widgets::placeholder;

/// Live UI application.
pub struct SweepLoomApp {
    pub(crate) nav: Nav,
    sampler: ProcessSampler,
    last_sample: Instant,
    pub(crate) snapshot: Option<ProcessSnapshotSet>,
    pub(crate) sessions: Vec<LiveSession>,
    pub(crate) selected_session: Option<SessionId>,
    pub(crate) group_raw: bool,
    pub(crate) session_sort: Sort,
    pub(crate) review_sort: Sort,
    pub(crate) explorer_sort: Sort,
    pub(crate) process_sort: Sort,
    pub(crate) history: HistoryStore,
    pub(crate) inventory: Option<InventoryReport>,
    pub(crate) inventory_error: Option<String>,
    pub(crate) scan_root: String,
    pub(crate) locations: UserLocations,
    pub(crate) confirm_terminate: bool,
    pub(crate) confirm_force: bool,
    pub(crate) pending_force: Option<Vec<ProcessKey>>,
    pub(crate) action_message: Option<String>,
    pub(crate) review: Vec<ReviewRow>,
    pub(crate) last_receipt: Option<Receipt>,
    pub(crate) free_gb: String,
    pub(crate) free_ram_gb: String,
    pub(crate) reduce_cpu: String,
    pub(crate) planned_keys: HashSet<ProcessKey>,
    pub(crate) confirm_planned: bool,
    pub(crate) volumes: Vec<(PathBuf, u64, u64)>,
    pub(crate) scanning: bool,
    scan_rx: Option<Receiver<ScanOutcome>>,
}

impl SweepLoomApp {
    /// Construct and take the first process sample.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let locations = UserLocations::current();
        let mut sampler = ProcessSampler::new();
        let (snapshot, sessions) = sample_with(&mut sampler, &locations);
        let mut app = Self {
            nav: Nav::Overview,
            sampler,
            last_sample: Instant::now(),
            snapshot: Some(snapshot),
            sessions,
            selected_session: None,
            group_raw: false,
            session_sort: Sort::size_desc(),
            review_sort: Sort::size_desc(),
            explorer_sort: Sort::size_desc(),
            process_sort: Sort::size_desc(),
            history: HistoryStore::default(),
            inventory: None,
            inventory_error: None,
            scan_root: locations.home.display().to_string(),
            locations,
            confirm_terminate: false,
            confirm_force: false,
            pending_force: None,
            action_message: None,
            review: Vec::new(),
            last_receipt: None,
            free_gb: "1".to_owned(),
            free_ram_gb: "2".to_owned(),
            reduce_cpu: "10".to_owned(),
            planned_keys: HashSet::new(),
            confirm_planned: false,
            volumes: volume_space(),
            scanning: false,
            scan_rx: None,
        };
        if let Some(snapshot) = &app.snapshot {
            app.history
                .record(&snapshot.processes, snapshot.captured_at);
        }
        app.rebuild_review();
        app
    }

    pub(crate) fn refresh_live(&mut self) {
        if self.last_sample.elapsed() < Duration::from_secs(1) {
            return;
        }
        let mut snapshot = self.sampler.refresh(Duration::ZERO);
        snapshot.resolve_parents();
        let _ = enrich_network(&mut snapshot.processes);
        self.history
            .record(&snapshot.processes, snapshot.captured_at);
        let roots = session_roots(self);
        self.sessions = sessions_from_snapshot(&mut snapshot, &roots);
        let live: HashSet<ProcessKey> = self
            .sessions
            .iter()
            .flat_map(|session| session.processes.iter().copied())
            .collect();
        self.planned_keys.retain(|key| live.contains(key));
        self.snapshot = Some(snapshot);
        self.last_sample = Instant::now();
    }

    pub(crate) fn run_scan(&mut self) {
        if self.scanning {
            return;
        }
        let root = PathBuf::from(self.scan_root.trim());
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        self.scanning = true;
        self.inventory_error = None;
        self.action_message = Some(format!("Scanning {}…", root.display()));
        self.scan_rx = Some(scan_job::spawn(root, processes, self.locations.clone()));
    }

    fn poll_scan(&mut self) {
        let Some(rx) = &self.scan_rx else {
            return;
        };
        let Ok(outcome) = rx.try_recv() else {
            return;
        };
        self.scan_rx = None;
        self.scanning = false;
        match outcome {
            Ok((report, rows)) => {
                let n = rows.len();
                self.inventory = Some(report);
                self.review = rows;
                self.inventory_error = None;
                self.action_message = Some(format!("{n} candidates"));
            }
            Err(error) => self.inventory_error = Some(error),
        }
    }
}

impl eframe::App for SweepLoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_scan();
        if self.scanning {
            ctx.request_repaint_after(Duration::from_millis(200));
        } else {
            self.refresh_live();
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        draw_chrome(ctx, self);
    }
}

fn sample_with(
    sampler: &mut ProcessSampler,
    locations: &UserLocations,
) -> (ProcessSnapshotSet, Vec<LiveSession>) {
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _ = enrich_network(&mut snapshot.processes);
    let sessions = sessions_from_snapshot(&mut snapshot, &home_only(locations));
    (snapshot, sessions)
}

fn home_only(locations: &UserLocations) -> AttributionRoots {
    AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    }
}

fn session_roots(app: &SweepLoomApp) -> AttributionRoots {
    let mut projects = vec![app.locations.home.clone()];
    if let Some(report) = &app.inventory {
        for project in &report.projects {
            if !projects.iter().any(|item| item == project) {
                projects.push(project.clone());
            }
        }
    }
    AttributionRoots {
        projects,
        current_project: None,
    }
}

fn draw_chrome(ctx: &egui::Context, app: &mut SweepLoomApp) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("SweepLoom").size(26.0).strong());
            ui.label(
                RichText::new("by Weavatrix")
                    .size(15.0)
                    .color(egui::Color32::from_rgb(168, 174, 186)),
            );
            ui.separator();
            ui.label(
                RichText::new("Reclaim your workstation without losing your workspace").size(16.0),
            );
        });
        ui.add_space(8.0);
    });
    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(196.0)
        .show(ctx, |ui| {
            ui.add_space(12.0);
            for nav in Nav::ALL {
                let label = RichText::new(nav.label()).size(17.0);
                if ui.selectable_label(app.nav == nav, label).clicked() {
                    app.nav = nav;
                }
                ui.add_space(2.0);
            }
        });
    egui::CentralPanel::default().show(ctx, |ui| draw_page(app, ui));
}

fn draw_page(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    match app.nav {
        Nav::Overview => screens::ui_overview(app, ui),
        Nav::Sessions => screens::ui_sessions(app, ui),
        Nav::Storage => screens::ui_review(app, ui),
        Nav::Explorer => screens::ui_storage(app, ui),
        Nav::Projects => screens::ui_projects(app, ui),
        Nav::Browser => placeholder(
            ui,
            "Browser",
            "Optional companion for lastAccessed / Discard / Bookmark+Close.",
        ),
        Nav::Ai => screens::ui_ai(app, ui),
        Nav::Rules => placeholder(
            ui,
            "Rules",
            "Declarative TOML cleaners. No shell from downloaded rules.",
        ),
        Nav::History => screens::ui_history(app, ui),
        Nav::Settings => screens::ui_settings(app, ui),
    }
}
