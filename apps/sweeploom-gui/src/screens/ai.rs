//! Inspect-first AI stores. Cache/log rows can be cleaned one at a time.

use std::time::SystemTime;

use eframe::egui::{self, RichText};
use egui_extras::Column;
use sweeploom_ai::ContextAdvice;
use sweeploom_core::DeletionStrategy;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, short_path};
use crate::nav::Nav;
use crate::sort::{Col, header_cell};
use crate::theme;
use crate::widgets::{self, page_title, table_scroll_height};

use super::ai_rows::{self, AiGroup, Line};

pub fn ui_ai(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "AI",
        "Select rows to inspect in Finder. Clean regenerable caches/logs or explicitly move history and generated media to Trash. Credentials, databases and managed worktrees remain protected.",
    );
    toolbar(app, ui);
    if app.ai_offers.is_none() && !app.ai_listing {
        app.run_ai_listing();
    }
    if app.ai_listing {
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(18.0, 18.0), |ui| {
                ui.spinner();
            });
            ui.label("Sizing AI stores…");
        });
    }
    if app.ai_offers.as_ref().is_some_and(Vec::is_empty) {
        ui.label("No local AI session stores were found under the home directory.");
        return;
    }
    draw_table(app, ui);
}

fn toolbar(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    crate::disk_history::history_link(app, ui);
    ui.add_space(theme::SM);
    widgets::toolbar(ui, |ui| {
        let mut group = app.ai_group;
        ui.label(RichText::new("Group by").size(13.0).color(theme::muted(ui)));
        if widgets::segmented(
            ui,
            &mut group,
            &[
                (AiGroup::Tool, AiGroup::Tool.label()),
                (AiGroup::Category, AiGroup::Category.label()),
                (AiGroup::None, AiGroup::None.label()),
            ],
        ) {
            app.ai_group = group;
        }
        ui.add(egui::Separator::default().vertical().spacing(theme::MD));
        if widgets::button(ui, "Refresh listing", !app.ai_listing).clicked() {
            app.run_ai_listing();
        }
        let paths = app
            .ai_offers
            .iter()
            .flatten()
            .flat_map(|offer| &offer.entries)
            .filter(|entry| entry.selected)
            .map(|entry| entry.candidate.path.clone())
            .collect::<Vec<_>>();
        if widgets::button(ui, "Open Review", true).clicked() {
            app.nav = Nav::Storage;
        }
        crate::disk_actions::toolbar(app, ui, &paths);
    });
    widgets::action_note(ui, app.action_message.as_deref());
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.ai_sort;
    let group = app.ai_group;
    let collapsed = app.collapsed_ai_groups.clone();
    let mut toggle = None;
    let mut action = None;
    {
        let Some(offers) = app.ai_offers.as_mut() else {
            return;
        };
        let lines = ai_rows::table_lines(offers, sort, group);
        let visible = ai_rows::visible_lines(&lines, &collapsed);
        let row_count = visible.len();
        let height = table_scroll_height(ui);
        crate::widgets::table(ui, "ai-grid")
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .min_scrolled_height(height)
            .max_scroll_height(height)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(34.0).clip(true).resizable(false))
            .column(Column::remainder().at_least(140.0).clip(true))
            .column(Column::exact(80.0).clip(true))
            .column(Column::exact(72.0).clip(true))
            .column(Column::exact(100.0).clip(true))
            .column(Column::exact(56.0).clip(true))
            .column(Column::exact(96.0).clip(true))
            .header(30.0, |mut header| {
                header.col(|ui| {
                    let mut all = offers
                        .iter()
                        .flat_map(|offer| &offer.entries)
                        .all(|entry| entry.selected);
                    if widgets::check(ui, &mut all).changed() {
                        for entry in offers.iter_mut().flat_map(|offer| &mut offer.entries) {
                            entry.selected = all;
                        }
                    }
                });
                header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
                header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
                header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
                header.col(|ui| {
                    ui.strong("Change");
                });
                header.col(|ui| header_cell(ui, &mut sort, Col::Procs, "Files"));
                header.col(|ui| header_cell(ui, &mut sort, Col::Safety, "Policy"));
            })
            .body(|body| {
                body.rows(widgets::TABLE_ROW, row_count, |mut row| {
                    match visible.get(row.index()) {
                        Some(line @ Line::Group { key, .. }) => {
                            let mut members = Vec::new();
                            let mut inside = false;
                            for candidate_line in &lines {
                                match candidate_line {
                                    Line::Group {
                                        key: candidate_key, ..
                                    } => inside = candidate_key == key,
                                    Line::Item(o, e) if inside => members.push((*o, *e)),
                                    _ => {}
                                }
                            }
                            let mut selected = !members.is_empty()
                                && members.iter().all(|&(o, e)| offers[o].entries[e].selected);
                            let before = selected;
                            fill_group(
                                &mut row,
                                line,
                                !collapsed.contains(key),
                                &mut toggle,
                                &mut selected,
                            );
                            if before != selected {
                                for (o, e) in members {
                                    offers[o].entries[e].selected = selected;
                                }
                            }
                        }
                        Some(Line::Item(offer, entry)) => {
                            fill_item(
                                offers,
                                &mut row,
                                *offer,
                                *entry,
                                &mut action,
                                &app.scan_history,
                            );
                        }
                        None => {}
                    }
                });
            });
    }
    app.ai_sort = sort;
    if let Some(action) = action {
        app.path_action(action);
    }
    if let Some(key) = toggle
        && !app.collapsed_ai_groups.remove(&key)
    {
        app.collapsed_ai_groups.insert(key);
    }
}

