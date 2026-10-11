//! One Project output (Review) table row.

use super::*;

pub(super) fn fill_review_row(
    rows: &mut [ReviewRow],
    row: &mut egui_extras::TableRow<'_, '_>,
    index: usize,
    action: &mut Option<crate::disk_actions::PathAction>,
    history: &crate::scan_history::ScanHistory,
) {
    let Some(item) = rows.get(index) else {
        return;
    };
    let blocked = item.candidate.safety.is_blocked();
    let inspect_only = item.candidate.deletion == DeletionStrategy::InspectOnly;
    let name = candidate_caption(&item.title, &item.candidate.path);
    let path = short_path(&item.candidate.path);
    let size = format_bytes(
        item.candidate
            .allocated_bytes
            .unwrap_or(item.candidate.logical_bytes),
    );
    let rebuild = item.candidate.rebuild.cost.label().to_owned();
    let safety = if inspect_only && !blocked {
        if item.candidate.user_policy == sweeploom_core::UserPolicy::NeverClean {
            "Protected store — inspect".to_owned()
        } else {
            "Manual Trash".to_owned()
        }
    } else {
        safety_text(&item.candidate.safety)
    };
    let mut selected = item.selected;
    let candidate_path = item.candidate.path.clone();
    row.set_selected(selected);
    row.col(|ui| {
        crate::widgets::check(ui, &mut selected);
    });
    row.col(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add(
                egui::Label::new(RichText::new(name).size(15.0))
                    .truncate()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(RichText::new(&path).size(12.5).color(theme::muted(ui)))
                    .truncate()
                    .selectable(false),
            );
        })
        .response
        .interact(egui::Sense::click())
        .on_hover_text(path)
        .clicked()
        .then(|| selected = !selected);
    });
    row.col(|ui| {
        ui.label(&size);
    });
    row.col(|ui| {
        crate::scan_history::trend(
            ui,
            history,
            crate::scan_history::Source::Review,
            &candidate_path,
        );
    });
    row.col(|ui| {
        ui.label(&rebuild);
    });
    row.col(|ui| {
        let tone = if blocked {
            theme::Tone::Warn
        } else if safety.starts_with("Safe") {
            theme::Tone::Ok
        } else {
            theme::Tone::Muted
        };
        widgets::pill(ui, &safety, tone);
    });
    crate::disk_actions::menu(&row.response(), &candidate_path, action);
    if let Some(item) = rows.get_mut(index) {
        item.selected = selected;
    }
}
