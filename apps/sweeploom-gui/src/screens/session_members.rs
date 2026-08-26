//! Member processes of a logical session. Helpers can be stopped without the root.

use eframe::egui::{self, RichText};
use sweeploom_core::{LiveSession, ProcessKey, ProcessSafetyClass, ProcessSnapshot, SessionKind};
use sweeploom_session::classify_process;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::widgets::pointer;

pub fn draw(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let members = member_snapshots(app, session);
    if members.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.label(RichText::new("Member processes").size(16.0).strong());
    if session.kind == SessionKind::Browser {
        ui.label("Tabs are not OS processes. Discard stale tabs on Browser; this tree stays Keep.");
        if pointer(ui.button("Open Browser")).clicked() {
            app.nav = Nav::Browser;
        }
        for process in members.iter().take(16) {
            ui.label(format!(
                "{}  pid {}  {}  {:.1}%",
                process.name,
                process.pid,
                format_bytes(process.rss_bytes),
                process.cpu_percent
            ));
        }
        return;
    }
    ui.label(
        "Check a helper to stop it without ending the whole session. Root is the session itself.",
    );
    let root = session.processes.first().copied();
    let mut selected = std::mem::take(&mut app.helper_keys);
    for process in members.iter().take(24) {
        draw_member_row(ui, process, root, &mut selected);
    }
    app.helper_keys = selected;
    draw_stop(app, ui, session);
}

fn member_snapshots(app: &SweepLoomApp, session: &LiveSession) -> Vec<ProcessSnapshot> {
    let Some(snapshot) = &app.snapshot else {
        return Vec::new();
    };
    session
        .processes
        .iter()
        .filter_map(|key| {
            snapshot
                .processes
                .iter()
                .find(|process| process.key == *key)
                .cloned()
        })
        .collect()
}

fn draw_member_row(
    ui: &mut egui::Ui,
    process: &ProcessSnapshot,
    root: Option<ProcessKey>,
    selected: &mut std::collections::HashSet<ProcessKey>,
) {
    let blocked = process.safety_class == ProcessSafetyClass::SystemCritical;
    let mut on = selected.contains(&process.key);
    let role = member_role(process, root);
    let cmd = short_command(&process.command);
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!blocked, egui::Checkbox::new(&mut on, ""))
            .changed()
        {
            if on {
                selected.insert(process.key);
            } else {
                selected.remove(&process.key);
            }
        }
        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&process.name).strong());
                ui.label(
                    RichText::new(format!(
                        "{role} · pid {} · {} · {:.1}%",
                        process.pid,
                        format_bytes(process.rss_bytes),
                        process.cpu_percent
                    ))
                    .color(crate::theme::muted(ui)),
                );
            });
            if !cmd.is_empty() {
                ui.monospace(cmd);
            }
        });
    });
}

fn member_role(process: &ProcessSnapshot, root: Option<ProcessKey>) -> &'static str {
    if Some(process.key) == root {
        return "root";
    }
    classify_process(process)
        .map(|item| item.kind.label())
        .unwrap_or("helper")
}

fn short_command(command: &[String]) -> String {
    let joined = command.join(" ");
    if joined.chars().count() <= 72 {
        joined
    } else {
        format!("{}…", joined.chars().take(72).collect::<String>())
    }
}

fn draw_stop(app: &mut SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let keys: Vec<ProcessKey> = session
        .processes
        .iter()
        .copied()
        .filter(|key| app.helper_keys.contains(key))
        .collect();
    if keys.is_empty() {
        app.confirm_helpers = false;
        return;
    }
    if !app.confirm_helpers {
        if pointer(ui.button("Stop selected processes…")).clicked() {
            app.confirm_helpers = true;
            app.confirm_terminate = false;
        }
        return;
    }
    ui.label(format!(
        "Stop {} selected process(es) in {}? The rest of the session stays running.",
        keys.len(),
        session.label()
    ));
    ui.horizontal(|ui| {
        if pointer(ui.button("Cancel")).clicked() {
            app.confirm_helpers = false;
        }
        if pointer(ui.button("Stop gracefully")).clicked() {
            super::session_actions::apply_stop(app, &keys, false);
            app.helper_keys.clear();
            app.confirm_helpers = false;
        }
    });
}
