//! Explorer header: scan root, saved scans, actions and scan status.

use eframe::egui::{self, RichText};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub(super) fn header(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut saved_index = None;
    crate::widgets::toolbar(ui, |ui| {
        ui.label(
            RichText::new("Root")
                .size(13.0)
                .color(crate::theme::muted(ui)),
        );
        ui.add(
            egui::TextEdit::singleline(&mut app.scan_root)
                .desired_width(360.0)
                .margin(egui::Margin::symmetric(8, 5))
                .clip_text(true),
        );
        let walking = app.scan_rx.is_some();
        let scan = if walking { "Scanning…" } else { "Scan" };
        let clicked = if walking {
            crate::widgets::button(ui, scan, false).clicked()
        } else {
            crate::widgets::primary_button(ui, scan).clicked()
        };
        if clicked {
            app.run_scan();
        }
        crate::folder_access_ui::explorer_picker(app, ui);
        ui.add(
            egui::Separator::default()
                .vertical()
                .spacing(crate::theme::MD),
        );
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
    status(app, ui);
}

fn status(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    crate::disk_history::history_link(app, ui);
    ui.add_space(crate::theme::SM);
    crate::widgets::toolbar(ui, |ui| {
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
}
