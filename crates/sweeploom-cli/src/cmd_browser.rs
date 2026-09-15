//! `sweeploom browser` — process-level pressure, no fake tab counts.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sweeploom_browser::{BrowserPressure, StoredCompanion, load_snapshot};
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};

use crate::bytes::format_bytes;

pub fn run() {
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _capability = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    let mut pressure = BrowserPressure::from_live(&sessions, &snapshot.processes);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0);
    let stored = load_snapshot(&locations.app_data).ok().flatten();
    pressure.companion_connected = stored.as_ref().is_some_and(|item| item.is_fresh(now_ms));
    println!(
        "companion={} hosts={} rss={}",
        if pressure.companion_connected {
            "connected"
        } else if stored.is_some() {
            "stale"
        } else {
            "disconnected"
        },
        pressure.hosts.len(),
        format_bytes(pressure.rss_bytes())
    );
    println!("{}", tab_line(stored.as_ref(), now_ms));
    if pressure.hosts.is_empty() {
        println!("no browser process trees");
        return;
    }
    for host in &pressure.hosts {
        println!(
            "{:<8} sessions={:<3} proc={:<4} rss={:<10} cpu={:>5.1}%",
            host.family,
            host.sessions,
            host.processes,
            format_bytes(host.rss_bytes),
            host.cpu_percent
        );
    }
}

/// Tab line for `sweeploom browser`. A stale snapshot is not a tab count.
fn tab_line(stored: Option<&StoredCompanion>, now_ms: u64) -> String {
    match stored.filter(|item| item.is_fresh(now_ms)) {
        Some(item) => format!(
            "tabs={} discard_suggestions={} written_ms={}",
            item.tabs.tabs.len(),
            item.tabs.discard_count(now_ms),
            item.written_unix_ms
        ),
        None => "tab lastAccessed unavailable without the companion; not shown as zero".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_browser::{CompanionTabs, FRESH_MS, StoredCompanion, TabSnapshot};

    fn companion(written_unix_ms: u64, tabs: u32) -> StoredCompanion {
        StoredCompanion {
            written_unix_ms,
            tabs: CompanionTabs {
                tabs: (0..tabs)
                    .map(|index| TabSnapshot {
                        tab_id: i64::from(index),
                        window_id: 1,
                        title: String::new(),
                        url: "https://example.com".to_owned(),
                        last_accessed_ms: None,
                        pinned: false,
                        audible: false,
                        discarded: false,
                        incognito: false,
                    })
                    .collect(),
                active_tab_id: None,
            },
            instance_id: String::new(),
            epoch: 0,
        }
    }

    #[test]
    fn stale_companion_never_reports_tab_counts() {
        let stored = companion(1_000, 3);
        let now_ms = 1_000 + FRESH_MS + 1;
        assert!(!stored.is_fresh(now_ms));
        let line = tab_line(Some(&stored), now_ms);
        assert!(
            !line.contains("tabs="),
            "stale companion reported tab counts: {line}"
        );
    }

    #[test]
    fn fresh_companion_reports_tab_counts() {
        let stored = companion(1_000, 3);
        let now_ms = 1_000 + FRESH_MS;
        assert!(stored.is_fresh(now_ms));
        assert!(tab_line(Some(&stored), now_ms).starts_with("tabs=3 "));
    }

    #[test]
    fn missing_companion_reports_no_tab_counts() {
        assert!(!tab_line(None, 1_000).contains("tabs="));
    }
}
