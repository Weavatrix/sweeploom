//! Session reclaim planner. Selection only; terminate stays explicit.

use std::collections::HashSet;

use eframe::egui::{self, RichText};
use sweeploom_core::{LiveSession, ProcessKey};
use sweeploom_session::{plan_free_ram, plan_quiet_workstation, plan_reduce_cpu};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets;

/// Planner group: pre-select sessions by RAM, CPU or quiet target. Confirm UI is [`extras`].
pub fn controls(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.label(
        RichText::new("Select for cleanup")
            .size(13.0)
            .color(crate::theme::muted(ui)),
    );
    if widgets::input_group(
        ui,
        "Free RAM",
        Some((&mut app.free_ram_gb, "GB")),
        "Select",
        "Pre-select forgotten sessions for this much RAM. Terminate is never automatic.",
    ) {
        app.plan_free_ram();
    }
    if widgets::input_group(
        ui,
        "Cut CPU by",
        Some((&mut app.reduce_cpu, "%")),
        "Select",
        "Pre-select forgotten sessions until this CPU share. Terminate is never automatic.",
    ) {
        app.plan_reduce_cpu();
    }
    if widgets::input_group(
        ui,
        "Quiet workstation",
        None,
        "Select",
        "Pre-select forgotten sessions. Protects the current project and browsers. Terminate is never automatic.",
    ) {
        app.plan_quiet();
    }
    let (count, rss, cpu) = planned_totals(app);
    if let Some(text) = planned_caption(count, rss, cpu) {
        widgets::pill(ui, &format!("Planned {text}"), crate::theme::Tone::Caution);
    }
}

/// Confirm / force / planner messages. Hidden when idle.
pub fn extras(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let planned: Vec<LiveSession> = app
        .sessions
        .iter()
        .filter(|session| session_planned(session, &app.planned_keys))
        .cloned()
        .collect();
    if !planned.is_empty() {
        draw_confirm(app, ui, &planned);
    }
    let _ = super::session_actions::draw_force(app, ui);
    if let Some(message) = &app.action_message
        && is_session_message(message)
    {
        ui.label(message);
    }
}

fn planned_totals(app: &SweepLoomApp) -> (usize, u64, f32) {
    let planned: Vec<&LiveSession> = app
        .sessions
        .iter()
        .filter(|session| session_planned(session, &app.planned_keys))
        .collect();
    let rss = planned
        .iter()
        .map(|session| session.recommendation.estimated_reclaimable_rss)
        .sum();
    let cpu = planned.iter().map(|session| session.cpu_percent).sum();
    (planned.len(), rss, cpu)
}

#[must_use]
fn planned_caption(count: usize, rss: u64, cpu: f32) -> Option<String> {
    (count > 0).then(|| format!("{count} · {} · {:.0}% CPU", format_bytes(rss), cpu))
}

fn is_session_message(text: &str) -> bool {
    text.contains("planned")
        || text.starts_with("Asked ")
        || text.starts_with("Stop failed")
        || text.starts_with("Nothing planned")
        || text.starts_with("Enter a size")
}

/// Checkbox for one session row. System-critical stays disabled.
pub fn checkbox(ui: &mut egui::Ui, session: &LiveSession, planned: &mut HashSet<ProcessKey>) {
    let mut on = session_planned(session, planned);
    let enabled = !session.safety.terminate_disabled;
    if widgets::check_enabled(ui, &mut on, enabled).changed() {
        set_planned(session, planned, on);
    }
}

/// Checkbox that plans every member of a title group.
pub fn checkbox_group(
    ui: &mut egui::Ui,
    sessions: &[LiveSession],
    indexes: &[usize],
    planned: &mut HashSet<ProcessKey>,
) {
    let members: Vec<&LiveSession> = indexes.iter().filter_map(|&i| sessions.get(i)).collect();
    if members.is_empty() {
        return;
    }
    let mut on = members
        .iter()
        .all(|session| session_planned(session, planned));
    let enabled = members
        .iter()
        .any(|session| !session.safety.terminate_disabled);
    if widgets::check_enabled(ui, &mut on, enabled).changed() {
        for session in members {
            if !session.safety.terminate_disabled {
                set_planned(session, planned, on);
            }
        }
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
        if widgets::danger_button(ui, "Terminate planned…", true).clicked() {
            app.confirm_planned = true;
            app.confirm_terminate = false;
        }
        return;
    }
    if planned.iter().any(|session| {
        session
            .project
            .as_ref()
            .is_some_and(|project| super::session_git::blocked(ui.ctx(), &project.0) == Some(true))
    }) {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "WARNING: a planned project has Git changes. Stopping does not discard them.",
        );
    }
    ui.label(format!(
        "Terminate {} planned session(s)? Recommendation never bypasses a blocker.",
        planned.len()
    ));
    ui.horizontal(|ui| {
        if crate::widgets::pointer(ui.button("Cancel")).clicked() {
            app.confirm_planned = false;
        }
        if widgets::apply_button(ui, "Terminate gracefully").clicked() {
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

#[cfg(test)]
mod tests {
    #[test]
    fn explorer_status_stays_off_sessions() {
        assert!(!super::is_session_message("252 candidates"));
        assert!(!super::is_session_message(
            "Folders ready. Building Review in the background…"
        ));
        assert!(super::is_session_message("3 session(s) planned"));
    }

    #[test]
    fn idle_plan_has_no_caption() {
        assert_eq!(super::planned_caption(0, 0, 0.0), None);
        assert_eq!(
            super::planned_caption(2, 2 * 1024 * 1024 * 1024, 4.0).as_deref(),
            Some("2 · 2.1 GB · 4% CPU")
        );
    }
}
