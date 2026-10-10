//! Saved scans and path measurements, independent of live process history.

use crate::{app::SweepLoomApp, format::format_bytes, nav::Nav, scan_history, theme, widgets};
use eframe::egui::{self, RichText};
use egui_extras::Column;

pub(crate) fn history_link(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        if ui.small_button("Scan history").clicked() {
            app.nav = Nav::DiskHistory;
        }
        ui.label(
            RichText::new(
                "Measurements are saved locally. Change compares finished scans of the same kind.",
            )
            .small()
            .color(theme::muted(ui)),
        );
    });
    if let Some(error) = &app.scan_history.error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Scan history could not be saved or loaded: {error}"),
        );
    }
}

pub(crate) fn open_scan(app: &mut SweepLoomApp, index: usize) {
    if app.scan_rx.is_some() {
        return;
    }
    let Some(saved) = app.scan_history.scans().get(index).cloned() else {
        return;
    };
    app.scan_root = saved.report.root.display().to_string();
    app.scan_entries = saved.report.entries;
    app.scan_bytes = saved.report.tree.disk_bytes();
    app.inventory = Some(saved.report);
    app.inventory_at = Some(saved.at);
    app.inventory_cached = true;
    app.inventory_error = None;
    app.selected_explorer.clear();
    app.expanded_explorer.clear();
    app.nav = Nav::Explorer;
}

pub(crate) fn ui(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    widgets::page_title(
        ui,
        "Scan history",
        "Saved Explorer scans and folder measurements survive restarts. No measurements are invented for time before history was enabled.",
    );
    if let Some(error) = &app.scan_history.error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    let mut open = None;
    egui::CollapsingHeader::new(format!(
        "Saved Explorer scans · {}",
        app.scan_history.scans().len()
    ))
    .default_open(true)
    .show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("saved-explorer-scans")
            .max_height(145.0)
            .show(ui, |ui| {
                for (index, saved) in app.scan_history.scans().iter().enumerate().rev() {
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(app.scan_rx.is_none(), egui::Button::new("Open"))
                            .clicked()
                        {
                            open = Some(index);
                        }
                        ui.label(scan_history::timestamp(saved.at));
                        ui.label(format_bytes(saved.report.tree.disk_bytes()));
                        if saved.report.capped
                            || saved.report.errors > 0
                            || saved.report.tree.incomplete
                        {
                            ui.label("Partial");
                        }
                        ui.add(
                            egui::Label::new(saved.report.root.display().to_string()).truncate(),
                        )
                        .on_hover_text(saved.report.root.display().to_string());
                    });
                }
                if app.scan_history.scans().is_empty() {
                    ui.label("Complete a scan in Explorer to save its tree here.");
                }
            });
    });
    if let Some(index) = open {
        open_scan(app, index);
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Find folder / object");
        ui.add(egui::TextEdit::singleline(&mut app.disk_history_filter).desired_width(340.0));
        if ui.button("Clear").clicked() {
            app.disk_history_filter.clear();
        }
        ui.label(
            RichText::new("12 Explorer scans · 32 measurements per path")
                .small()
                .color(theme::muted(ui)),
        );
    });
    if let Some((source, path)) = &app.disk_history_selected
        && let Some(series) = app.scan_history.get(*source, path)
    {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(path.display().to_string()).strong()).truncate());
            ui.horizontal(|ui| {
                ui.label(source.label());
                scan_history::trend(ui, &app.scan_history, *source, path);
                let samples = series
                    .points
                    .iter()
                    .filter(|p| p.usage.complete && p.basis == series.latest().basis)
                    .map(|p| p.usage.bytes as f32)
                    .collect::<Vec<_>>();
                if samples.len() > 1 {
                    widgets::sparkline(ui, &samples, egui::vec2(220.0, 40.0), theme::accent());
                }
            });
            egui::ScrollArea::vertical()
                .id_salt("path-scan-samples")
                .max_height(115.0)
                .show(ui, |ui| {
                    for point in series.points.iter().rev() {
                        let files = if point.usage.files == 0
                            && matches!(
                                source,
                                crate::scan_history::Source::Review
                                    | crate::scan_history::Source::Native
                            ) {
                            "file count unavailable".into()
                        } else {
                            format!("{} files", point.usage.files)
                        };
                        ui.label(format!(
                            "{} · {} · {}{} · {}",
                            scan_history::timestamp(point.at),
                            format_bytes(point.usage.bytes),
                            files,
                            if point.usage.complete {
                                ""
                            } else {
                                " · partial"
                            },
                            point.basis.label()
                        ));
                    }
                });
        });
    }
    let filter = app.disk_history_filter.to_lowercase();
    let mut order = app
        .scan_history
        .series()
        .iter()
        .filter(|series| {
            series
                .path
                .to_string_lossy()
                .to_lowercase()
                .contains(&filter)
                || series.source.label().to_lowercase().contains(&filter)
        })
        .collect::<Vec<_>>();
    // Put the largest observed changes first, then the most recent measurements.
    order.sort_by_key(|series| {
        std::cmp::Reverse((
            series.delta().map_or(0, |(delta, _)| delta.unsigned_abs()),
            series.latest().at,
        ))
    });
    let mut selected = None;
    let height = widgets::table_scroll_height(ui);
    widgets::table(ui, "disk-history-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(170.0).clip(true))
        .column(Column::exact(110.0).clip(true))
        .column(Column::exact(104.0).clip(true))
        .column(Column::exact(102.0).clip(true))
        .column(Column::exact(192.0).clip(true))
        .column(Column::exact(60.0).clip(true))
        .header(32.0, |mut header| {
            for caption in [
                "Path / object",
                "Source",
                "Last size",
                "Change",
                "Last measured",
                "Scans",
            ] {
                header.col(|ui| {
                    ui.strong(caption);
                });
            }
        })
        .body(|body| {
            body.rows(36.0, order.len(), |mut row| {
                let series = order[row.index()];
                let point = series.latest();
                row.set_selected(
                    app.disk_history_selected
                        .as_ref()
                        .is_some_and(|(s, p)| *s == series.source && p == &series.path),
                );
                row.col(|ui| {
                    ui.add(egui::Label::new(series.path.display().to_string()).truncate())
                        .on_hover_text(series.path.display().to_string());
                });
                row.col(|ui| {
                    ui.label(series.source.label());
                });
                row.col(|ui| {
                    ui.label(crate::format::format_bytes_bound(
                        point.usage.bytes,
                        point.usage.complete,
                    ));
                });
                row.col(|ui| {
                    scan_history::trend(ui, &app.scan_history, series.source, &series.path);
                });
                row.col(|ui| {
                    ui.label(scan_history::timestamp(point.at));
                });
                row.col(|ui| {
                    ui.label(series.points.len().to_string());
                });
                if row.response().clicked() {
                    selected = Some((series.source, series.path.clone()));
                }
            });
        });
    if let Some(selected) = selected {
        app.disk_history_selected = Some(selected);
    }
}
