//! Selected session member list and terminate actions.

use eframe::egui::{self, RichText};
use sweeploom_core::LiveSession;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

use super::session_actions;
use super::session_label;
use super::session_members;
use super::session_observe;

pub fn draw_details(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let Some(id) = app.selected_session else {
        ui.label("Select a session to see member processes that can be stopped on their own.");
        return;
    };
    let Some(session) = app.sessions.iter().find(|item| item.id == id).cloned() else {
        ui.label("Select a session to see member processes that can be stopped on their own.");
        return;
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            session_details(app, ui, &session);
        });
}

fn session_details(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let title = session_label::title(session, processes);
    ui.separator();
    ui.label(RichText::new(title).size(20.0).strong());
    ui.label(format!(
        "{} · RAM {} · CPU {:.1}% · processes {} · {}",
        session.kind.label(),
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity.label()
    ));
    session_observe::draw(app, ui, session);
    session_members::draw(app, ui, session);
    if session.safety.terminate_disabled {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "Terminate disabled (system-critical).",
        );
    }
    session_actions::draw(app, ui, session);
}
