//! TCP ESTATS byte counters via IP Helper (`GetPerTcpConnectionEStats`).
//!
//! Enabling collection requires membership in Administrators (Microsoft Learn:
//! SetPerTcpConnectionEStats). If that fails, byte rates stay unavailable and
//! are never shown as zero. Kernel ETW is a later privileged backend.

use std::collections::HashMap;
use std::mem::size_of;
use std::ptr;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetPerTcpConnectionEStats, MIB_TCPROW_LH, MIB_TCPROW_LH_0,
    SetPerTcpConnectionEStats, TCP_ESTATS_DATA_ROD_v0, TCP_ESTATS_DATA_RW_v0, TCP_ESTATS_TYPE,
    TCP_TABLE_OWNER_PID_CONNECTIONS,
};
use windows_sys::Win32::Networking::WinSock::AF_INET;

/// `TcpConnectionEstatsData` — Microsoft Learn `TCP_ESTATS_TYPE`.
const TCP_ESTATS_DATA: TCP_ESTATS_TYPE = 1;

#[repr(C)]
struct TcpRowOwnerPid {
    state: u32,
    local_addr: u32,
    local_port: u32,
    remote_addr: u32,
    remote_port: u32,
    owning_pid: u32,
}

static BASELINE: Mutex<Option<HashMap<u32, (u64, u64)>>> = Mutex::new(None);

/// Observed TCP payload bytes per PID since SweepLoom started watching.
///
/// `None` means ESTATS could not be enabled or no IPv4 TCP rows were readable.
/// `Some` with an empty map is the first successful sample (deltas start next).
pub fn load() -> Option<HashMap<u32, (u64, u64)>> {
    let now = current_totals();
    if now.is_empty() {
        return None;
    }
    let Ok(mut guard) = BASELINE.lock() else {
        return None;
    };
    let Some(base) = guard.as_mut() else {
        *guard = Some(now);
        return Some(HashMap::new());
    };
    let mut out = HashMap::new();
    for (pid, (rx, tx)) in &now {
        let (brx, btx) = base.entry(*pid).or_insert((*rx, *tx));
        out.insert(*pid, (rx.saturating_sub(*brx), tx.saturating_sub(*btx)));
    }
    Some(out)
}

fn current_totals() -> HashMap<u32, (u64, u64)> {
    let mut by_pid: HashMap<u32, (u64, u64)> = HashMap::new();
    for row in read_owner_pid_table() {
        let Some((rx, tx)) = row_bytes(&row) else {
            continue;
        };
        let entry = by_pid.entry(row.owning_pid).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(rx);
        entry.1 = entry.1.saturating_add(tx);
    }
    by_pid
}

fn win_ok(status: u32) -> bool {
    status == NO_ERROR || status == ERROR_SUCCESS
}

fn tcp_row(row: &TcpRowOwnerPid) -> MIB_TCPROW_LH {
    MIB_TCPROW_LH {
        Anonymous: MIB_TCPROW_LH_0 { dwState: row.state },
        dwLocalAddr: row.local_addr,
        dwLocalPort: row.local_port,
        dwRemoteAddr: row.remote_addr,
        dwRemotePort: row.remote_port,
    }
}

fn row_bytes(row: &TcpRowOwnerPid) -> Option<(u64, u64)> {
    let mut tcp = tcp_row(row);
    if !enable_estats(&mut tcp) {
        return None;
    }
    read_rod(&mut tcp)
}

fn enable_estats(tcp: &mut MIB_TCPROW_LH) -> bool {
    let mut enable = TCP_ESTATS_DATA_RW_v0 {
        EnableCollection: true,
    };
    // SAFETY: `tcp` and `enable` are live stack values matching the IP Helper layout.
    let set = unsafe {
        SetPerTcpConnectionEStats(
            tcp,
            TCP_ESTATS_DATA,
            ptr::from_mut(&mut enable).cast(),
            0,
            size_of::<TCP_ESTATS_DATA_RW_v0>() as u32,
            0,
        )
    };
    win_ok(set)
}

