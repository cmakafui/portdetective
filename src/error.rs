//! Error types for Port Detective

use thiserror::Error;

/// All errors that can occur in Port Detective
#[derive(Debug, Error)]
pub enum PortDetectiveError {
    #[allow(dead_code)]
    #[error("Port {0} is not valid (must be 1-65535)")]
    InvalidPort(u16),

    #[error("Could not enumerate network sockets: {0}")]
    NetworkError(String),

    #[error("Process {0} not found or no longer running")]
    ProcessNotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Kill failed for PID {pid}: {reason}")]
    KillFailed { pid: u32, reason: String },

    #[allow(dead_code)]
    #[error("No process found on port {0}")]
    PortFree(u16),

    #[error("Operation cancelled by user")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, PortDetectiveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_port_message() {
        let err = PortDetectiveError::InvalidPort(0);
        assert_eq!(err.to_string(), "Port 0 is not valid (must be 1-65535)");
    }

    #[test]
    fn test_network_error_message() {
        let err = PortDetectiveError::NetworkError("socket read failed".to_string());
        assert_eq!(
            err.to_string(),
            "Could not enumerate network sockets: socket read failed"
        );
    }

    #[test]
    fn test_process_not_found_message() {
        let err = PortDetectiveError::ProcessNotFound(12345);
        assert_eq!(
            err.to_string(),
            "Process 12345 not found or no longer running"
        );
    }

    #[test]
    fn test_permission_denied_message() {
        let err = PortDetectiveError::PermissionDenied("Cannot kill PID 1".to_string());
        assert_eq!(err.to_string(), "Permission denied: Cannot kill PID 1");
    }

    #[test]
    fn test_kill_failed_message() {
        let err = PortDetectiveError::KillFailed {
            pid: 999,
            reason: "ESRCH".to_string(),
        };
        assert_eq!(err.to_string(), "Kill failed for PID 999: ESRCH");
    }

    #[test]
    fn test_port_free_message() {
        let err = PortDetectiveError::PortFree(8080);
        assert_eq!(err.to_string(), "No process found on port 8080");
    }

    #[test]
    fn test_cancelled_message() {
        let err = PortDetectiveError::Cancelled;
        assert_eq!(err.to_string(), "Operation cancelled by user");
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_ok() -> Result<u32> {
            Ok(42)
        }

        fn returns_err() -> Result<u32> {
            Err(PortDetectiveError::Cancelled)
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }
}