fn fill_group(
    row: &mut egui_extras::TableRow<'_, '_>,
    line: &Line,
    expanded: bool,
    toggle: &mut Option<String>,
    selected: &mut bool,
) {
    let Line::Group {
        key,
        title,
        bytes,
        files,
        count,
    } = line
    else {
        return;
    };
    row.set_selected(*selected);
    row.col(|ui| {
        widgets::check(ui, selected);
    });
    row.col(|ui| {
        crate::brand::show_tool(ui, title, 16.0);
        if crate::widgets::disclose(ui, expanded, crate::theme::accent()) {
            *toggle = Some(key.to_owned());
        }
        if ui
            .add(
                egui::Label::new(RichText::new(format!("{title} · {count}")).strong())
                    .sense(egui::Sense::click()),
            )
            .clicked()
        {
            *toggle = Some(key.to_owned());
        }
    });
    row.col(|_ui| {});
    row.col(|ui| {
        ui.label(format_bytes(*bytes));
    });
    row.col(|_ui| {});
    row.col(|ui| {
        ui.label(files.to_string());
    });
    row.col(|_ui| {});
}

fn fill_item(
    offers: &mut [sweeploom_ai::AiOffer],
    row: &mut egui_extras::TableRow<'_, '_>,
    offer: usize,
    entry: usize,
    action: &mut Option<crate::disk_actions::PathAction>,
    history: &crate::scan_history::ScanHistory,
) {
    let Some(item) = offers.get(offer).and_then(|item| item.entries.get(entry)) else {
        return;
    };
    let name = item.relative.clone();
    let class = item.class.label();
    let size = format_bytes(
        item.candidate
            .allocated_bytes
            .unwrap_or(item.candidate.logical_bytes),
    );
    let candidate_path = item.candidate.path.clone();
    let files = item.candidate.file_count;
    let path = short_path(&item.candidate.path);
    let policy = if item.candidate.deletion == DeletionStrategy::InspectOnly {
        if item.candidate.safety.is_blocked() {
            "blocked"
        } else {
            "manual Trash"
        }
    } else {
        "cleanable"
    };
    let hover = item.class.is_prompt_context().then(|| {
        let tokens = sweeploom_ai::estimated_prompt_tokens_with_files(
            item.class,
            &item.relative,
            item.candidate.logical_bytes,
            item.candidate.file_count,
        );
        let idle = item
            .candidate
            .activity
            .latest_any_modified
            .and_then(|stamp| SystemTime::now().duration_since(stamp).ok());
        match sweeploom_ai::advise_context(&item.relative, item.class, idle) {
            ContextAdvice::SuggestPark => format!(
                "~{tokens} always-on tokens. Stale skill/plugin index (30+ days idle). Review before parking — SweepLoom will not disable it."
            ),
            ContextAdvice::Keep | ContextAdvice::LeaveAlone => format!(
                "~{tokens} always-on tokens (capped; archive/history not counted). Keep."
            ),
        }
    });
    let mut selected = item.selected;
    row.set_selected(selected);
    row.col(|ui| {
        widgets::check(ui, &mut selected);
    });
    row.col(|ui| {
        ui.add(
            egui::Label::new(RichText::new(name))
                .truncate()
                .selectable(false)
                .sense(egui::Sense::click()),
        )
        .on_hover_text(path)
        .clicked()
        .then(|| selected = !selected);
    });
    row.col(|ui| {
        let kind = ui.label(class);
        if let Some(hover) = hover.as_deref() {
            kind.on_hover_text(hover);
        }
    });
    row.col(|ui| {
        ui.label(size);
    });
    row.col(|ui| {
        crate::scan_history::trend(
            ui,
            history,
            crate::scan_history::Source::Ai,
            &candidate_path,
        );
    });
    row.col(|ui| {
        ui.label(files.to_string());
    });
    row.col(|ui| {
        ui.label(RichText::new(policy).color(theme::muted(ui)));
    });
    crate::disk_actions::menu(&row.response(), &candidate_path, action);
    if let Some(item) = offers
        .get_mut(offer)
        .and_then(|offer| offer.entries.get_mut(entry))
    {
        item.selected = selected;
    }
}
