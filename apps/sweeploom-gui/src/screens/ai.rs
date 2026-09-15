//! Inspect-first AI stores. Cache/log rows can be cleaned one at a time.

use std::time::SystemTime;

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_ai::ContextAdvice;
use sweeploom_core::DeletionStrategy;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, short_path};
use crate::nav::Nav;
use crate::sort::{Col, header_cell};
use crate::theme;
use crate::widgets::{self, page_title, pointer, table_scroll_height};

use super::ai_rows::{self, AiGroup, Line};

pub fn ui_ai(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "AI",
        "Inspect-first. Credentials and SQLite stay blocked. Cache/log rows can be cleaned one at a time — never auto-selected. File contents are not opened.",
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
    ui.horizontal_wrapped(|ui| {
        if pointer(ui.button("Refresh listing")).clicked() {
            app.run_ai_listing();
        }
        if pointer(ui.button("Clean selected caches")).clicked() {
            app.apply_ai_clean();
        }
        if pointer(ui.button("Open Review")).clicked() {
            app.nav = Nav::Storage;
        }
        ui.label("Group by");
        for mode in [AiGroup::Tool, AiGroup::Category, AiGroup::None] {
            if pointer(ui.selectable_label(app.ai_group == mode, mode.label())).clicked() {
                app.ai_group = mode;
            }
        }
    });
    widgets::action_note(ui, app.action_message.as_deref());
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.ai_sort;
    let group = app.ai_group;
    let collapsed = app.collapsed_ai_groups.clone();
    let mut toggle = None;
    {
        let Some(offers) = app.ai_offers.as_mut() else {
            return;
        };
        let lines = ai_rows::table_lines(offers, sort, group);
        let visible = ai_rows::visible_lines(&lines, &collapsed);
        let row_count = visible.len();
        let height = table_scroll_height(ui);
        TableBuilder::new(ui)
            .id_salt("ai-grid")
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .min_scrolled_height(height)
            .max_scroll_height(height)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(36.0).clip(true).resizable(false))
            .column(Column::remainder().at_least(140.0).clip(true))
            .column(Column::exact(80.0).clip(true))
            .column(Column::exact(72.0).clip(true))
            .column(Column::exact(56.0).clip(true))
            .column(Column::exact(96.0).clip(true))
            .header(32.0, |mut header| {
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
                header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
                header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
                header.col(|ui| {
                    ui.strong("Files");
                });
                header.col(|ui| {
                    ui.strong("Policy");
                });
            })
            .body(|body| {
                body.rows(widgets::TABLE_ROW, row_count, |mut row| {
                    match visible.get(row.index()) {
                        Some(line @ Line::Group { key, .. }) => {
                            fill_group(&mut row, line, !collapsed.contains(key), &mut toggle);
                            if row.response().clicked() {
                                toggle = Some(key.clone());
                            }
                        }
                        Some(Line::Item(offer, entry)) => {
                            fill_item(offers, &mut row, *offer, *entry);
                        }
                        None => {}
                    }
                });
            });
    }
    app.ai_sort = sort;
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
    row.col(|_ui| {});
    row.col(|ui| {
        crate::brand::show_tool(ui, title, 16.0);
        if crate::widgets::disclose(ui, expanded, crate::theme::accent()) {
            *toggle = Some(key.to_owned());
        }
        ui.label(RichText::new(format!("{title}  ·  {count}")).strong());
    });
    row.col(|_ui| {});
    row.col(|ui| {
        ui.label(format_bytes(*bytes));
    });
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
) {
    let clean = ai_rows::can_clean(&offers[offer], entry);
    let Some(item) = offers.get(offer).and_then(|item| item.entries.get(entry)) else {
        return;
    };
    let name = item.relative.clone();
    let class = item.class.label();
    let size = format_bytes(item.candidate.logical_bytes);
    let files = item.candidate.file_count;
    let path = short_path(&item.candidate.path);
    let policy = if item.candidate.deletion == DeletionStrategy::InspectOnly {
        if item.candidate.safety.is_blocked() {
            "blocked"
        } else {
            "inspect only"
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
    row.col(|ui| {
        if clean {
            if ui.checkbox(&mut selected, "").changed()
                && let Some(item) = offers
                    .get_mut(offer)
                    .and_then(|item| item.entries.get_mut(entry))
            {
                item.selected = selected;
            }
        } else {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, ""));
        }
    });
    row.col(|ui| {
        ui.add(
            egui::Label::new(RichText::new(name))
                .truncate()
                .selectable(false),
        )
        .on_hover_text(path);
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
        ui.label(files.to_string());
    });
    row.col(|ui| {
        ui.label(RichText::new(policy).color(theme::muted(ui)));
    });
}

impl SweepLoomApp {
    /// Apply checked cache/log rows through CleanPlan. Secrets never enter the plan.
    pub fn apply_ai_clean(&mut self) {
        let Some(offers) = &self.ai_offers else {
            return;
        };
        let selected: Vec<_> = offers
            .iter()
            .flat_map(|offer| offer.entries.iter())
            .filter(|entry| entry.selected && entry.class.can_clean())
            .filter(|entry| {
                entry.candidate.deletion != DeletionStrategy::InspectOnly
                    && !entry.candidate.safety.is_blocked()
            })
            .map(|entry| entry.candidate.clone())
            .collect();
        if selected.is_empty() {
            self.action_message =
                Some("Nothing selected. Credentials and SQLite cannot be checked.".to_owned());
            return;
        }
        if self.apply_rx.is_some() {
            self.action_message = Some("A cleanup is already running.".to_owned());
            return;
        }
        let processes = self
            .snapshot
            .as_ref()
            .map(|item| item.processes.clone())
            .unwrap_or_default();
        self.ai_offers = None;
        self.action_message = Some("Applying AI cleanup in the background…".to_owned());
        self.apply_rx = Some(crate::scan_job::spawn_apply(selected, processes));
    }
}
