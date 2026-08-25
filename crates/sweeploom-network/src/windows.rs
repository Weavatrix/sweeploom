//! Owner-PID sockets via `netstat2` (IPHLPAPI). Byte rates stay unavailable.

use std::collections::HashMap;

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};

use super::Endpoint;

/// Load TCP/UDP endpoints keyed by owning PID.
#[must_use]
pub fn load() -> HashMap<u32, Vec<Endpoint>> {
    let Ok(sockets) = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    ) else {
        return HashMap::new();
    };
    let mut by_pid: HashMap<u32, Vec<Endpoint>> = HashMap::new();
    for socket in sockets {
        let endpoint = match socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => Endpoint {
                local_port: tcp.local_port,
                remote: (tcp.state != TcpState::Listen)
                    .then(|| format!("{}:{}", tcp.remote_addr, tcp.remote_port)),
                listening: tcp.state == TcpState::Listen,
            },
            ProtocolSocketInfo::Udp(udp) => Endpoint {
                local_port: udp.local_port,
                remote: None,
                listening: true,
            },
        };
        for pid in socket.associated_pids {
            by_pid.entry(pid).or_default().push(endpoint.clone());
        }
    }
    by_pid
}
