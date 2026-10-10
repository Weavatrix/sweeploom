//! Storage scan and Folder Inspector tree.

use eframe::egui::{self, RichText};
use egui_extras::Column;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, header_cell};
use crate::widgets::{page_title, table_scroll_height};

use super::explorer_rows::{self, Line};

pub fn ui_storage(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Explorer",
        "Folder inspector. Symlinks are not followed. Scan runs in the background. Use the arrow to expand, checkbox to select, or right-click for actions. Double-click to scan a folder. Size shows allocated disk blocks.",
    );
    ui.horizontal(|ui| {
        ui.label("Root");
        ui.add(
            egui::TextEdit::singleline(&mut app.scan_root)
                .desired_width(280.0)
                .clip_text(true),
        );
        let walking = app.scan_rx.is_some();
        let scan = if walking { "Scanning…" } else { "Scan" };
        if crate::widgets::pointer(ui.add_enabled(!walking, egui::Button::new(scan))).clicked() {
            app.run_scan();
        }
    });
    let mut saved_index = None;
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_id_salt("explorer-saved-scans")
            .selected_text("Previous scans…")
            .show_ui(ui, |ui| {
                for (index, saved) in app.scan_history.scans().iter().enumerate().rev() {
                    let caption = format!(
                        "{} · {}",
                        crate::scan_history::timestamp(saved.at),
                        saved.report.root.display()
                    );
                    if ui
                        .add_enabled(app.scan_rx.is_none(), egui::Button::new(caption))
                        .clicked()
                    {
                        saved_index = Some(index);
                    }
                }
                if app.scan_history.scans().is_empty() {
                    ui.label("No saved scans yet");
                }
            });
        if let Some(at) = app.inventory_at {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    if app.inventory_cached {
                        "Saved scan:"
                    } else {
                        "Scanned:"
                    },
                    crate::scan_history::timestamp(at)
                ))
                .small()
                .color(crate::theme::muted(ui)),
            );
        }
    });
    if let Some(index) = saved_index {
        crate::disk_history::open_scan(app, index);
    }
    crate::disk_history::history_link(app, ui);
    ui.horizontal_wrapped(|ui| {
        let paths = app.selected_explorer.iter().cloned().collect::<Vec<_>>();
        crate::disk_actions::toolbar(app, ui, &paths);
    });
    crate::widgets::action_note(ui, app.action_message.as_deref());
    if let Some(report) = &app.inventory {
        ui.label(format!(
            "Showing: {}{}",
            report.root.display(),
            if report.tree.incomplete {
                " (partial scan)"
            } else {
                ""
            }
        ));
    }
    if app.scan_rx.is_some() {
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(18.0, 18.0), |ui| {
                ui.spinner();
            });
            ui.label(
                RichText::new(format!(
                    "{} entries · {}",
                    app.scan_entries,
                    format_bytes(app.scan_bytes)
                ))
                .color(crate::theme::muted(ui)),
            );
            let hint = if app.scan_hint.is_empty() {
                "walking…".to_owned()
            } else {
                std::path::Path::new(&app.scan_hint)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&app.scan_hint)
                    .to_owned()
            };
            ui.add(egui::Label::new(RichText::new(hint).color(crate::theme::muted(ui))).truncate());
        });
    } else if let Some(report) = &app.inventory {
        ui.label(
            RichText::new(format!(
                "{} entries · {} projects · {} allocated estimate",
                report.entries,
                report.projects.len(),
                format_bytes(report.tree.disk_bytes()),
            ))
            .color(crate::theme::muted(ui)),
        );
    }
    if let Some(report) = &app.inventory
        && report.errors > 0
    {
        ui.label(format!(
            "{} paths could not be read. Folder sizes are partial estimates.",
            report.errors
        ));
    }
    if let Some(error) = &app.inventory_error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    if app.inventory.is_some() {
        folder_table(app, ui);
    } else if app.scan_rx.is_none() {
        ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
    }
}

