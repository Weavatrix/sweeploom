//! Background/foreground transitions, live sampling and deep-link routing.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use eframe::egui::{self, ViewportCommand};
use sweeploom_core::{ProcessKey, SessionId};
use sweeploom_history::HistoryStore;
use sweeploom_network::enrich_network;

use super::SweepLoomApp;
use crate::live;
use crate::nav::{Dest, Nav};
use crate::tray;

impl SweepLoomApp {
    /// Open a screen, landing on the sub-view a deep link asks for.
    pub(crate) fn go(&mut self, dest: Dest) {
        self.nav = dest.nav();
        match dest {
            Dest::ProjectOutput => self.native_cleanup.show_project(),
            Dest::DiskScans => self.history_view = crate::screens::HistoryView::DiskScans,
            Dest::Screen(_) => {}
        }
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
