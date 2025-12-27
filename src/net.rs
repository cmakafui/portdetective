//! Port to PID discovery using netstat2

use crate::cli::ProtocolFilter;
use crate::error::{PortDetectiveError, Result};
use crate::model::Protocol;
use netstat2::{
    AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpState, get_sockets_info,
};
use std::collections::HashMap;

/// A socket bound to a port
#[derive(Debug, Clone)]
pub struct BoundSocket {
    pub pid: u32,
    pub port: u16,
    pub protocol: Protocol,
    #[allow(dead_code)]
    pub local_addr: String,
}

/// Find all processes listening on a specific port
pub fn find_processes_by_port(port: u16, filter: ProtocolFilter) -> Result<Vec<BoundSocket>> {
    let sockets = get_listening_sockets(filter)?;
    Ok(sockets.into_iter().filter(|s| s.port == port).collect())
}

/// Get all listening sockets
pub fn get_listening_sockets(filter: ProtocolFilter) -> Result<Vec<BoundSocket>> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;

    let proto_flags = match filter {
        ProtocolFilter::TcpOnly => ProtocolFlags::TCP,
        ProtocolFilter::UdpOnly => ProtocolFlags::UDP,
        ProtocolFilter::Both => ProtocolFlags::TCP | ProtocolFlags::UDP,
    };

    let sockets = get_sockets_info(af_flags, proto_flags)
        .map_err(|e| PortDetectiveError::NetworkError(e.to_string()))?;

    let mut result = Vec::new();

    for socket in sockets {
        if let Some(bound) = extract_listening_socket(&socket) {
            result.push(bound);
        }
    }

    Ok(result)
}

/// Get all listening ports grouped by port number
pub fn get_listening_ports(filter: ProtocolFilter) -> Result<HashMap<u16, Vec<BoundSocket>>> {
    let sockets = get_listening_sockets(filter)?;
    let mut map: HashMap<u16, Vec<BoundSocket>> = HashMap::new();

    for socket in sockets {
        map.entry(socket.port).or_default().push(socket);
    }

    Ok(map)
}

fn extract_listening_socket(socket: &SocketInfo) -> Option<BoundSocket> {
    let pids = &socket.associated_pids;
    if pids.is_empty() {
        return None;
    }

    match &socket.protocol_socket_info {
        ProtocolSocketInfo::Tcp(tcp) => {
            // Only listening sockets
            if tcp.state != TcpState::Listen {
                return None;
            }
            Some(BoundSocket {
                pid: pids[0],
                port: tcp.local_port,
                protocol: Protocol::Tcp,
                local_addr: format!("{}", tcp.local_addr),
            })
        }
        ProtocolSocketInfo::Udp(udp) => {
            // UDP sockets don't have state, include all bound ones
            Some(BoundSocket {
                pid: pids[0],
                port: udp.local_port,
                protocol: Protocol::Udp,
                local_addr: format!("{}", udp.local_addr),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_listening_sockets_both() {
        let result = get_listening_sockets(ProtocolFilter::Both);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_listening_sockets_tcp_only() {
        let result = get_listening_sockets(ProtocolFilter::TcpOnly);
        assert!(result.is_ok());

        // All returned sockets should be TCP
        for socket in result.unwrap() {
            assert_eq!(socket.protocol, Protocol::Tcp);
        }
    }

    #[test]
    fn test_get_listening_sockets_udp_only() {
        let result = get_listening_sockets(ProtocolFilter::UdpOnly);
        assert!(result.is_ok());

        // All returned sockets should be UDP
        for socket in result.unwrap() {
            assert_eq!(socket.protocol, Protocol::Udp);
        }
    }

    #[test]
    fn test_find_processes_by_port_unlikely() {
        // Port 65535 is unlikely to have a listener in test environments
        let result = find_processes_by_port(65535, ProtocolFilter::Both);
        assert!(result.is_ok());
        // We just verify the function works, not that it's empty
        // (some systems may have something on high ports)
    }

    #[test]
    fn test_get_listening_ports_returns_hashmap() {
        let result = get_listening_ports(ProtocolFilter::Both);
        assert!(result.is_ok());

        let map = result.unwrap();
        // Each port key should have at least one socket
        for (_port, sockets) in &map {
            assert!(!sockets.is_empty());
        }
    }

    #[test]
    fn test_bound_socket_fields() {
        // Create a mock BoundSocket to verify structure
        let socket = BoundSocket {
            pid: 1234,
            port: 8080,
            protocol: Protocol::Tcp,
            local_addr: "127.0.0.1".to_string(),
        };

        assert_eq!(socket.pid, 1234);
        assert_eq!(socket.port, 8080);
        assert_eq!(socket.protocol, Protocol::Tcp);
        assert_eq!(socket.local_addr, "127.0.0.1");
    }
}
