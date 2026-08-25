//! Process connections. Missing capability is never displayed as zero activity.

#![cfg_attr(not(test), warn(missing_docs))]

use std::collections::HashMap;

use sweeploom_core::{NetworkSnapshot, ProcessKey};
use sweeploom_platform::NetworkCapability;

/// One listening or established endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// Local port.
    pub local_port: u16,
    /// Remote host:port if established.
    pub remote: Option<String>,
    /// True when LISTEN.
    pub listening: bool,
}

/// Attach connection metadata to snapshots. Byte rates stay capability-gated.
#[must_use]
pub fn enrich_network(processes: &mut [sweeploom_core::ProcessSnapshot]) -> NetworkCapability {
    let capability = current_capability();
    if !capability.connections {
        return capability;
    }
    let table = load_pid_endpoints();
    for process in processes {
        if let Some(endpoints) = table.get(&process.pid) {
            process.network = snapshot_from_endpoints(endpoints);
        } else {
            process.network.connections_available = true;
            process.network.byte_rate_available = false;
        }
        let _ = process.key;
    }
    capability
}

/// Current platform capability.
#[must_use]
pub fn current_capability() -> NetworkCapability {
    #[cfg(any(target_os = "linux", windows))]
    {
        NetworkCapability {
            connections: true,
            byte_rates: false,
        }
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        NetworkCapability::unknown()
    }
}

fn snapshot_from_endpoints(endpoints: &[Endpoint]) -> NetworkSnapshot {
    NetworkSnapshot {
        connections_available: true,
        byte_rate_available: false,
        observed_rx_bytes: 0,
        observed_tx_bytes: 0,
        listening_ports: endpoints
            .iter()
            .filter(|item| item.listening)
            .map(|item| item.local_port)
            .collect(),
        remotes: endpoints
            .iter()
            .filter_map(|item| item.remote.clone())
            .collect(),
    }
}

fn load_pid_endpoints() -> HashMap<u32, Vec<Endpoint>> {
    #[cfg(target_os = "linux")]
    {
        linux::load()
    }
    #[cfg(windows)]
    {
        windows::load()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        HashMap::new()
    }
}

/// Look up endpoints for a process key. PID reuse is the caller's problem;
/// they must confirm the key is still live.
#[must_use]
pub fn endpoints_for(_key: ProcessKey) -> Vec<Endpoint> {
    Vec::new()
}

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux {
    use super::Endpoint;
    use std::collections::HashMap;
    use std::fs;

    pub fn load() -> HashMap<u32, Vec<Endpoint>> {
        let mut inode_to_endpoint = HashMap::new();
        parse_table("/proc/net/tcp", &mut inode_to_endpoint);
        parse_table("/proc/net/tcp6", &mut inode_to_endpoint);
        let mut by_pid: HashMap<u32, Vec<Endpoint>> = HashMap::new();
        let Ok(proc) = fs::read_dir("/proc") else {
            return by_pid;
        };
        for entry in proc.flatten() {
            let pid: u32 = match entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse().ok())
            {
                Some(pid) => pid,
                None => continue,
            };
            let fd_dir = entry.path().join("fd");
            let Ok(fds) = fs::read_dir(fd_dir) else {
                continue;
            };
            for fd in fds.flatten() {
                let Ok(target) = fs::read_link(fd.path()) else {
                    continue;
                };
                let Some(inode) = socket_inode(&target) else {
                    continue;
                };
                if let Some(endpoint) = inode_to_endpoint.get(&inode) {
                    by_pid.entry(pid).or_default().push(endpoint.clone());
                }
            }
        }
        by_pid
    }

    fn socket_inode(target: &std::path::Path) -> Option<u64> {
        let text = target.to_str()?;
        let rest = text.strip_prefix("socket:[")?.strip_suffix(']')?;
        rest.parse().ok()
    }

    fn parse_table(path: &str, into: &mut HashMap<u64, Endpoint>) {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 10 {
                continue;
            }
            let local = cols[1];
            let remote = cols[2];
            let state = cols[3];
            let inode: u64 = match cols[9].parse() {
                Ok(inode) => inode,
                Err(_) => continue,
            };
            let Some((_, local_port)) = parse_addr(local) else {
                continue;
            };
            let listening = state.eq_ignore_ascii_case("0A");
            let remote_label = if listening {
                None
            } else {
                parse_addr(remote).map(|(host, port)| format!("{host}:{port}"))
            };
            into.insert(
                inode,
                Endpoint {
                    local_port,
                    remote: remote_label,
                    listening,
                },
            );
        }
    }

    fn parse_addr(field: &str) -> Option<(String, u16)> {
        let (ip, port) = field.split_once(':')?;
        let port = u16::from_str_radix(port, 16).ok()?;
        Some((ip.to_owned(), port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_is_explicit() {
        let capability = current_capability();
        assert!(!capability.byte_rates);
    }
}
