//! Shared file selection actions and explicit confirmations.
use crate::{app::SweepLoomApp, widgets};
use crossbeam_channel::Receiver;
use eframe::egui;
use std::path::{Path, PathBuf};
use sweeploom_core::{Candidate, DeletionStrategy, UserPolicy};
use sweeploom_exec::{TrashTarget, prepare_trash};

#[path = "disk_actions_fs.rs"]
mod fs;
use fs::{remove_nodes, reveal};

#[derive(Default)]
pub(crate) struct DiskActions {
    pending: Option<Pending>,
    trash_rx: Option<Receiver<(Vec<PathBuf>, Vec<String>)>>,
}

enum Pending {
    Clean(Vec<Candidate>),
    Trash(Vec<TrashTarget>),
}

#[derive(Clone)]
pub(crate) enum PathAction {
    Finder(PathBuf),
    Clean(PathBuf),
    Trash(PathBuf),
}

pub(crate) fn can_clean(candidate: &Candidate) -> bool {
    !candidate.safety.is_blocked()
        && candidate.deletion == DeletionStrategy::PermanentGenerated
        && !matches!(
            candidate.user_policy,
            UserPolicy::NeverClean | UserPolicy::Keep | UserPolicy::PinProject
        )
}

pub(crate) fn menu(response: &egui::Response, path: &Path, action: &mut Option<PathAction>) {
    response.context_menu(|ui| {
        if ui.button("Show in Finder").clicked() {
            *action = Some(PathAction::Finder(path.into()));
            ui.close();
        }
        if ui.button("Clean generated files…").clicked() {
            *action = Some(PathAction::Clean(path.into()));
            ui.close();
        }
        if ui.button("Move to Trash…").clicked() {
            *action = Some(PathAction::Trash(path.into()));
            ui.close();
        }
    });
}

pub(crate) fn toolbar(app: &mut SweepLoomApp, ui: &mut egui::Ui, paths: &[PathBuf]) {
    let ready = !paths.is_empty() && !app.disk_actions.busy() && app.apply_rx.is_none();
    let count = app.clean_candidates_for(paths).len();
    if widgets::button(ui, "Show in Finder", !paths.is_empty()).clicked() {
        app.reveal_paths(paths);
    }
    if widgets::danger_button(ui, "Clean generated…", ready && count > 0)
        .on_disabled_hover_text("Select regenerable output. Inspect-only or blocked data can be inspected or explicitly moved to Trash.")
        .clicked()
    {
        app.request_clean(app.clean_candidates_for(paths));
    }
    if widgets::danger_button(ui, "Move to Trash…", ready).clicked() {
        app.request_trash(paths);
    }
    ui.add_space(crate::theme::XS);
    widgets::caption(
        ui,
        format!("{} selected · {count} cleanable artifacts", paths.len()),
    );
}

impl DiskActions {
    pub(crate) fn busy(&self) -> bool {
        self.trash_rx.is_some()
    }
}

impl SweepLoomApp {
    pub(crate) fn clean_candidates_for(&self, paths: &[PathBuf]) -> Vec<Candidate> {
        let ai = self
            .ai_offers
            .iter()
            .flatten()
            .flat_map(|offer| offer.entries.iter().map(|entry| &entry.candidate));
        let mut items: Vec<Candidate> = self
            .review
            .iter()
            .map(|row| &row.candidate)
            .chain(ai)
            .filter(|candidate| {
                can_clean(candidate) && paths.iter().any(|path| candidate.path.starts_with(path))
            })
            .cloned()
            .collect();
        items.sort_by_key(|item| item.path.components().count());
        let mut kept: Vec<Candidate> = Vec::new();
        for item in items {
            if !kept
                .iter()
                .any(|parent| item.path.starts_with(&parent.path))
            {
                kept.push(item);
            }
        }
        kept
    }

    pub(crate) fn path_action(&mut self, action: PathAction) {
        match action {
            PathAction::Finder(path) => self.reveal_paths(&[path]),
            PathAction::Clean(path) => self.request_clean(self.clean_candidates_for(&[path])),
            PathAction::Trash(path) => self.request_trash(&[path]),
        }
    }

    pub(crate) fn reveal_paths(&mut self, paths: &[PathBuf]) {
        self.action_message = Some(match reveal(paths) {
            Ok(()) => format!("Showing {} item(s) in Finder.", paths.len()),
            Err(error) => format!("Could not open file manager: {error}"),
        });
    }