fn folder_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.explorer_sort;
    let expanded = app.expanded_explorer.clone();
    let lines = {
        let Some(report) = &app.inventory else {
            return;
        };
        explorer_rows::visible_lines(&report.tree, sort, &expanded)
    };
    let row_count = lines.len();
    let height = table_scroll_height(ui);
    let mut toggle = None;
    let mut picked = None;
    let mut action = None;
    let mut selection = app.selected_explorer.clone();
    crate::widgets::table(ui, "explorer-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(36.0).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(110.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(64.0).clip(true))
        .header(32.0, |mut header| {
            header.col(|ui| {
                let mut all =
                    !lines.is_empty() && lines.iter().all(|line| selection.contains(&line.path));
                if crate::widgets::check(ui, &mut all).changed() {
                    for line in &lines {
                        if all {
                            selection.insert(line.path.clone());
                        } else {
                            selection.remove(&line.path);
                        }
                    }
                }
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size on disk"));
            header.col(|ui| {
                ui.strong("Change");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Category"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Procs, "Files"));
        })
        .body(|body| {
            body.rows(crate::widgets::TABLE_ROW, row_count, |mut row| {
                let Some(line) = lines.get(row.index()) else {
                    return;
                };
                let mut selected = selection.contains(&line.path);
                let (disclosed, clicked, double) = fill_line(
                    &mut row,
                    line,
                    &mut selected,
                    &app.scan_history,
                    app.inventory_at,
                );
                if disclosed && line.has_children {
                    toggle = Some(explorer_rows::path_key(&line.path));
                }
                if clicked {
                    selected = !selected;
                }
                if double && !line.is_file {
                    picked = Some(line.path.display().to_string());
                }
                if selected {
                    selection.insert(line.path.clone());
                } else {
                    selection.remove(&line.path);
                }
                crate::disk_actions::menu(&row.response(), &line.path, &mut action);
            });
        });
    app.explorer_sort = sort;
    app.selected_explorer = selection;
    if let Some(action) = action {
        app.path_action(action);
    }
    if let Some(key) = toggle
        && !app.expanded_explorer.remove(&key)
    {
        app.expanded_explorer.insert(key);
    }
    if let Some(path) = picked {
        app.scan_root = path;
        app.run_scan();
    }
}

fn fill_line(
    row: &mut egui_extras::TableRow<'_, '_>,
    line: &Line,
    selected: &mut bool,
    history: &crate::scan_history::ScanHistory,
    at: Option<u64>,
) -> (bool, bool, bool) {
    let name = line.name.clone();
    let size = format_bytes(line.bytes);
    let files = line.files.to_string();
    let category = line.category.label();
    let depth = line.depth;
    let glyph = category_glyph(line.category);
    let open = line.expanded;
    let can_open = line.has_children;
    let mut disclosed = false;
    let mut clicked = false;
    let mut double = false;
    row.set_selected(*selected);
    row.col(|ui| {
        crate::widgets::check(ui, selected);
    });
    row.col(|ui| {
        ui.add_space(depth as f32 * 12.0);
        if can_open {
            disclosed = crate::widgets::disclose(ui, open, crate::theme::accent());
        } else {
            ui.add_space(11.0);
        }
        crate::icons::show(ui, glyph, 14.0, crate::theme::accent());
        let response = ui
            .add(
                egui::Label::new(RichText::new(name).size(15.0))
                    .truncate()
                    .sense(egui::Sense::click()),
            )
            .on_hover_text(line.path.display().to_string());
        clicked = response.clicked();
        double = response.double_clicked();
    });
    row.col(|ui| {
        ui.label(size).on_hover_text(format!("Size on disk: {}\nLogical size: {}\nSparse files and shared APFS blocks can differ from space freed.", format_bytes(line.bytes), format_bytes(line.logical_bytes)));
    });
    row.col(|ui| {
        if at.is_some() {
            crate::scan_history::trend_at(
                ui,
                history,
                crate::scan_history::Source::Explorer,
                &line.path,
                at,
            );
        } else {
            ui.label(
                RichText::new("Scanning…")
                    .small()
                    .color(crate::theme::muted(ui)),
            );
        }
    });
    row.col(|ui| {
        ui.label(category);
    });
    row.col(|ui| {
        ui.label(files);
    });
    (disclosed, clicked, double)
}

fn category_glyph(_category: sweeploom_storage::PathCategory) -> crate::icons::Glyph {
    crate::icons::Glyph::Explorer
}
