//! Projects with group selection and shared file actions.
use super::project_rows::{
    Line, ProjectGroup, refresh_cards, size_caption, sort_cards, table_lines,
};
use crate::{
    app::SweepLoomApp,
    disk_actions,
    nav::Nav,
    sort::{Col, header_cell},
    theme, widgets,
};
use eframe::egui::{self, RichText};
use egui_extras::Column;
use std::path::PathBuf;

pub fn ui_projects(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    widgets::page_title(
        ui,
        "Projects",
        "Sizes cover entire project folders, including source and generated files. Folder groups total their projects; nested projects count once. Double-click to open Explorer.",
    );
    refresh_cards(app);
    let mut paths = app.project_cards.iter().collect::<Vec<_>>();
    // Artifact sizes prioritize measurement only; they never stand in for the folder size.
    paths.sort_by_key(|card| std::cmp::Reverse(card.artifact_bytes));
    let paths = paths
        .into_iter()
        .map(|card| card.path.clone())
        .collect::<Vec<_>>();
    app.project_sizes
        .poll(&paths, ui.ctx(), &mut app.scan_history);
    refresh_cards(app);
    widgets::toolbar(ui, |ui| {
        let mut group = app.project_group;
        ui.label(RichText::new("Group by").size(13.0).color(theme::muted(ui)));
        let modes = [ProjectGroup::Kind, ProjectGroup::Parent, ProjectGroup::None];
        let options = modes.map(|mode| (mode, mode.label()));
        if widgets::segmented(ui, &mut group, &options) {
            app.project_group = group;
        }
        ui.add(egui::Separator::default().vertical().spacing(theme::MD));
        if widgets::button(ui, "Rebuild", !app.scanning).clicked() {
            app.rebuild_review();
        }
        if widgets::pointer(ui.button("Refresh sizes")).clicked() {
            app.project_sizes.invalidate();
            refresh_cards(app);
        }
        if widgets::pointer(ui.button("Open Review")).clicked() {
            app.nav = Nav::Storage;
        }
        let paths = app.selected_projects.iter().cloned().collect::<Vec<_>>();
        disk_actions::toolbar(app, ui, &paths);
    });
    widgets::action_note(ui, app.action_message.as_deref());
    crate::disk_history::history_link(app, ui);
    if app.project_cards.is_empty() {
        ui.label(if app.scanning {
            "Discovering projects…"
        } else {
            "Rebuild Review or scan Explorer to discover projects."
        });
        return;
    }
    let measured = app
        .project_cards
        .iter()
        .filter(|card| card.bytes.is_some_and(|u| u.complete))
        .count();
    let failed = app
        .project_cards
        .iter()
        .filter(|card| card.size_error.is_some() || card.bytes.is_some_and(|u| u.errors > 0))
        .count();
    ui.label(
        RichText::new(format!(
            "Project folders: {measured}/{} measured · {failed} with read errors",
            app.project_cards.len()
        ))
        .small()
        .color(theme::muted(ui)),
    );
    sort_cards(&mut app.project_cards, app.project_sort);
    draw_table(app, ui);
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.project_sort;
    let group = app.project_group;
    let lines = table_lines(
        &app.project_cards,
        group,
        &app.collapsed_project_groups,
        sort,
    );
    let height = widgets::table_scroll_height(ui);
    let mut selection = app.selected_projects.clone();
    let mut toggle = None;
    let mut scan = None;
    let mut action = None;
    widgets::table(ui, "projects-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(34.0).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(124.0).clip(true))
        .column(Column::exact(116.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(220.0).clip(true))
        .header(30.0, |mut header| {
            header.col(|ui| {
                let mut all = app
                    .project_cards
                    .iter()
                    .all(|card| selection.contains(&card.path));
                if widgets::check(ui, &mut all).changed() {
                    for card in &app.project_cards {
                        if all {
                            selection.insert(card.path.clone());
                        } else {
                            selection.remove(&card.path);
                        }
                    }
                }
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size on disk"));
            header.col(|ui| { ui.strong("Change"); });
            header.col(|ui| {
                ui.strong("Artifacts");
            });
        })
        .body(|body| {
            body.rows(44.0, lines.len(), |mut row| match &lines[row.index()] {
                Line::Group {
                    key,
                    title,
                    count,
                    bytes,
                    expanded,
                } => {
                    let members = app
                        .project_cards
                        .iter()
                        .filter(|card| match group {
                            ProjectGroup::Kind => card.group_kind == key,
                            _ => card.folder_key == *key,
                        })
                        .map(|card| card.path.clone())
                        .collect::<Vec<_>>();
                    let mut all = members.iter().all(|path| selection.contains(path));
                    row.set_selected(all);
                    row.col(|ui| {
                        if widgets::check(ui, &mut all).changed() {
                            for path in &members {
                                if all {
                                    selection.insert(path.clone());
                                } else {
                                    selection.remove(path);
                                }
                            }
                        }
                    });
                    row.col(|ui| {
                        if widgets::disclose(ui, *expanded, theme::accent()) {
                            toggle = Some(key.clone());
                        }
                        let response = ui.add(
                            egui::Label::new(RichText::new(format!("{title} · {count}")).strong())
                                .truncate()
                                .sense(egui::Sense::click()),
                        );
                        if response.clicked() {
                            toggle = Some(key.clone());
                        }
                    });
                    row.col(|ui| {
                        ui.label(if group == ProjectGroup::Kind {
                            "Kind"
                        } else {
                            "Folder"
                        });
                    });
                    row.col(|ui| {
                        ui.label(size_caption(*bytes, None))
                            .on_hover_text("Total of measured project folders in this group, counting nested projects once. A >= value is incomplete while other projects are still being measured or could not be read.");
                    });
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    if group == ProjectGroup::Parent {
                        disk_actions::menu(&row.response(), &PathBuf::from(key), &mut action);
                    }
                }
                Line::Project(index) => {
                    let card = &app.project_cards[*index];
                    let mut selected = selection.contains(&card.path);
                    row.set_selected(selected);
                    row.col(|ui| {
                        widgets::check(ui, &mut selected);
                    });
                    row.col(|ui| {
                        if group != ProjectGroup::None {
                            ui.add_space(18.0);
                        }
                        let response = ui
                            .vertical(|ui| {
                                ui.add(
                                    egui::Label::new(RichText::new(card.name()).size(15.0))
                                        .truncate(),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(card.path.display().to_string())
                                            .size(11.5)
                                            .color(theme::muted(ui)),
                                    )
                                    .truncate(),
                                );
                            })
                            .response
                            .interact(egui::Sense::click())
                            .on_hover_text(card.path.display().to_string());
                        if response.clicked() {
                            selected = !selected;
                        }
                        if response.double_clicked() {
                            scan = Some(card.path.clone());
                        }
                    });
                    row.col(|ui| {
                        ui.label(&card.kinds);
                    });
                    row.col(|ui| {
                        let note = match (&card.bytes, &card.size_error) {
                            (_, Some(error)) => format!("Could not read the project folder: {error}"),
                            (Some(usage), _) => format!("Whole project folder: {} files, {} read errors. Includes source, dependencies, hidden and generated files.", usage.files, usage.errors),
                            _ => "Measuring the whole project folder in the background. Artifact size is shown separately.".into(),
                        };
                        ui.label(size_caption(card.bytes, card.size_error.as_deref())).on_hover_text(note);
                        if app.project_sizes.is_cached(&card.path) { ui.label(RichText::new("Saved").small().color(theme::muted(ui))); }
                    });
                    row.col(|ui| {
                        if card.bytes.is_some() { crate::scan_history::trend(ui, &app.scan_history, crate::scan_history::Source::Projects, &card.path); }
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(&card.artifacts).truncate())
                            .on_hover_text(&card.artifacts);
                    });
                    if selected {
                        selection.insert(card.path.clone());
                    } else {
                        selection.remove(&card.path);
                    }
                    disk_actions::menu(&row.response(), &card.path, &mut action);
                }
            });
        });
    app.project_sort = sort;
    app.selected_projects = selection;
    if let Some(key) = toggle
        && !app.collapsed_project_groups.remove(&key)
    {
        app.collapsed_project_groups.insert(key);
    }
    if let Some(action) = action {
        app.path_action(action);
    }
    if let Some(path) = scan {
        app.scan_root = path.display().to_string();
        app.nav = Nav::Explorer;
        app.run_scan();
    }
}
