//! Human session titles. GenericApp is the process name, not "App".

use sweeploom_core::{LiveSession, ProcessSafetyClass, ProcessSnapshot, SessionKind};

/// Title shown in tables and details.
#[must_use]
pub fn title(session: &LiveSession, processes: &[ProcessSnapshot]) -> String {
    let root = session.processes.first().and_then(|key| {
        processes
            .iter()
            .find(|process| process.key == *key)
            .map(|process| pretty_name(&process.name))
    });
    match session.kind {
        SessionKind::GenericApp | SessionKind::Unknown => {
            root.unwrap_or_else(|| session.label().to_owned())
        }
        _ => session.label().to_owned(),
    }
}

/// Sessions worth showing before the leftover-app flood.
#[must_use]
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
        processes.iter().any(|process| {
            process.key == *key
                && matches!(
                    process.safety_class,
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
