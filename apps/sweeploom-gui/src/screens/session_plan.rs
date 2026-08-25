//! Session reclaim planner. Selection only; terminate stays explicit.

use std::collections::HashSet;

use eframe::egui::{self, Color32, RichText};
use sweeploom_core::{LiveSession, ProcessKey};
use sweeploom_session::{plan_free_ram, plan_quiet_workstation, plan_reduce_cpu};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use sweeploom_dev::inspect;

/// Planner toolbar: Free X GB RAM, Reduce CPU, terminate planned.
pub fn draw(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Free");
        ui.add(egui::TextEdit::singleline(&mut app.free_ram_gb).desired_width(40.0));
        ui.label("GB RAM");
        if ui.button("Select to free RAM").clicked() {
            app.plan_free_ram();
        }
        ui.separator();
        ui.label("Cut");
        ui.add(egui::TextEdit::singleline(&mut app.reduce_cpu).desired_width(40.0));
        ui.label("% CPU");
        if ui.button("Select to reduce CPU").clicked() {
            app.plan_reduce_cpu();
        }
        ui.separator();
        if ui.button("Quiet workstation").clicked() {
            app.plan_quiet();
        }
    });
    let planned: Vec<LiveSession> = app
        .sessions
        .iter()
        .filter(|session| session_planned(session, &app.planned_keys))
        .cloned()
        .collect();
    let rss: u64 = planned
        .iter()
        .map(|session| session.recommendation.estimated_reclaimable_rss)
        .sum();
    let cpu: f32 = planned.iter().map(|session| session.cpu_percent).sum();
    ui.label(format!(
        "{} planned · ~{} RAM · {:.1}% CPU — terminate is never automatic",
        planned.len(),
        format_bytes(rss),
        cpu
    ));
    if !planned.is_empty() {
        draw_confirm(app, ui, &planned);
    }
    let _ = super::session_actions::draw_force(app, ui);
    if let Some(message) = &app.action_message {
        ui.label(message);
    }
}

/// Checkbox for one session row. System-critical stays disabled.
pub fn checkbox(ui: &mut egui::Ui, session: &LiveSession, planned: &mut HashSet<ProcessKey>) {
    let mut on = session_planned(session, planned);
    let enabled = !session.safety.terminate_disabled;
    if ui
        .add_enabled(enabled, egui::Checkbox::new(&mut on, ""))
        .changed()
    {
        set_planned(session, planned, on);
    }
}

/// True when this session is in the current plan.
#[must_use]
pub fn session_planned(session: &LiveSession, planned: &HashSet<ProcessKey>) -> bool {
    session.processes.iter().any(|key| planned.contains(key))
}

fn set_planned(session: &LiveSession, planned: &mut HashSet<ProcessKey>, on: bool) {
    if on {
        planned.extend(session.processes.iter().copied());
    } else {
        for key in &session.processes {
            planned.remove(key);
        }
    }
}

fn draw_confirm(app: &mut SweepLoomApp, ui: &mut egui::Ui, planned: &[LiveSession]) {
    if !app.confirm_planned {
        if ui
            .button(RichText::new("Terminate planned…").color(Color32::from_rgb(240, 180, 120)))
            .clicked()
        {
            app.confirm_planned = true;
            app.confirm_terminate = false;
        }
        return;
    }
    if planned.iter().any(|session| {
        session
            .project
            .as_ref()
            .is_some_and(|project| inspect(&project.0).assessment().is_blocked())
    }) {
        ui.colored_label(
            Color32::from_rgb(240, 160, 80),
            "WARNING: a planned project has Git changes. Stopping does not discard them.",
        );
    }
    ui.label(format!(
        "Terminate {} planned session(s)? Recommendation never bypasses a blocker.",
        planned.len()
    ));
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            app.confirm_planned = false;
        }
        if ui.button("Terminate gracefully").clicked() {
            app.terminate_planned();
        }
    });
}

impl SweepLoomApp {
    /// Pre-select forgotten sessions until `free_ram_gb` is reached.
    pub fn plan_free_ram(&mut self) {
        let gb: f64 = self.free_ram_gb.trim().parse().unwrap_or(0.0);
        let target = (gb * 1_000_000_000.0) as u64;
        apply_ids(self, plan_free_ram(&self.sessions, target), target == 0);
    }

    /// Pre-select forgotten sessions until combined CPU reaches `reduce_cpu`.
    pub fn plan_reduce_cpu(&mut self) {
        let target: f32 = self.reduce_cpu.trim().parse().unwrap_or(0.0);
        apply_ids(self, plan_reduce_cpu(&self.sessions, target), target <= 0.0);
    }

    /// Pre-select forgotten sessions, protecting the current project and browsers.
    pub fn plan_quiet(&mut self) {
        apply_ids(
            self,
            plan_quiet_workstation(&self.sessions, self.current_project.as_ref()),
            false,
        );
    }

    /// Ask planned sessions to stop. Force-kill remains a second confirm.
    pub fn terminate_planned(&mut self) {
        let keys: Vec<ProcessKey> = self
            .sessions
            .iter()
            .filter(|session| session_planned(session, &self.planned_keys))
            .filter(|session| {
                !session.safety.terminate_disabled && !session.safety.assessment.is_blocked()
            })
            .flat_map(|session| session.processes.iter().copied())
            .collect();
        self.confirm_planned = false;
        if keys.is_empty() {
            self.action_message = Some("Nothing planned that is safe to stop.".to_owned());
            return;
        }
        let control = sweeploom_process::SysinfoProcessControl::new();
        self.action_message = Some(
            match sweeploom_process::stop_session_gracefully(&keys, &control) {
                Ok(()) => format!("Asked {} processes to stop.", keys.len()),
                Err(error) => format!("Stop failed: {error}"),
            },
        );
        self.pending_force = Some(keys);
        self.confirm_force = false;
    }
}

fn apply_ids(app: &mut SweepLoomApp, ids: Vec<sweeploom_core::SessionId>, bad_target: bool) {
    app.planned_keys.clear();
    app.confirm_planned = false;
    if bad_target {
        app.action_message = Some("Enter a size greater than 0.".to_owned());
        return;
    }
    for session in &app.sessions {
        if ids.contains(&session.id) {
            app.planned_keys.extend(session.processes.iter().copied());
        }
    }
    app.action_message = Some(format!("{} session(s) planned", ids.len()));
}
