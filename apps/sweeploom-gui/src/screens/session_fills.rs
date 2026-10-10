//! Session table cell drawing.

use super::session_plan;
use super::session_rows::{Line, SessionRow};
use crate::format::format_bytes;
use eframe::egui::RichText;
use sweeploom_core::LiveSession;

pub(super) fn fill_line(
    lines: &[Line],
    sessions: &[LiveSession],
    presentations: &[SessionRow],
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
            fill_session(
                sessions,
                presentations,
                planned,
                selected_id,
                row,
                selected,
                line,
            );
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
        node_procs,
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
        ui.spacing_mut().item_spacing.x = 6.0;
        if crate::widgets::disclose(ui, *expanded, crate::theme::muted(ui)) {
            *toggle = Some(key.clone());
        }
        crate::brand::show_session(ui, *kind, title, 18.0);
        crate::widgets::two_line(
            ui,
            RichText::new(title).strong(),
            &format!("{count} launches"),
        );
    });
    row.col(|ui| {
        process_count(ui, *procs, *node_procs);
    });
    row.col(|ui| {
        ui.label(format_bytes(*rss));
    });
    row.col(|ui| {
        cpu_label(ui, *cpu);
    });
    row.col(|ui| {
        crate::widgets::status_pill(ui, status);
    });
    row.col(|ui| {
        project_label(ui, project, None);
    });
}

fn fill_session(
    sessions: &[LiveSession],
    presentations: &[SessionRow],
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
    let Some(presentation) = presentations.iter().find(|p| p.index == *index) else {
        return;
    };
    let rss = format_bytes(session.rss_bytes);
    row.set_selected(is_selected);
    row.col(|ui| {
        session_plan::checkbox(ui, session, planned);
    });
    row.col(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if *nested {
            ui.add_space(22.0);
        }
        crate::brand::show_session(ui, session.kind, title, 18.0);
        crate::widgets::two_line(ui, title.as_str(), &presentation.subtitle)
            .on_hover_text(&presentation.source_tip);
    });
    row.col(|ui| {
        process_count(ui, session.processes.len(), presentation.node_procs);
    });
    row.col(|ui| {
        ui.label(&rss);
    });
    row.col(|ui| {
        cpu_label(ui, session.cpu_percent);
    });
    row.col(|ui| {
        status_cell(ui, &presentation.status, &presentation.status_detail);
    });
    row.col(|ui| {
        project_label(ui, &presentation.project, Some(&presentation.source_tip));
    });
    if row.response().clicked() {
        *selected = Some(id);
    }
}

fn process_count(ui: &mut egui::Ui, total: usize, nodes: usize) {
    ui.spacing_mut().item_spacing.x = 6.0;
    ui.label(total.to_string());
    if nodes > 0 {
        ui.label(
            RichText::new(format!("{nodes} Node"))
                .size(12.0)
                .color(crate::theme::muted(ui)),
        );
    }
}

fn cpu_label(ui: &mut egui::Ui, cpu: f32) {
    let tone = if cpu >= 80.0 {
        crate::theme::Tone::Warn
    } else {
        crate::theme::Tone::Neutral
    };
    ui.label(RichText::new(format!("{cpu:.1}%")).color(crate::theme::tone(ui, tone)));
}

fn status_cell(ui: &mut egui::Ui, status: &str, detail: &str) {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, 38.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            crate::widgets::status_pill(ui, status);
            ui.add(
                egui::Label::new(
                    RichText::new(detail)
                        .size(11.5)
                        .color(crate::theme::muted(ui)),
                )
                .truncate()
                .selectable(false),
            )
            .on_hover_text(
                "Observed CPU / disk / available network activity. Uptime is not proven idle.",
            );
        },
    );
}

fn project_label(ui: &mut egui::Ui, project: &str, tip: Option<&str>) {
    let quiet = matches!(project, "/" | "—" | "Multiple workspaces");
    let mut text = RichText::new(project);
    if quiet {
        text = text.color(crate::theme::muted(ui));
    }
    let response = ui.add(egui::Label::new(text).truncate().selectable(false));
    if let Some(tip) = tip {
        response.on_hover_text(tip);
    }
}