fn read_rod(tcp: &mut MIB_TCPROW_LH) -> Option<(u64, u64)> {
    let mut rod = empty_rod();
    // SAFETY: `rod` is a zeroed ROD buffer; IP Helper writes only `RodSize` bytes.
    let got = unsafe {
        GetPerTcpConnectionEStats(
            tcp,
            TCP_ESTATS_DATA,
            ptr::null_mut(),
            0,
            0,
            ptr::null_mut(),
            0,
            0,
            ptr::from_mut(&mut rod).cast(),
            0,
            size_of::<TCP_ESTATS_DATA_ROD_v0>() as u32,
        )
    };
    win_ok(got).then_some((rod.DataBytesIn, rod.DataBytesOut))
}

fn empty_rod() -> TCP_ESTATS_DATA_ROD_v0 {
    TCP_ESTATS_DATA_ROD_v0 {
        DataBytesOut: 0,
        DataSegsOut: 0,
        DataBytesIn: 0,
        DataSegsIn: 0,
        SegsOut: 0,
        SegsIn: 0,
        SoftErrors: 0,
        SoftErrorReason: 0,
        SndUna: 0,
        SndNxt: 0,
        SndMax: 0,
        ThruBytesAcked: 0,
        RcvNxt: 0,
        ThruBytesReceived: 0,
    }
}

fn query_tcp_table(buf: *mut core::ffi::c_void, size: &mut u32) -> u32 {
    // SAFETY: caller owns `buf` for `*size` bytes, or passes null for a size probe.
    unsafe {
        GetExtendedTcpTable(
            buf,
            size,
            0,
            u32::from(AF_INET),
            TCP_TABLE_OWNER_PID_CONNECTIONS,
            0,
        )
    }
}

fn read_owner_pid_table() -> Vec<TcpRowOwnerPid> {
    let mut size = 0_u32;
    query_tcp_table(ptr::null_mut(), &mut size);
    if size < 4 {
        return Vec::new();
    }
    let mut buf = vec![0_u8; size as usize];
    let mut status = query_tcp_table(buf.as_mut_ptr().cast(), &mut size);
    if status == ERROR_INSUFFICIENT_BUFFER {
        buf.resize(size as usize, 0);
        status = query_tcp_table(buf.as_mut_ptr().cast(), &mut size);
    }
    if !win_ok(status) {
        return Vec::new();
    }
    parse_owner_pid_table(&buf)
}

fn parse_owner_pid_table(buf: &[u8]) -> Vec<TcpRowOwnerPid> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let count = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let row_size = size_of::<TcpRowOwnerPid>();
    let mut rows = Vec::new();
    for index in 0..count {
        let start = 4 + index * row_size;
        let Some(row) = parse_row(buf.get(start..start.saturating_add(row_size))) else {
            break;
        };
        rows.push(row);
    }
    rows
}

fn parse_row(slice: Option<&[u8]>) -> Option<TcpRowOwnerPid> {
    let slice = slice.filter(|item| item.len() == size_of::<TcpRowOwnerPid>())?;
    Some(TcpRowOwnerPid {
        state: u32_le(slice, 0),
        local_addr: u32_le(slice, 4),
        local_port: u32_le(slice, 8),
        remote_addr: u32_le(slice, 12),
        remote_port: u32_le(slice, 16),
        owning_pid: u32_le(slice, 20),
    })
}

fn u32_le(slice: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(slice[offset..offset + 4].try_into().unwrap_or([0; 4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skips_truncated_rows() {
        let mut buf = vec![1, 0, 0, 0];
        buf.extend_from_slice(&[0_u8; 8]);
        assert!(parse_owner_pid_table(&buf).is_empty());
    }

    #[test]
    fn parse_one_row() {
        let mut buf = vec![1, 0, 0, 0];
        buf.extend_from_slice(&5_u32.to_le_bytes());
        buf.extend_from_slice(&0x0100007f_u32.to_le_bytes());
        buf.extend_from_slice(&0x5000_u32.to_le_bytes());
        buf.extend_from_slice(&0_u32.to_le_bytes());
        buf.extend_from_slice(&0_u32.to_le_bytes());
        buf.extend_from_slice(&4242_u32.to_le_bytes());
        let rows = parse_owner_pid_table(&buf);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].owning_pid, 4242);
        assert_eq!(rows[0].state, 5);
    }
}
