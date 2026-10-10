//! Selected session summary, activity and member actions in a separate card.

use eframe::egui::{self, RichText};
use sweeploom_core::LiveSession;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::theme;
use crate::widgets::{self, StatTile, TileViz, Tone};

use super::session_actions;
use super::session_label;
use super::session_members;
use super::session_observe;

#[path = "session_detail_info.rs"]
mod info;

pub fn draw_details(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let session = app
        .selected_session
        .and_then(|id| app.sessions.iter().find(|item| item.id == id).cloned());
    let Some(session) = session else {
        ui.add_space(theme::SM);
        widgets::caption(
            ui,
            "Select a session to see its activity, launch origin and member processes that can be stopped on their own.",
        );
        return;
    };
    ui.add_space(theme::SM);
    egui::ScrollArea::vertical()
        .id_salt(("session-details", session.id.0))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            widgets::card(ui, |ui| session_details(app, ui, &session));
        });
}

fn session_details(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let title = session_label::title(session, processes);
    ui.horizontal(|ui| {
        crate::brand::show_session(ui, session.kind, &title, 22.0);
        ui.label(RichText::new(&title).size(20.0).strong());
        widgets::status_pill(ui, session_label::status(session));
        widgets::caption(ui, session.kind.label());
    });
    ui.add_space(theme::MD);
    widgets::stat_row(ui, &stat_tiles(app, session), 170.0);
    info::draw_identity(app, ui, session);
    ui.add_space(theme::LG);
    widgets::section_heading(ui, "Activity");
    session_observe::draw(app, ui, session);
    session_members::draw(app, ui, session);
    ui.add_space(theme::MD);
    ui.separator();
    ui.add_space(theme::XS);
    if session.safety.terminate_disabled {
        ui.label(
            RichText::new("Terminate disabled (system-critical).")
                .color(theme::tone(ui, Tone::Warn)),
        );
    }
    session_actions::draw(app, ui, session);
}

fn stat_tiles(app: &SweepLoomApp, session: &LiveSession) -> Vec<StatTile> {
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let root = session_label::root(session, processes);
    let node_count = session
        .processes
        .iter()
        .filter(|key| {
            processes
                .iter()
                .any(|p| p.key == **key && sweeploom_session::is_node(p))
        })
        .count();
    let (used, cores) = app
        .snapshot
        .as_ref()
        .map_or((0, 1), |s| (s.memory.used_bytes, s.cpu.cores.len().max(1)));
    let ram_share = session.rss_bytes as f32 / used.max(1) as f32;
    let machine = session.cpu_percent / (cores as f32 * 100.0);
    let tile = |label: &str, value: String, sub: String, tone, viz| StatTile {
        icon: None,
        label: label.into(),
        value,
        sub,
        tone,
        viz,
    };
    vec![
        tile(
            "Memory",
            format_bytes(session.rss_bytes),
            format!("{:.0}% of RAM in use", ram_share * 100.0),
            Tone::Neutral,
            TileViz::Meter(ram_share),
        ),
        tile(
            "CPU",
            format!("{:.1}%", session.cpu_percent),
            format!("≈ {:.1} of {cores} cores", session.cpu_percent / 100.0),
            if session.cpu_percent >= 80.0 {
                Tone::Warn
            } else {
                Tone::Neutral
            },
            TileViz::Meter(machine),
        ),
        tile(
            "Processes",
            session.processes.len().to_string(),
            format!("{node_count} Node processes"),
            Tone::Neutral,
            TileViz::None,
        ),
        tile(
            "Running for",
            root.map(|p| session_label::duration(p.runtime))
                .unwrap_or_else(|| "Unavailable".into()),
            root.map(|p| format!("Root PID {}", p.pid))
                .unwrap_or_else(|| session.kind.label().into()),
            Tone::Neutral,
            TileViz::None,
        ),
    ]
}

pub(super) fn field(ui: &mut egui::Ui, label: &str, value: impl Into<String>) {
    ui.label(RichText::new(label).color(theme::muted(ui)));
    ui.add(egui::Label::new(value.into()).wrap().selectable(true));
    ui.end_row();
}
