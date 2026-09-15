//! Explicit session terminate. Never automatic. Git dirty is a warning.

use eframe::egui::{self, RichText};
use sweeploom_core::{LiveSession, PendingTermination, SessionKind};
use sweeploom_dev::inspect;
use sweeploom_platform::ProcessControlCapabilities;
use sweeploom_process::{
    SysinfoProcessControl, force_stop_session, still_running, stop_session_gracefully,
};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn draw(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    if session.safety.terminate_disabled || session.kind == SessionKind::Browser {
        return;
    }
    if let Some(project) = &session.project {
        let git = inspect(&project.0);
        if git.assessment().is_blocked() {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "WARNING: project has Git changes. Stopping the session does not discard them.",
            );
        }
    }
    ui.horizontal(|ui| {
        if crate::widgets::pointer(
            ui.button(RichText::new("Terminate session").color(ui.visuals().warn_fg_color)),
        )
        .clicked()
        {
            let root = session.processes.first().copied();
            app.pending_stop =
                root.and_then(|root| PendingTermination::freeze(root, &session.processes));
            app.confirm_terminate = app.pending_stop.is_some();
            app.confirm_force = false;
            if app.pending_stop.is_none() {
                app.action_message =
                    Some("Cannot confirm: a process is missing a stable start time.".to_owned());
            }
        }
    });
    if !app.confirm_terminate {
        if let Some(message) = &app.action_message {
            ui.label(message);
        }
        return;
    }
    let Some(pending) = app.pending_stop.clone() else {
        app.confirm_terminate = false;
        return;
    };
    if !pending.still_valid(&session.processes) {
        app.pending_stop = None;
        app.confirm_terminate = false;
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "Confirmation cancelled: the session membership changed.",
        );
        return;
    }
    ui.label(format!(
        "Terminate {} ({} frozen process keys, {})?",
        session.label(),
        pending.approved_keys.len(),
        format_bytes(session.rss_bytes)
    ));
    let graceful = ProcessControlCapabilities::host().graceful_stop;
    ui.horizontal(|ui| {
        if crate::widgets::pointer(ui.button("Cancel")).clicked() {
            app.confirm_terminate = false;
            app.pending_stop = None;
        }
        if graceful {
            if crate::widgets::pointer(ui.button("Terminate gracefully")).clicked() {
                apply_stop(app, &pending.approved_keys, false);
            }
        } else {
            ui.label(
                RichText::new("Graceful stop is unsupported on Windows.")
                    .color(ui.visuals().warn_fg_color),
            );
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
        ui.visuals().warn_fg_color,
        format!(
            "{} process(es) still live after graceful stop. Force-kill is never automatic.",
            live.len()
        ),
    );
    if !app.confirm_force {
        if crate::widgets::pointer(ui.button("Force kill remaining…")).clicked() {
            app.confirm_force = true;
        }
        return true;
    }
    ui.horizontal(|ui| {
        if crate::widgets::pointer(ui.button("Cancel")).clicked() {
            app.confirm_force = false;
        }
        if crate::widgets::pointer(
            ui.button(RichText::new("Force kill").color(ui.visuals().error_fg_color)),
        )
        .clicked()
        {
            let keys = app.pending_force.clone().unwrap_or_default();
            apply_stop(app, &keys, true);
        }
    });
    true
}

pub(crate) fn apply_stop(app: &mut SweepLoomApp, keys: &[sweeploom_core::ProcessKey], force: bool) {
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
    app.pending_stop = None;
    app.confirm_force = false;
    app.confirm_planned = false;
    app.confirm_helpers = false;
}
