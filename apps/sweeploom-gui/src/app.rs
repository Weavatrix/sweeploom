//! Application state. Screens live in `screens/`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, RichText};

use sweeploom_core::{LiveSession, Receipt};
use sweeploom_dev::ReviewRow;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet};
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};
use sweeploom_storage::{InventoryLimits, InventoryReport, scan_inventory};

use crate::nav::Nav;
use crate::screens;
use crate::widgets::placeholder;

/// Live UI application.
pub struct SweepLoomApp {
    pub(crate) nav: Nav,
    sampler: ProcessSampler,
    last_sample: Instant,
    pub(crate) snapshot: Option<ProcessSnapshotSet>,
    pub(crate) sessions: Vec<LiveSession>,
    pub(crate) selected_session: Option<usize>,
    pub(crate) group_raw: bool,
    pub(crate) inventory: Option<InventoryReport>,
    pub(crate) inventory_error: Option<String>,
    pub(crate) scan_root: String,
    pub(crate) locations: UserLocations,
    pub(crate) confirm_terminate: bool,
    pub(crate) action_message: Option<String>,
    pub(crate) review: Vec<ReviewRow>,
    pub(crate) last_receipt: Option<Receipt>,
}

impl SweepLoomApp {
    /// Construct and take the first process sample.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(style);
        let locations = UserLocations::current();
        let mut sampler = ProcessSampler::new();
        let (snapshot, sessions) = sample_with(&mut sampler, &locations);
        Self {
            nav: Nav::Overview,
            sampler,
            last_sample: Instant::now(),
            snapshot: Some(snapshot),
            sessions,
            selected_session: None,
            group_raw: false,
            inventory: None,
            inventory_error: None,
            scan_root: locations.home.display().to_string(),
            locations,
            confirm_terminate: false,
            action_message: None,
            review: Vec::new(),
            last_receipt: None,
        }
    }

    pub(crate) fn refresh_live(&mut self) {
        if self.last_sample.elapsed() < Duration::from_secs(1) {
            return;
        }
        let mut snapshot = self.sampler.refresh(Duration::ZERO);
        snapshot.resolve_parents();
        let _ = enrich_network(&mut snapshot.processes);
        self.sessions = sessions_from_snapshot(&mut snapshot, &home_roots(&self.locations));
        self.snapshot = Some(snapshot);
        self.last_sample = Instant::now();
    }

    pub(crate) fn run_scan(&mut self) {
        let root = PathBuf::from(self.scan_root.trim());
        match scan_inventory(&root, InventoryLimits::default()) {
            Ok(report) => {
                self.inventory = Some(report);
                self.inventory_error = None;
                self.rebuild_review();
            }
            Err(error) => self.inventory_error = Some(error.to_string()),
        }
    }
}

impl eframe::App for SweepLoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_live();
        ctx.request_repaint_after(Duration::from_secs(1));
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
    let sessions = sessions_from_snapshot(&mut snapshot, &home_roots(locations));
    (snapshot, sessions)
}

fn home_roots(locations: &UserLocations) -> AttributionRoots {
    AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    }
}

fn draw_chrome(ctx: &egui::Context, app: &mut SweepLoomApp) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("SweepLoom").strong());
            ui.label(RichText::new("by Weavatrix").weak());
            ui.separator();
            ui.label("Reclaim your workstation without losing your workspace");
        });
        ui.add_space(4.0);
    });
    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(168.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            for nav in Nav::ALL {
                if ui.selectable_label(app.nav == nav, nav.label()).clicked() {
                    app.nav = nav;
                }
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
        Nav::Ai => placeholder(
            ui,
            "AI",
            "Inspect-first Claude/Codex storage. Search-before-delete is later.",
        ),
        Nav::Rules => placeholder(
            ui,
            "Rules",
            "Declarative TOML cleaners. No shell from downloaded rules.",
        ),
        Nav::History => placeholder(
            ui,
            "History",
            "Observed history only — SweepLoom does not invent activity from before it started.",
        ),
        Nav::Settings => screens::ui_settings(app, ui),
    }
}
