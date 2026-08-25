//! Process-level browser pressure. Tab counts stay unknown without the companion.

use sweeploom_core::{LiveSession, ProcessSnapshot, SessionKind};

/// One browser family rolled up from live sessions.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserHost {
    /// Chrome, Edge, Firefox, Brave, Safari, or Browser.
    pub family: &'static str,
    /// Logical SweepLoom sessions in this family.
    pub sessions: usize,
    /// Member processes.
    pub processes: usize,
    /// Combined RSS. Not uniquely reclaimable by killing the tree.
    pub rss_bytes: u64,
    /// Combined CPU percent.
    pub cpu_percent: f32,
}

/// Workstation browser pressure visible without the extension.
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserPressure {
    /// Native-messaging companion. False until the host is connected.
    pub companion_connected: bool,
    /// Rolled-up browser families, largest RSS first.
    pub hosts: Vec<BrowserHost>,
}

impl BrowserPressure {
    /// Roll up `SessionKind::Browser` sessions. Tab heat is not inferred.
    #[must_use]
    pub fn from_live(sessions: &[LiveSession], processes: &[ProcessSnapshot]) -> Self {
        let mut hosts: Vec<BrowserHost> = Vec::new();
        for session in sessions
            .iter()
            .filter(|item| item.kind == SessionKind::Browser)
        {
            let family = family_of(session, processes);
            match hosts.iter_mut().find(|host| host.family == family) {
                Some(host) => {
                    host.sessions += 1;
                    host.processes += session.processes.len();
                    host.rss_bytes = host.rss_bytes.saturating_add(session.rss_bytes);
                    host.cpu_percent += session.cpu_percent;
                }
                None => hosts.push(BrowserHost {
                    family,
                    sessions: 1,
                    processes: session.processes.len(),
                    rss_bytes: session.rss_bytes,
                    cpu_percent: session.cpu_percent,
                }),
            }
        }
        hosts.sort_by_key(|host| std::cmp::Reverse(host.rss_bytes));
        Self {
            companion_connected: false,
            hosts,
        }
    }

    /// Combined RSS across browser families.
    #[must_use]
    pub fn rss_bytes(&self) -> u64 {
        self.hosts.iter().map(|host| host.rss_bytes).sum()
    }
}

/// Map a process name to a browser family.
#[must_use]
pub fn family_from_name(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("msedge") {
        "Edge"
    } else if lower.contains("chrome") {
        "Chrome"
    } else if lower.contains("firefox") {
        "Firefox"
    } else if lower.contains("brave") {
        "Brave"
    } else if lower.contains("safari") {
        "Safari"
    } else {
        "Browser"
    }
}

fn family_of(session: &LiveSession, processes: &[ProcessSnapshot]) -> &'static str {
    session
        .processes
        .first()
        .and_then(|key| processes.iter().find(|process| process.key == *key))
        .map(|process| family_from_name(&process.name))
        .unwrap_or("Browser")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    use sweeploom_core::{
        ProcessKey, Recommendation, SessionActivity, SessionDiskUsage, SessionId,
        SessionNetworkUsage, SessionRecommendation, SessionSafety,
    };

    fn chrome_session() -> LiveSession {
        LiveSession {
            id: SessionId(1),
            kind: SessionKind::Browser,
            project: None,
            processes: vec![ProcessKey {
                pid: 10,
                started_at_unix_ms: 1,
            }],
            started_at: Some(SystemTime::UNIX_EPOCH),
            observed_last_activity: Some(SystemTime::UNIX_EPOCH),
            rss_bytes: 2_000_000_000,
            cpu_percent: 1.5,
            disk: SessionDiskUsage::default(),
            network: SessionNetworkUsage::default(),
            activity: SessionActivity::BackgroundActive,
            safety: SessionSafety::user(),
            recommendation: SessionRecommendation {
                recommendation: Recommendation::Keep,
                estimated_reclaimable_rss: 0,
            },
        }
    }

    #[test]
    fn rolls_up_chrome_from_session() {
        let process = sweeploom_core::ProcessSnapshot {
            key: ProcessKey {
                pid: 10,
                started_at_unix_ms: 1,
            },
            pid: 10,
            parent: None,
            name: "chrome.exe".into(),
            exe: None,
            cwd: None,
            command: vec!["chrome.exe".into()],
            started_at: None,
            runtime: std::time::Duration::from_secs(1),
            rss_bytes: 2_000_000_000,
            virtual_bytes: 2_000_000_000,
            cpu_percent: 1.5,
            accumulated_cpu_ms: 0,
            disk_read_delta: 0,
            disk_write_delta: 0,
            network: sweeploom_core::NetworkSnapshot::default(),
            project: None,
            session: None,
            safety_class: sweeploom_core::ProcessSafetyClass::UserApp,
        };
        let pressure = BrowserPressure::from_live(&[chrome_session()], &[process]);
        assert_eq!(pressure.hosts.len(), 1);
        assert_eq!(pressure.hosts[0].family, "Chrome");
        assert!(!pressure.companion_connected);
        assert_eq!(pressure.rss_bytes(), 2_000_000_000);
    }
}
