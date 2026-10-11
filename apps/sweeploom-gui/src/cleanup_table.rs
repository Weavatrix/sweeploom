use super::super::*;
use crate::{
    format::format_bytes,
    sort::{Col, header_cell},
    widgets,
};
use eframe::egui;

pub(super) fn draw(
    app: &mut SweepLoomApp,
    ui: &mut egui::Ui,
    processes: Option<&[sweeploom_core::ProcessSnapshot]>,
) {
    let pane = app.native_cleanup.pane;
    let state = app.native_cleanup.active_mut();
    let Some(listing) = &mut state.listing else {
        return;
    };
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.filter)
                .id_salt(("cleanup-filter", pane))
                .hint_text(
                    egui::RichText::new("Filter by name or path…").color(crate::theme::muted(ui)),
                )
                .margin(egui::Margin::symmetric(8, 5))
                .desired_width(280.0),
        );
        let measured = listing
            .items
            .iter()
            .filter(|item| item.bytes.is_some())
            .count();
        widgets::caption(
            ui,
            format!("{} items · {measured} measured", listing.items.len()),
        );
    });
    ui.add_space(crate::theme::XS);
    let mut sort = state.sort;
    let indices = visible_indices(listing, sort, &state.filter);
    let height = widgets::table_scroll_height(ui);
    let mut reveal = None;
    widgets::table(ui, &format!("native-cleanup-grid-{pane:?}"))
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::exact(34.0).resizable(false))
        .column(egui_extras::Column::remainder().at_least(220.0).clip(true))
        .column(egui_extras::Column::exact(118.0).clip(true))
        .column(egui_extras::Column::exact(104.0).clip(true))
        .column(egui_extras::Column::exact(64.0).clip(true))
        .column(egui_extras::Column::exact(96.0).clip(true))
        .column(
            egui_extras::Column::initial(300.0)
                .at_least(160.0)
                .clip(true),
        )
        .header(30.0, |mut header| {
            header.col(|ui| {
                let mut all = !indices.is_empty()
                    && indices.iter().all(|&index| listing.items[index].selected);
                if widgets::check(ui, &mut all).changed() {
                    for &index in &indices {
                        listing.items[index].selected = all;
                    }
                }
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Safety, "Kind"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size on disk"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Procs, "Files"));
            header.col(|ui| {
                ui.strong("Change");
            });
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Status / action"));
        })
        .body(|body| {
            body.rows(widgets::TABLE_ROW_TALL, indices.len(), |mut row| {
                let item = &mut listing.items[indices[row.index()]];
                if let Some(path) = fill_row(&mut row, item, &app.scan_history, processes) {
                    reveal = Some(path);
                }
            });
        });
    state.sort = sort;
    if let Some(path) = reveal {
        app.reveal_paths(&[path]);
    }
}

fn visible_indices(listing: &mut Listing, sort: Sort, filter: &str) -> Vec<usize> {
    listing.items.sort_by(|a, b| {
        let unknown = match sort.col {
            Col::Size => Some((a.bytes.is_none(), b.bytes.is_none())),
            Col::Procs => Some((a.usage.is_none(), b.usage.is_none())),
            _ => None,
        };
        if let Some((a, b)) = unknown
            && a != b
        {
            return a.cmp(&b);
        }
        let order = match sort.col {
            Col::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            Col::Status => a.status.cmp(&b.status),
            Col::Safety => a.kind_label().cmp(b.kind_label()),
            Col::Procs => a
                .usage
                .map(|usage| usage.files)
                .cmp(&b.usage.map(|usage| usage.files)),
            _ => a.bytes.cmp(&b.bytes),
        }
        .then(a.id.cmp(&b.id));
        if sort.desc { order.reverse() } else { order }
    });
    let query = filter.to_lowercase();
    listing
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            query.is_empty()
                || format!("{} {} {}", item.name, item.id, item.status)
                    .to_lowercase()
                    .contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
#[path = "cleanup_table_tests.rs"]
mod tests;

fn fill_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    item: &mut Item,
    history: &crate::scan_history::ScanHistory,
    processes: Option<&[sweeploom_core::ProcessSnapshot]>,
) -> Option<PathBuf> {
    let mut reveal = None;
    row.set_selected(item.selected);
    row.col(|ui| {
        widgets::check(ui, &mut item.selected);
    });
    row.col(|ui| {
        let origin = item
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| item.id.clone());
        let response = widgets::two_line(ui, item.name.as_str(), &origin)
            .on_hover_text(&origin)
            .interact(egui::Sense::click());
        if response.clicked() {
            item.selected = !item.selected;
        }
        if let Some(path) = &item.path {
            response.context_menu(|ui| {
                if ui.button("Show in Finder").clicked() {
                    reveal = Some(path.clone());
                    ui.close();
                }
            });
        }
    });
    row.col(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        let color = super::kind_color(ui, item.kind_label(), 0);
        ui.painter().rect_filled(rect, 2.0, color);
        ui.add(egui::Label::new(item.kind_label()).truncate());
    });
    row.col(|ui| {
        let prefix = if item.usage.is_some_and(|usage| !usage.complete) {
            "≥ "
        } else {
            ""
        };
        ui.label(
            item.bytes
                .map(|bytes| format!("{prefix}{}", format_bytes(bytes)))
                .unwrap_or_else(|| "—".into()),
        )
        .on_hover_text(
            item.usage
                .map(|usage| {
                    format!(
                        "Allocated: {}\nLogical: {}\nRead errors: {}",
                        format_bytes(usage.bytes),
                        format_bytes(usage.logical_bytes),
                        usage.errors
                    )
                })
                .unwrap_or_else(|| {
                    "Tool-reported size; shared layers may reduce the space freed".into()
                }),
        );
    });
    row.col(|ui| {
        ui.label(
            item.usage
                .map(|usage| usage.files.to_string())
                .unwrap_or_else(|| "—".into()),
        );
    });
    row.col(|ui| {
        crate::scan_history::trend(
            ui,
            history,
            crate::scan_history::Source::Native,
            &item.history_key(),
        );
    });
    row.col(|ui| status_cell(ui, item, processes));
    reveal
}

fn status_cell(
    ui: &mut egui::Ui,
    item: &Item,
    processes: Option<&[sweeploom_core::ProcessSnapshot]>,
) {
    let status = if item.enabled && !item.cleanable(processes) {
        format!("In use or awaiting process evidence; {}", item.status)
    } else {
        item.status.clone()
    };
    let tone = if item.enabled && !item.cleanable(processes) {
        crate::theme::Tone::Caution
    } else if item.enabled {
        crate::theme::Tone::Neutral
    } else {
        crate::theme::Tone::Muted
    };
    ui.spacing_mut().item_spacing.x = 6.0;
    let rest = match verdict(&status) {
        Some((word, tone, rest)) => {
            widgets::pill(ui, word, tone);
            rest
        }
        None => status.as_str(),
    };
    ui.add(
        egui::Label::new(egui::RichText::new(rest).color(crate::theme::tone(ui, tone))).truncate(),
    )
    .on_hover_text(&status);
}

/// Leading verdict word ("Suggested · …") as a pill, so the column scans at a glance.
fn verdict(status: &str) -> Option<(&str, crate::theme::Tone, &str)> {
    use crate::theme::Tone;
    let (word, rest) = status.split_once(" · ")?;
    let tone = match word {
        "Suggested" | "Safe" | "Unused" => Tone::Ok,
        "Review" | "Stale" | "Old" => Tone::Caution,
        "Keep" | "In use" | "Protected" | "Booted" => Tone::Muted,
        _ => return None,
    };
    Some((word, tone, rest))
}
