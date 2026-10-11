//! Projects with group selection and shared file actions.
use super::project_rows::{Line, ProjectGroup, refresh_cards, sort_cards, table_lines};
use crate::{
    app::SweepLoomApp,
    disk_actions,
    nav::Nav,
    sort::{Col, header_cell},
    theme, widgets,
};
use eframe::egui::{self, RichText};
use egui_extras::Column;

#[path = "projects_table.rs"]
mod table;

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
        if widgets::button(ui, "Project output", true)
            .on_hover_text("Open Cleanup → Project output")
            .clicked()
        {
            app.go(crate::nav::Dest::ProjectOutput);
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
            "Rebuild Cleanup → Project output or scan Explorer to discover projects."
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
            header.col(|ui| table::select_all(ui, &app.project_cards, &mut selection));
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size on disk"));
            header.col(|ui| {
                ui.strong("Change");
            });
            header.col(|ui| {
                ui.strong("Artifacts");
            });
        })
        .body(|body| {
            let mut state = table::RowState {
                selection: &mut selection,
                toggle: &mut toggle,
                scan: &mut scan,
                action: &mut action,
            };
            body.rows(
                widgets::TABLE_ROW_TALL,
                lines.len(),
                |mut row| match &lines[row.index()] {
                    Line::Group {
                        key,
                        title,
                        count,
                        bytes,
                        expanded,
                    } => {
                        let group_line = table::GroupLine {
                            key,
                            title,
                            count: *count,
                            bytes: *bytes,
                            expanded: *expanded,
                        };
                        table::group_row(app, &mut row, group, &group_line, &mut state);
                    }
                    Line::Project(index) => {
                        table::project_row(app, &mut row, group, *index, &mut state);
                    }
                },
            );
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
