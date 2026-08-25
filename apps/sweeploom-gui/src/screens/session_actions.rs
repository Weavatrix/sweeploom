//! Explicit session terminate. Never automatic. Git dirty is a warning.

use eframe::egui::{self, Color32, RichText};
use sweeploom_core::LiveSession;
use sweeploom_dev::inspect;
use sweeploom_process::{
    SysinfoProcessControl, force_stop_session, still_running, stop_session_gracefully,
};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn draw(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    if session.safety.terminate_disabled {
        return;
    }
    if let Some(project) = &session.project {
        let git = inspect(&project.0);
        if git.assessment().is_blocked() {
            ui.colored_label(
                Color32::from_rgb(240, 160, 80),
                "WARNING: project has Git changes. Stopping the session does not discard them.",
            );
        }
    }
    ui.horizontal(|ui| {
        if ui
            .button(RichText::new("Terminate session").color(Color32::from_rgb(240, 180, 120)))
            .clicked()
        {
            app.confirm_terminate = true;
            app.confirm_force = false;
        }
    });
    if !app.confirm_terminate {
        if let Some(message) = &app.action_message {
            ui.label(message);
        }
        return;
    }
    ui.label(format!(
        "Terminate {} ({} processes, {})?",
        session.label(),
        session.processes.len(),
        format_bytes(session.rss_bytes)
    ));
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            app.confirm_terminate = false;
        }
        if ui.button("Terminate gracefully").clicked() {
            apply_stop(app, &session.processes, false);
        }
    });
}

pub(crate) fn draw_force(app: &mut SweepLoomApp, ui: &mut egui::Ui) -> bool {
    let Some(pending) = &app.pending_force else {
        return false;
    };
    let live = app
        .snapshot
        .as_ref()
        .map(|item| still_running(pending, &item.processes))
        .unwrap_or_default();
    if live.is_empty() {
        app.pending_force = None;
        app.confirm_force = false;
        return false;
    }
    ui.colored_label(
        Color32::from_rgb(240, 160, 80),
        format!(
            "{} process(es) still live after graceful stop. Force-kill is never automatic.",
            live.len()
        ),
    );
    if !app.confirm_force {
        if ui.button("Force kill remaining…").clicked() {
            app.confirm_force = true;
        }
        return true;
    }
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            app.confirm_force = false;
        }
        if ui
            .button(RichText::new("Force kill").color(Color32::from_rgb(240, 120, 80)))
            .clicked()
        {
            let keys = app.pending_force.clone().unwrap_or_default();
            apply_stop(app, &keys, true);
        }
    });
    true
}

fn apply_stop(app: &mut SweepLoomApp, keys: &[sweeploom_core::ProcessKey], force: bool) {
    let control = SysinfoProcessControl::new();
    app.action_message = Some(if force {
        match force_stop_session(keys, &control) {
            Ok(()) => format!("Force-killed {} process key(s).", keys.len()),
            Err(error) => format!("Force kill failed: {error}"),
        }
    } else {
        match stop_session_gracefully(keys, &control) {
            Ok(()) => format!("Asked {} processes to stop.", keys.len()),
            Err(error) => format!("Stop failed: {error}"),
        }
    });
    app.pending_force = Some(keys.to_vec());
    app.confirm_terminate = false;
    app.confirm_force = false;
    app.confirm_planned = false;
}
