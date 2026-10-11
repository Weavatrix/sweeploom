//! Process-tree member list. The Browser process stays Keep.

use sweeploom_core::{ProcessKey, SessionId};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::pointer;
use eframe::egui::{self, RichText};

use super::browser_trees::TreeRow;

pub(super) fn draw(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, rows: &[TreeRow]) {
    draw_tree_detail(app, ui);
    draw_stop(app, ui, rows);
    let _ = super::session_actions::draw_force(app, ui);
}

fn draw_tree_detail(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let Some(id) = app.browser.selected_tree else {
        ui.label("Select a row for RAM/CPU vs the machine and member processes.");
        return;
    };
    let Some(session) = app.sessions.iter().find(|item| item.id == id).cloned() else {
        return;
    };
    egui::ScrollArea::vertical()
        .id_salt("browser-tree-detail")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.separator();
            let root = app.snapshot.as_ref().and_then(|snapshot| {
                session.processes.first().and_then(|key| {
                    snapshot
                        .processes
                        .iter()
                        .find(|process| process.key == *key)
                })
            });
            if let Some(root) = root {
                ui.label(RichText::new(format!("{} · PID {}", root.name, root.pid)).strong());
                if let Some(exe) = &root.exe {
                    ui.label(exe.display().to_string());
                }
            }
            super::session_observe::draw(app, ui, &session);
            let processes = app
                .snapshot
                .as_ref()
                .map(|item| item.processes.as_slice())
                .unwrap_or(&[]);
            ui.add_space(6.0);
            ui.label(RichText::new("Member processes").strong());
            for key in &session.processes {
                if let Some(process) = processes.iter().find(|item| item.key == *key) {
                    ui.label(format!(
                        "{}  pid {}  {}  {:.1}%",
                        process.name,
                        process.pid,
                        format_bytes(process.rss_bytes),
                        process.cpu_percent
                    ));
                }
            }
        });
}

fn draw_stop(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, rows: &[TreeRow]) {
    let chosen: Vec<SessionId> = rows
        .iter()
        .filter(|row| row.stoppable && app.browser.tree_ids.contains(&row.id))
        .map(|row| row.id)
        .collect();
    if chosen.is_empty() {
        app.browser.confirm_helpers = false;
        ui.label("Check Renderer / Content / Utility / Extension rows to stop them. GPU and the Browser process stay.");
        return;
    }
    if !app.browser.confirm_helpers {
        if pointer(ui.button(format!("Stop {} helper tree(s)…", chosen.len()))).clicked() {
            app.browser.confirm_helpers = true;
        }
        return;
    }
    ui.label("Edge/Chrome itself stays. Those helpers unload; tabs they served may reload.");
    ui.horizontal(|ui| {
        if pointer(ui.button("Cancel")).clicked() {
            app.browser.confirm_helpers = false;
        }
        if pointer(ui.button("Stop gracefully")).clicked() {
            stop_helpers(app, &chosen);
        }
    });
}

fn stop_helpers(app: &mut SweepLoomApp, ids: &[SessionId]) {
    let keys: Vec<ProcessKey> = app
        .sessions
        .iter()
        .filter(|session| ids.contains(&session.id))
        .flat_map(|session| session.processes.iter().copied())
        .collect();
    super::session_actions::apply_stop(app, &keys, false);
    app.browser.tree_ids.clear();
    app.browser.confirm_helpers = false;
}
