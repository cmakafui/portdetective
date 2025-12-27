//! Data models for Port Detective

use chrono::{DateTime, Local};
use serde::Serialize;
use std::path::PathBuf;

/// Information about a process bound to a port
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Local>>,
    pub protocol: Protocol,
}

/// Report about a port's status
#[derive(Debug, Clone, Serialize)]
pub struct PortReport {
    pub port: u16,
    pub protocol: Protocol,
    pub status: PortStatus,
    pub processes: Vec<ProcessInfo>,
}

impl PortReport {
    pub fn free(port: u16, protocol: Protocol) -> Self {
        Self {
            port,
            protocol,
            status: PortStatus::Free,
            processes: Vec::new(),
        }
    }

    pub fn in_use(port: u16, protocol: Protocol, processes: Vec<ProcessInfo>) -> Self {
        Self {
            port,
            protocol,
            status: PortStatus::InUse,
            processes,
        }
    }
}

/// Whether a port is in use
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortStatus {
    Free,
    InUse,
}

/// Network protocol
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    Both,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
            Protocol::Both => write!(f, "tcp/udp"),
        }
    }
}

/// Entry in the port list
#[derive(Debug, Clone, Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_report_free() {
        let report = PortReport::free(8080, Protocol::Tcp);
        assert_eq!(report.port, 8080);
        assert_eq!(report.protocol, Protocol::Tcp);
        assert_eq!(report.status, PortStatus::Free);
        assert!(report.processes.is_empty());
    }

    #[test]
    fn test_port_report_in_use() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node".to_string(),
            user: "dev".to_string(),
            command: vec!["node".to_string(), "server.js".to_string()],
            cwd: None,
            parent_pid: None,
            parent_name: None,
            started: None,
            protocol: Protocol::Tcp,
        };

        let report = PortReport::in_use(3000, Protocol::Tcp, vec![process]);
        assert_eq!(report.port, 3000);
        assert_eq!(report.status, PortStatus::InUse);
        assert_eq!(report.processes.len(), 1);
        assert_eq!(report.processes[0].name, "node");
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", Protocol::Tcp), "tcp");
        assert_eq!(format!("{}", Protocol::Udp), "udp");
        assert_eq!(format!("{}", Protocol::Both), "tcp/udp");
    }

    #[test]
    fn test_port_report_json_serialization() {
        let report = PortReport::free(443, Protocol::Tcp);
        let json = serde_json::to_string(&report).unwrap();

        assert!(json.contains("\"port\":443"));
        assert!(json.contains("\"status\":\"free\""));
        assert!(json.contains("\"protocol\":\"tcp\""));
    }

    #[test]
    fn test_process_info_json_skips_none_fields() {
        let process = ProcessInfo {
            pid: 100,
            name: "test".to_string(),
            user: "root".to_string(),
            command: vec![],
            cwd: None,
            parent_pid: None,
            parent_name: None,
            started: None,
            protocol: Protocol::Udp,
        };

        let json = serde_json::to_string(&process).unwrap();

        // These optional fields should not appear when None
        assert!(!json.contains("cwd"));
        assert!(!json.contains("parent_pid"));
        assert!(!json.contains("parent_name"));
        assert!(!json.contains("started"));

        // Required fields should always appear
        assert!(json.contains("\"pid\":100"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"protocol\":\"udp\""));
    }

    #[test]
    fn test_process_info_json_includes_some_fields() {
        let process = ProcessInfo {
            pid: 200,
            name: "nginx".to_string(),
            user: "www".to_string(),
            command: vec!["nginx".to_string(), "-g".to_string()],
            cwd: Some(std::path::PathBuf::from("/var/www")),
            parent_pid: Some(1),
            parent_name: Some("systemd".to_string()),
            started: None,
            protocol: Protocol::Tcp,
        };

        let json = serde_json::to_string(&process).unwrap();

        assert!(json.contains("\"cwd\":\"/var/www\""));
        assert!(json.contains("\"parent_pid\":1"));
        assert!(json.contains("\"parent_name\":\"systemd\""));
    }

    #[test]
    fn test_port_entry_serialization() {
        let entry = PortEntry {
            port: 22,
            protocol: Protocol::Tcp,
            pid: 500,
            name: "sshd".to_string(),
            user: "root".to_string(),
            command: "/usr/sbin/sshd -D".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"port\":22"));
        assert!(json.contains("\"name\":\"sshd\""));
    }
}
