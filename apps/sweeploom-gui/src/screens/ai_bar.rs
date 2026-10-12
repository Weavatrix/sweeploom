//! AI toolbar (grouping, refresh, disk actions) and group-row selection.

use eframe::egui::{self, RichText};

use super::super::ai_rows::{AiGroup, Line};
use crate::app::SweepLoomApp;
use crate::theme;
use crate::widgets;

pub(super) fn toolbar(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    if crate::edition::STORE {
        let stores = crate::folder_access::missing(&app.locations.home, crate::folder_access::AI);
        if !stores.is_empty() {
            crate::folder_access_ui::prompt(
                app,
                ui,
                "AI tool folders need access",
                "Claude, Codex, Cursor and other agents keep sessions, logs and caches in these folders. Grant them to size and clean those stores.",
                &stores,
            );
        }
    }
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
        if widgets::button(ui, "Project output", true)
            .on_hover_text("Open Cleanup → Project output")
            .clicked()
        {
            app.go(crate::nav::Dest::ProjectOutput);
        }
        crate::disk_actions::toolbar(app, ui, &paths);
    });
    widgets::action_note(ui, app.action_message.as_deref());
}

/// Group row whose checkbox selects every member entry.
pub(super) fn group_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    lines: &[Line],
    line: &Line,
    key: &str,
    offers: &mut [sweeploom_ai::AiOffer],
    open: bool,
    toggle: &mut Option<String>,
) {
    let mut members = Vec::new();
    let mut inside = false;
    for candidate_line in lines {
        match candidate_line {
            Line::Group {
                key: candidate_key, ..
            } => inside = candidate_key == key,
            Line::Item(o, e) if inside => members.push((*o, *e)),
            _ => {}
        }
    }
    let mut selected =
        !members.is_empty() && members.iter().all(|&(o, e)| offers[o].entries[e].selected);
    let before = selected;
    super::fill_group(row, line, open, toggle, &mut selected);
    if before != selected {
        for (o, e) in members {
            offers[o].entries[e].selected = selected;
        }
    }
}
