//! Projects table rows: folder/kind groups and individual project folders.

use std::collections::HashSet;
use std::path::PathBuf;

use eframe::egui::{self, RichText};
use sweeploom_storage::DiskUsage;

use super::super::project_rows::{ProjectGroup, size_caption};
use crate::app::SweepLoomApp;
use crate::disk_actions::{self, PathAction};
use crate::{theme, widgets};

/// Mutable outcomes collected while drawing rows.
pub(super) struct RowState<'a> {
    pub selection: &'a mut HashSet<PathBuf>,
    pub toggle: &'a mut Option<String>,
    pub scan: &'a mut Option<PathBuf>,
    pub action: &'a mut Option<PathAction>,
}

/// One group header line.
pub(super) struct GroupLine<'a> {
    pub key: &'a str,
    pub title: &'a str,
    pub count: usize,
    pub bytes: Option<DiskUsage>,
    pub expanded: bool,
}

/// Header checkbox: select or clear every project folder.
pub(super) fn select_all(
    ui: &mut egui::Ui,
    cards: &[super::super::ProjectCard],
    selection: &mut HashSet<PathBuf>,
) {
    let mut all = cards.iter().all(|card| selection.contains(&card.path));
    if widgets::check(ui, &mut all).changed() {
        for card in cards {
            if all {
                selection.insert(card.path.clone());
            } else {
                selection.remove(&card.path);
            }
        }
    }
}

pub(super) fn group_row(
    app: &SweepLoomApp,
    row: &mut egui_extras::TableRow<'_, '_>,
    group: ProjectGroup,
    line: &GroupLine<'_>,
    state: &mut RowState<'_>,
) {
    let members = app
        .project_cards
        .iter()
        .filter(|card| match group {
            ProjectGroup::Kind => card.group_kind == line.key,
            _ => card.folder_key == line.key,
        })
        .map(|card| card.path.clone())
        .collect::<Vec<_>>();
    let mut all = members.iter().all(|path| state.selection.contains(path));
    row.set_selected(all);
    row.col(|ui| {
        if widgets::check(ui, &mut all).changed() {
            for path in &members {
                if all {
                    state.selection.insert(path.clone());
                } else {
                    state.selection.remove(path);
                }
            }
        }
    });
    row.col(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if widgets::disclose(ui, line.expanded, theme::muted(ui)) {
            *state.toggle = Some(line.key.to_owned());
        }
        let title = ui.add(
            egui::Label::new(RichText::new(line.title).strong())
                .truncate()
                .sense(egui::Sense::click()),
        );
        ui.label(
            RichText::new(format!("{} projects", line.count))
                .size(12.0)
                .color(theme::muted(ui)),
        );
        if title.clicked() {
            *state.toggle = Some(line.key.to_owned());
        }
    });
    row.col(|ui| {
        let kind = if group == ProjectGroup::Kind {
            "Kind"
        } else {
            "Folder"
        };
        ui.label(RichText::new(kind).color(theme::muted(ui)));
    });
    row.col(|ui| {
        ui.label(RichText::new(size_caption(line.bytes, None)).strong())
            .on_hover_text("Total of measured project folders in this group, counting nested projects once. A >= value is incomplete while other projects are still being measured or could not be read.");
    });
    row.col(|_ui| {});
    row.col(|_ui| {});
    if group == ProjectGroup::Parent {
        disk_actions::menu(&row.response(), &PathBuf::from(line.key), state.action);
    }
}

pub(super) fn project_row(
    app: &SweepLoomApp,
    row: &mut egui_extras::TableRow<'_, '_>,
    group: ProjectGroup,
    index: usize,
    state: &mut RowState<'_>,
) {
    let card = &app.project_cards[index];
    let mut selected = state.selection.contains(&card.path);
    row.set_selected(selected);
    row.col(|ui| {
        widgets::check(ui, &mut selected);
    });
    row.col(|ui| {
        if group != ProjectGroup::None {
            ui.add_space(22.0);
        }
        let path = card.path.display().to_string();
        let response = widgets::two_line(ui, card.name(), &path)
            .interact(egui::Sense::click())
            .on_hover_text(&path);
        if response.clicked() {
            selected = !selected;
        }
        if response.double_clicked() {
            *state.scan = Some(card.path.clone());
        }
    });
    row.col(|ui| {
        ui.add(egui::Label::new(&card.kinds).truncate())
            .on_hover_text(&card.kinds);
    });
    row.col(|ui| {
        let note = match (&card.bytes, &card.size_error) {
            (_, Some(error)) => format!("Could not read the project folder: {error}"),
            (Some(usage), _) => format!(
                "Whole project folder: {} files, {} read errors. Includes source, dependencies, hidden and generated files.",
                usage.files, usage.errors
            ),
            _ => "Measuring the whole project folder in the background. Artifact size is shown separately.".into(),
        };
        ui.label(size_caption(card.bytes, card.size_error.as_deref()))
            .on_hover_text(note);
        if app.project_sizes.is_cached(&card.path) {
            ui.label(RichText::new("Saved").size(11.5).color(theme::muted(ui)));
        }
    });
    row.col(|ui| {
        if card.bytes.is_some() {
            crate::scan_history::trend(
                ui,
                &app.scan_history,
                crate::scan_history::Source::Projects,
                &card.path,
            );
        }
    });
    row.col(|ui| {
        ui.add(egui::Label::new(&card.artifacts).truncate())
            .on_hover_text(&card.artifacts);
    });
    if selected {
        state.selection.insert(card.path.clone());
    } else {
        state.selection.remove(&card.path);
    }
    disk_actions::menu(&row.response(), &card.path, state.action);
}
