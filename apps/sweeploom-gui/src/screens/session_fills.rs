//! Session table cell drawing.

use super::session_plan;
use super::session_rows::Line;
use crate::format::format_bytes;
use eframe::egui::RichText;
use sweeploom_core::LiveSession;

pub(super) fn fill_line(
    lines: &[Line],
    sessions: &[LiveSession],
    planned: &mut std::collections::HashSet<sweeploom_core::ProcessKey>,
    selected_id: Option<sweeploom_core::SessionId>,
    row: &mut egui_extras::TableRow<'_, '_>,
    selected: &mut Option<sweeploom_core::SessionId>,
    toggle: &mut Option<String>,
) {
    match lines.get(row.index()) {
        Some(line @ Line::Group { key, .. }) => {
            fill_group(row, sessions, planned, line, toggle);
            if row.response().clicked() {
                *toggle = Some(key.clone());
            }
        }
        Some(line @ Line::Session { .. }) => {
            fill_session(sessions, planned, selected_id, row, selected, line);
        }
        None => {}
    }
}

fn fill_group(
    row: &mut egui_extras::TableRow<'_, '_>,
    sessions: &[LiveSession],
    planned: &mut std::collections::HashSet<sweeploom_core::ProcessKey>,
    line: &Line,
    toggle: &mut Option<String>,
) {
    let Line::Group {
        key,
        title,
        kind,
        rss,
        cpu,
        procs,
        count,
        status,
        project,
        expanded,
        members,
    } = line
    else {
        return;
    };
    row.col(|ui| {
        session_plan::checkbox_group(ui, sessions, members, planned);
    });
    row.col(|ui| {
        if crate::widgets::disclose(ui, *expanded, crate::theme::accent()) {
            *toggle = Some(key.clone());
        }
        crate::brand::show_session(ui, *kind, title, 16.0);
        ui.add(
            egui::Label::new(RichText::new(format!("{title}  ·  {count}")).strong())
                .truncate()
                .selectable(false),
        );
    });
    row.col(|ui| {
        ui.label(procs.to_string());
    });
    row.col(|ui| {
        ui.label(format_bytes(*rss));
    });
    row.col(|ui| {
        ui.label(format!("{cpu:.1}%"));
    });
    row.col(|ui| {
        ui.label(status);
    });
    row.col(|ui| {
        ui.add(egui::Label::new(project).truncate().selectable(false));
    });
}

fn fill_session(
    sessions: &[LiveSession],
    planned: &mut std::collections::HashSet<sweeploom_core::ProcessKey>,
    selected_id: Option<sweeploom_core::SessionId>,
    row: &mut egui_extras::TableRow<'_, '_>,
    selected: &mut Option<sweeploom_core::SessionId>,
    line: &Line,
) {
    let Line::Session {
        index,
        nested,
        title,
    } = line
    else {
        return;
    };
    let Some(session) = sessions.get(*index) else {
        return;
    };
    let is_selected = selected_id == Some(session.id);
    let id = session.id;
    let procs = session.processes.len().to_string();
    let rss = format_bytes(session.rss_bytes);
    let cpu = format!("{:.1}%", session.cpu_percent);
    let rec = session.activity.label().to_owned();
    let project = session
        .project
        .as_ref()
        .map(|item| super::session_rows::project_name(&item.0))
        .unwrap_or_else(|| "Unknown".to_owned());
    let project_tip = session
        .project
        .as_ref()
        .map(|item| crate::format::short_path(&item.0));
    row.set_selected(is_selected);
    row.col(|ui| {
        session_plan::checkbox(ui, session, planned);
    });
    row.col(|ui| {
        if *nested {
            ui.add_space(16.0);
        }
        crate::brand::show_session(ui, session.kind, title, 16.0);
        ui.add(
            egui::Label::new(RichText::new(title).size(16.0))
                .truncate()
                .selectable(false),
        );
    });
    row.col(|ui| {
        ui.label(&procs);
    });
    row.col(|ui| {
        ui.label(&rss);
    });
    row.col(|ui| {
        ui.label(&cpu);
    });
    row.col(|ui| {
        ui.label(&rec);
    });
    row.col(|ui| {
        let label = ui.add(egui::Label::new(&project).truncate().selectable(false));
        if let Some(tip) = project_tip {
            label.on_hover_text(tip);
        }
    });
    if row.response().clicked() {
        *selected = Some(id);
    }
}
