use super::super::*;
use crate::format::format_bytes;
use eframe::egui;
use std::process::Command;

pub(super) fn draw(
    app: &mut SweepLoomApp,
    ui: &mut egui::Ui,
    processes: Option<&[sweeploom_core::ProcessSnapshot]>,
) -> bool {
    let mut refresh = false;
    let busy =
        app.native_cleanup.applying.is_some() || app.apply_rx.is_some() || app.disk_actions.busy();
    crate::widgets::toolbar(ui, |ui| {
        if crate::widgets::button(
            ui,
            "Refresh listing",
            app.native_cleanup.active().rx.is_none(),
        )
        .clicked()
        {
            refresh = true;
        }
        if app.native_cleanup.pane == Pane::Docker
            && app
                .native_cleanup
                .active()
                .listing
                .as_ref()
                .is_some_and(|list| !list.note.is_empty())
            && crate::widgets::button(ui, "Open Docker Desktop", true).clicked()
        {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("/usr/bin/open").args(["-a", "Docker"]).spawn();
            }
        }
        if app.native_cleanup.pane == Pane::Models
            && crate::widgets::button(ui, "Open Ollama", true).clicked()
        {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("/usr/bin/open").args(["-a", "Ollama"]).spawn();
            }
        }
        let selected: Vec<_> = app
            .native_cleanup
            .active()
            .listing
            .iter()
            .flat_map(|list| &list.items)
            .filter(|item| item.selected)
            .cloned()
            .collect();
        actions(app, ui, &selected, processes);
        let paths = selected
            .iter()
            .filter_map(|item| item.path.clone())
            .collect::<Vec<_>>();
        if crate::widgets::button(ui, "Show in Finder", !paths.is_empty()).clicked() {
            app.reveal_paths(&paths);
        }
        if app.native_cleanup.pane == Pane::Ios
            && crate::widgets::danger_button(
                ui,
                "Erase device data…",
                !selected.is_empty()
                    && !busy
                    && selected
                        .iter()
                        .all(|item| item.enabled && item.kind == Kind::Device),
            )
            .clicked()
        {
            let mut devices = selected.clone();
            for device in &mut devices {
                device.status = "Erase data; keep device".into();
            }
            app.native_cleanup.pending = Some((app.native_cleanup.pane, devices));
        }
        let selected_bytes: u64 = selected
            .iter()
            .filter_map(|item| item.bytes)
            .fold(0, u64::saturating_add);
        ui.add_space(crate::theme::XS);
        crate::widgets::caption(
            ui,
            format!(
                "{} selected · {} listed size",
                selected.len(),
                format_bytes(selected_bytes)
            ),
        );
    });
    refresh
}

fn actions(
    app: &mut SweepLoomApp,
    ui: &mut egui::Ui,
    selected: &[Item],
    processes: Option<&[sweeploom_core::ProcessSnapshot]>,
) {
    let busy =
        app.native_cleanup.applying.is_some() || app.apply_rx.is_some() || app.disk_actions.busy();
    let native: Vec<_> = selected
        .iter()
        .filter(|item| item.cleanable(processes) && item.native())
        .cloned()
        .collect();
    let caches: Vec<_> = selected
        .iter()
        .filter(|item| item.cleanable(processes) && item.kind == Kind::Cache)
        .collect();
    if matches!(
        app.native_cleanup.pane,
        Pane::Docker | Pane::Ios | Pane::Models | Pane::Data
    ) && crate::widgets::danger_button(
        ui,
        "Remove selected objects…",
        !native.is_empty() && !busy,
    )
    .clicked()
    {
        app.native_cleanup.pending = Some((app.native_cleanup.pane, native));
    }
    if matches!(app.native_cleanup.pane, Pane::Caches | Pane::Apps)
        && crate::widgets::danger_button(ui, "Clean selected caches…", !caches.is_empty() && !busy)
            .clicked()
    {
        app.request_clean(
            caches
                .iter()
                .enumerate()
                .filter_map(|(index, item)| cache_candidate(item, index))
                .collect(),
        );
    }
    let trash_paths: Vec<_> = selected
        .iter()
        .filter(|item| {
            item.cleanable(processes) && matches!(item.kind, Kind::Cache | Kind::Trash)
        })
        .filter_map(|item| item.path.clone())
        .collect();
    if !matches!(app.native_cleanup.pane, Pane::Docker | Pane::Ios)
        && crate::widgets::danger_button(ui, "Move to Trash…", !trash_paths.is_empty() && !busy)
            .clicked()
    {
        app.request_trash(&trash_paths);
    }
}
