//! Explicit session terminate. Never automatic. Git dirty is a warning.

use eframe::egui::{self, Color32, RichText};
use sweeploom_core::LiveSession;
use sweeploom_dev::inspect;
use sweeploom_process::{SysinfoProcessControl, stop_session_gracefully};

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
            apply_stop(app, session);
        }
    });
}

fn apply_stop(app: &mut SweepLoomApp, session: &LiveSession) {
    let control = SysinfoProcessControl::new();
    app.action_message = Some(
        match stop_session_gracefully(&session.processes, &control) {
            Ok(()) => format!("Asked {} processes to stop.", session.processes.len()),
            Err(error) => format!("Stop failed: {error}"),
        },
    );
    app.confirm_terminate = false;
}