    pub(crate) fn request_clean(&mut self, candidates: Vec<Candidate>) {
        if self.apply_rx.is_some() || self.disk_actions.busy() {
            self.action_message = Some("A cleanup is already running.".into());
            return;
        }
        let candidates: Vec<_> = candidates.into_iter().filter(can_clean).collect();
        if candidates.is_empty() {
            self.action_message = Some("No cleanable generated files selected. Inspect-only data requires explicit Move to Trash; blockers are shown in Safety.".into());
            return;
        }
        self.disk_actions.pending = Some(Pending::Clean(candidates));
    }

    pub(crate) fn request_trash(&mut self, paths: &[PathBuf]) {
        if self.apply_rx.is_some() || self.disk_actions.busy() {
            self.action_message = Some("A cleanup is already running.".into());
            return;
        }
        let mut sorted = paths.to_vec();
        sorted.sort_by_key(|path| path.components().count());
        let mut targets: Vec<TrashTarget> = Vec::new();
        for path in sorted {
            if targets.iter().any(|target| path.starts_with(&target.path)) {
                continue;
            }
            // Manual, recoverable Trash has its own path and live-process checks.
            // A project's Git state only governs generated permanent cleanup.
            match prepare_trash(&path, &self.locations.home) {
                Ok(target) => targets.push(target),
                Err(error) => {
                    self.action_message = Some(format!("{}: {error}", path.display()));
                    return;
                }
            }
        }
        if !targets.is_empty() {
            self.disk_actions.pending = Some(Pending::Trash(targets));
        }
    }

    pub(crate) fn disk_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.disk_actions.pending else {
            return;
        };
        let (title, hint, paths, button) = match pending {
            Pending::Clean(items) => (
                "Confirm generated cleanup",
                "Permanently delete the listed generated files. They will need rebuilding. Live safety checks run again before deletion.",
                items.iter().map(|item| &item.path).collect::<Vec<_>>(),
                "Delete generated files",
            ),
            Pending::Trash(items) => (
                "Confirm Move to Trash",
                "Move only the listed items to the macOS Trash. Restore them from Finder if needed. Disk space is released after you empty Trash.",
                items.iter().map(|item| &item.path).collect::<Vec<_>>(),
                "Move to Trash",
            ),
        };
        let mut confirm = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("disk-confirmation")).show(ctx, |ui| {
            ui.set_width(580.0);
            ui.heading(title);
            ui.label(hint);
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    for path in paths {
                        ui.label(path.display().to_string());
                    }
                });
            ui.separator();
            ui.horizontal(|ui| {
                cancel = ui.button("Cancel").clicked();
                confirm = widgets::apply_button(ui, button).clicked();
            });
        });
        cancel |= response.should_close();
        if cancel {
            self.disk_actions.pending = None;
        }
        if confirm && let Some(pending) = self.disk_actions.pending.take() {
            let processes = self.snapshot.as_ref().map(|item| item.processes.clone());
            let Some(processes) = processes else {
                self.action_message =
                    Some("Process evidence is unavailable. Refresh and try again.".into());
                return;
            };
            match pending {
                Pending::Clean(items) => {
                    self.action_message = Some("Revalidating generated cleanup…".into());
                    self.apply_rx = Some(crate::scan_job::spawn_apply(items, processes));
                }
                Pending::Trash(items) => {
                    let (tx, rx) = crossbeam_channel::bounded(1);
                    let home = self.locations.home.clone();
                    self.disk_actions.trash_rx = Some(rx);
                    self.action_message = Some("Moving selected items to Trash…".into());
                    std::thread::spawn(move || {
                        let result = sweeploom_exec::apply_trash_with(&items, &home, &processes);
                        let _ = tx.send(result);
                    });
                }
            }
        }
    }

    pub(crate) fn poll_trash(&mut self) {
        let Some(rx) = &self.disk_actions.trash_rx else {
            return;
        };
        let Ok((removed, errors)) = rx.try_recv() else {
            return;
        };
        self.disk_actions.trash_rx = None;
        self.project_sizes.invalidate();
        self.review.retain(|row| {
            !removed
                .iter()
                .any(|path| row.candidate.path.starts_with(path))
        });
        self.project_cards
            .retain(|card| !removed.iter().any(|path| card.path.starts_with(path)));
        self.project_roots
            .retain(|root| !removed.iter().any(|path| root.starts_with(path)));
        self.selected_explorer.clear();
        self.selected_projects.clear();
        if let Some(report) = &mut self.inventory {
            remove_nodes(&mut report.tree, &removed);
        }
        self.ai_offers = None;
        self.action_message = Some(format!(
            "Moved {} item(s) to Trash. {}",
            removed.len(),
            errors.join("\n")
        ));
    }
}
