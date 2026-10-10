//! Session identity: workload, runtime and launch origin are separate facts.

use std::collections::HashSet;
use std::time::{Duration, SystemTime};
use sweeploom_core::{
    LiveSession, ProcessSafetyClass, ProcessSnapshot, SessionActivity, SessionKind,
};
use sweeploom_session::{is_node, node_identity};

use super::session_rows::SessionRow;
use crate::node_versions::NodeVersions;
#[path = "../cleanup_ios_session.rs"]
mod simulator;

pub(super) fn root<'a>(
    session: &LiveSession,
    processes: &'a [ProcessSnapshot],
) -> Option<&'a ProcessSnapshot> {
    session
        .processes
        .first()
        .and_then(|key| processes.iter().find(|p| p.key == *key))
}

/// Title shown in tables and details.
pub fn title(session: &LiveSession, processes: &[ProcessSnapshot]) -> String {
    if let Some(process) = root(session, processes) {
        if let Some(simulator) = simulator::describe(process) {
            return simulator.title;
        }
        if let Some(identity) = node_identity(process) {
            return if identity.workload == "Cursor agent" {
                identity.workload
            } else {
                format!(
                    "{} · {}",
                    if matches!(session.kind, SessionKind::GenericApp | SessionKind::Unknown) {
                        "Node"
                    } else {
                        session.kind.label()
                    },
                    identity.workload
                )
            };
        }
        if matches!(session.kind, SessionKind::GenericApp | SessionKind::Unknown) {
            return pretty_name(&process.name);
        }
    }
    session.label().to_owned()
}

pub(super) fn row(
    index: usize,
    session: &LiveSession,
    processes: &[ProcessSnapshot],
    versions: &NodeVersions,
    now: SystemTime,
) -> SessionRow {
    let title = title(session, processes);
    let root = root(session, processes);
    let identity = root.and_then(node_identity);
    let node_procs = session
        .processes
        .iter()
        .filter(|key| processes.iter().any(|p| p.key == **key && is_node(p)))
        .count();
    let pid = root
        .map(|p| format!("PID {}", p.pid))
        .unwrap_or_else(|| "PID unavailable".into());
    let age = root
        .filter(|p| p.started_at.is_some())
        .map(|p| format!("up {}", duration(p.runtime)))
        .unwrap_or_else(|| "age unknown".into());
    let runtime = identity.as_ref().map(|id| {
        id.executable
            .as_ref()
            .and_then(|path| versions.get(path))
            .map(|v| format!("Node {v}"))
            .or_else(|| id.version_hint.as_ref().map(|v| format!("Node {v} (path)")))
            .unwrap_or_else(|| "Node version unavailable".into())
    });
    let subtitle = [Some(pid), runtime, Some(age)]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
    let group_key = if let Some(id) = &identity {
        if id.workload == "Cursor agent" {
            title.clone()
        } else if id.entrypoint.is_none() && id.workload == "runtime / REPL" {
            format!("{title}\0{}", session.id.0)
        } else {
            format!(
                "{title}\0{:?}\0{:?}",
                id.entrypoint
                    .as_ref()
                    .or_else(|| root.and_then(|p| p.cwd.as_ref())),
                id.executable
            )
        }
    } else {
        title.clone()
    };
    let cwd = root.and_then(|p| p.cwd.as_ref());
    let project = session.project.as_ref().map(|p| &p.0).or(cwd);
    let simulator = root.and_then(simulator::describe);
    let source = simulator.as_ref().map_or_else(
        || {
            project
                .map(|p| super::session_rows::project_name(p))
                .unwrap_or_else(|| "cwd unavailable".into())
        },
        |sim| sim.project.clone(),
    );
    let origin = identity
        .as_ref()
        .and_then(|id| id.entrypoint.as_ref())
        .map(|p| format!("Script: {}\n", p.display()))
        .unwrap_or_default();
    let origin = simulator
        .map(|sim| format!("{}\n{origin}", sim.tip))
        .unwrap_or(origin);
    let source_tip = format!(
        "{origin}Working directory: {}\nProject: {}\nLaunch chain: {}",
        cwd.map(|p| p.display().to_string())
            .unwrap_or_else(|| "unavailable".into()),
        session
            .project
            .as_ref()
            .map(|p| p.0.display().to_string())
            .unwrap_or_else(|| "not attributed".into()),
        root.map(|p| launch_chain(p, processes))
            .unwrap_or_else(|| "unavailable".into())
    );
    SessionRow {
        index,
        title,
        group_key,
        subtitle,
        node_procs,
        status_detail: idle_label(session, now),
        source_tip,
        kind: session.kind,
        rss: session.rss_bytes,
        cpu: session.cpu_percent,
        procs: session.processes.len(),
        status: status(session).into(),
        project: source,
    }
}

pub(super) fn status(session: &LiveSession) -> &'static str {
    if session.kind.is_agent()
        && matches!(
            session.activity,
            SessionActivity::LikelyForgotten | SessionActivity::SleepingMemoryHeavy
        )
    {
        return "Quiet agent";
    }
    match session.activity {
        SessionActivity::BackgroundActive => {
            if session.observed_last_activity.is_some() {
                "Low activity"
            } else {
                "Observing"
            }
        }
        SessionActivity::LikelyForgotten => "Possibly stale",
        _ => session.activity.label(),
    }
}

pub(super) fn idle_label(session: &LiveSession, now: SystemTime) -> String {
    session
        .observed_last_activity
        .and_then(|at| now.duration_since(at).ok())
        .map(|idle| format!("Quiet for {}", duration(idle)))
        .unwrap_or_else(|| "Idle history unavailable".into())
}

pub(super) fn duration(span: Duration) -> String {
    let secs = span.as_secs();
    if secs >= 86400 {
        format!("{}d {}h", secs / 86400, secs % 86400 / 3600)
    } else if secs >= 3600 {
        format!("{}h {}m", secs / 3600, secs % 3600 / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

pub(super) fn launch_chain(process: &ProcessSnapshot, processes: &[ProcessSnapshot]) -> String {
    let mut parent = process.parent;
    let mut seen = HashSet::new();
    let mut labels = Vec::new();
    while let Some(key) = parent {
        if !seen.insert(key) || labels.len() >= 12 {
            break;
        }
        let Some(p) = processes.iter().find(|p| p.key == key) else {
            labels.push(format!("PID {} (not in snapshot)", key.pid));
            break;
        };
        let name = node_identity(p)
            .map(|id| id.workload)
            .unwrap_or_else(|| pretty_name(&p.name));
        labels.push(format!("{name} (PID {})", p.pid));
        parent = p.parent;
    }
    if labels.is_empty() {
        "parent unavailable".into()
    } else {
        labels.join(" <- ")
    }
}

/// Sessions worth showing before the leftover-app flood.
pub fn is_spotlight(session: &LiveSession, processes: &[ProcessSnapshot]) -> bool {
    if !matches!(session.kind, SessionKind::GenericApp | SessionKind::Unknown) {
        return true;
    }
    if session.rss_bytes >= 64_000_000 || session.cpu_percent > 0.5 {
        return true;
    }
    if !session.network.listening_ports.is_empty() {
        return true;
    }
    session.processes.iter().any(|key| {
        processes.iter().any(|p| {
            p.key == *key
                && matches!(
                    p.safety_class,
                    ProcessSafetyClass::Agent
                        | ProcessSafetyClass::DeveloperTool
                        | ProcessSafetyClass::DevServer
                        | ProcessSafetyClass::Helper
                )
        })
    })
}

fn pretty_name(name: &str) -> String {
    name.strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name)
        .to_owned()
}
