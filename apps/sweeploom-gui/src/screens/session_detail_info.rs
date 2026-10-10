//! Launch origin, activity assessment and runtime identity of a selected session.

use eframe::egui::{self, RichText};
use sweeploom_core::{LiveSession, ProcessSnapshot};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::theme;
use crate::widgets;

use super::super::session_label;
use super::field;

pub(super) fn draw_identity(app: &SweepLoomApp, ui: &mut egui::Ui, session: &LiveSession) {
    let processes = app
        .snapshot
        .as_ref()
        .map(|s| s.processes.as_slice())
        .unwrap_or(&[]);
    let now = app
        .snapshot
        .as_ref()
        .map(|s| s.captured_at)
        .unwrap_or_else(std::time::SystemTime::now);
    let presentation = session_label::row(0, session, processes, &app.node_versions, now);
    if let Some(root) = session_label::root(session, processes) {
        ui.add_space(theme::XS);
        widgets::pair(
            ui,
            ("session-identity", session.id.0),
            widgets::panel_frame,
            |ui| draw_origin(ui, session, root, processes),
            |ui| draw_assessment(app, ui, session, &presentation.status_detail, now),
        );
        egui::CollapsingHeader::new("Runtime, script & command")
            .id_salt(("session-command", session.id.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Credentials are redacted.")
                        .small()
                        .color(theme::muted(ui)),
                );
                egui::Grid::new(("session-runtime", session.id.0))
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        if let Some(identity) = sweeploom_session::node_identity(root) {
                            if let Some(exe) = identity.executable {
                                field(ui, "Node executable", exe.display().to_string());
                                let version =
                                    app.node_versions
                                        .get(&exe)
                                        .map(|v| format!("{v} (queried with --version)"))
                                        .or_else(|| {
                                            identity.version_hint.map(|v| {
                                    format!("{v} (installation path; runtime query unavailable)")
                                })
                                        })
                                        .unwrap_or_else(|| "unavailable / checking".into());
                                field(ui, "Node version", version);
                            }
                            if let Some(script) = identity.entrypoint {
                                field(ui, "Script", script.display().to_string());
                            }
                        }
                        field(ui, "Command", root.command.join(" "));
                    });
            });
    }
}

fn draw_origin(
    ui: &mut egui::Ui,
    session: &LiveSession,
    root: &ProcessSnapshot,
    processes: &[ProcessSnapshot],
) {
    widgets::panel_title(ui, "Launch & location");
    egui::Grid::new(("session-origin", session.id.0))
        .num_columns(2)
        .spacing([theme::LG, 6.0])
        .show(ui, |ui| {
            field(
                ui,
                "Working directory",
                root.cwd
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "Unavailable".into()),
            );
            field(
                ui,
                "Launched by",
                session_label::launch_chain(root, processes),
            );
            if let Some(project) = &session.project {
                field(ui, "Project", project.0.display().to_string());
            }
        });
}

fn draw_assessment(
    app: &SweepLoomApp,
    ui: &mut egui::Ui,
    session: &LiveSession,
    status_detail: &str,
    now: std::time::SystemTime,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Activity assessment").size(13.5).strong());
        widgets::caption(ui, status_detail);
    });
    ui.add_space(theme::SM);
    ui.add(
        egui::Label::new(assessment_reason(
            session,
            now,
            app.current_project.as_ref(),
        ))
        .wrap(),
    );
    if !session.network.byte_rate_available {
        ui.add_space(theme::XS);
        widgets::caption(
            ui,
            "Based on observed CPU and disk activity. Network activity is unavailable; quiet servers may be waiting for requests.",
        );
    }
}

fn assessment_reason(
    session: &LiveSession,
    now: std::time::SystemTime,
    current: Option<&sweeploom_core::ProjectId>,
) -> String {
    use sweeploom_core::SessionActivity;
    if session.safety.terminate_disabled {
        return "Keep: system-critical process tree.".into();
    }
    if current.is_some_and(|p| session.project.as_ref() == Some(p)) {
        return "Keep: belongs to the current workspace.".into();
    }
    if session.cpu_percent > 0.5 || session.disk.read_bytes > 0 || session.disk.write_bytes > 0 {
        return format!(
            "Keep: current CPU {:.1}% or disk I/O indicates activity.",
            session.cpu_percent
        );
    }
    if session.activity == SessionActivity::NetworkActive {
        return "Keep: network traffic was observed.".into();
    }
    if session.kind.is_agent() {
        return "Keep: agent/context process. Quiet activity and old age do not establish that the workspace is abandoned.".into();
    }
    if session.kind == sweeploom_core::SessionKind::Mcp
        && session.recommendation.recommendation == sweeploom_core::Recommendation::Keep
    {
        return "Keep: MCP belongs to a live agent process tree, or has current activity.".into();
    }
    if session.activity == SessionActivity::OrphanCandidate {
        return "Inspect: no live agent ancestor was found. This can be a standalone MCP service; missing ancestry alone does not prove it is stale.".into();
    }
    let idle = session
        .observed_last_activity
        .and_then(|t| now.duration_since(t).ok());
    let Some(idle) = idle else {
        return "Insufficient continuous observation. Process age is not counted as idle time."
            .into();
    };
    if idle.as_secs() < 7200 {
        return "Recent observation only; the 2-hour quiet threshold has not been reached. Keep observing.".into();
    }
    if !session.kind.is_known_dev() {
        return "Needs review: quiet for at least 2 hours, but the workload is unclassified. Keep until inspected.".into();
    }
    format!(
        "Review candidate: recognized helper with at least 2 hours of continuous low CPU / disk activity. Holds {} RAM. Listening ports and ownership should be checked before stopping.",
        format_bytes(session.rss_bytes)
    )
}
