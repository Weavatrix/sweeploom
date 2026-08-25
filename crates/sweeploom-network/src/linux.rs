//! `/proc` TCP/UDP inode → PID map. Byte rates stay unavailable without eBPF.

use std::collections::HashMap;
use std::fs;

use super::Endpoint;

/// Load TCP/UDP endpoints keyed by owning PID.
#[must_use]
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
