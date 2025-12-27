//! Process inspection using sysinfo

use crate::error::{PortDetectiveError, Result};
use crate::model::{ProcessInfo, Protocol};
use chrono::{DateTime, Local, TimeZone};
use sysinfo::{Pid, System, Users};

/// Inspect a process by PID and gather detailed information
pub fn inspect(pid: u32, protocol: Protocol) -> Result<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
    );

    let process = sys
        .process(Pid::from_u32(pid))
        .ok_or(PortDetectiveError::ProcessNotFound(pid))?;

    let users = Users::new_with_refreshed_list();

    // Get user name
    let user = process
        .user_id()
        .and_then(|uid| users.get_user_by_id(uid))
        .map(|u| u.name().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get command line
    let command: Vec<String> = process
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    // Get working directory
    let cwd = process.cwd().map(|p| p.to_path_buf());

    // Get start time
    let started = process_start_time(process.start_time());

    // Extract values we need before dropping the borrow
    let name = process.name().to_string_lossy().to_string();
    let parent = process.parent();

    // Now we can borrow sys mutably for parent info
    let (parent_pid, parent_name) = get_parent_info(&mut sys, parent);

    Ok(ProcessInfo {
        pid,
        name,
        user,
        command,
        cwd,
        parent_pid,
        parent_name,
        started,
        protocol,
    })
}

/// Get parent process name and PID
fn get_parent_info(sys: &mut System, parent_pid: Option<Pid>) -> (Option<u32>, Option<String>) {
    match parent_pid {
        Some(ppid) => {
            sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[ppid]), true);
            let parent_name = sys
                .process(ppid)
                .map(|p| p.name().to_string_lossy().to_string());
            (Some(ppid.as_u32()), parent_name)
        }
        None => (None, None),
    }
}

/// Convert Unix timestamp to local datetime
fn process_start_time(start_time: u64) -> Option<DateTime<Local>> {
    if start_time == 0 {
        return None;
    }
    Local.timestamp_opt(start_time as i64, 0).single()
}

/// Kill a process by PID
pub fn kill_process(pid: u32, force: bool) -> Result<()> {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid as NixPid;

    let signal = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    let nix_pid = NixPid::from_raw(pid as i32);

    kill(nix_pid, signal).map_err(|e| {
        if e == nix::errno::Errno::EPERM {
            PortDetectiveError::PermissionDenied(format!(
                "Cannot kill PID {}. Try running with elevated permissions.",
                pid
            ))
        } else {
            PortDetectiveError::KillFailed {
                pid,
                reason: e.to_string(),
            }
        }
    })
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn get_cwd_linux(pid: u32) -> Option<std::path::PathBuf> {
    procfs::process::Process::new(pid as i32)
        .ok()
        .and_then(|p| p.cwd().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_inspect_current_process() {
        let pid = process::id();
        let result = inspect(pid, Protocol::Tcp);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.pid, pid);
        assert!(!info.name.is_empty());
        assert_eq!(info.protocol, Protocol::Tcp);
    }

    #[test]
    fn test_inspect_preserves_protocol() {
        let pid = process::id();

        let tcp_result = inspect(pid, Protocol::Tcp);
        assert_eq!(tcp_result.unwrap().protocol, Protocol::Tcp);

        let udp_result = inspect(pid, Protocol::Udp);
        assert_eq!(udp_result.unwrap().protocol, Protocol::Udp);
    }

    #[test]
    fn test_inspect_nonexistent_process() {
        // PID 0 is kernel/scheduler, unlikely to be inspectable as regular process
        // Use a very high PID that's unlikely to exist
        let result = inspect(u32::MAX, Protocol::Tcp);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::PortDetectiveError::ProcessNotFound(_)
        ));
    }

    #[test]
    fn test_inspect_has_user() {
        let pid = process::id();
        let info = inspect(pid, Protocol::Tcp).unwrap();
        // User should be set (might be "unknown" in edge cases, but not empty)
        assert!(!info.user.is_empty());
    }

    #[test]
    fn test_inspect_command_field_exists() {
        let pid = process::id();
        let info = inspect(pid, Protocol::Tcp).unwrap();
        // Command is a Vec<String> - verify it's accessible
        // Note: On some platforms, command may be empty for certain processes
        let _ = info.command.len();
    }

    #[test]
    fn test_inspect_has_parent() {
        let pid = process::id();
        let info = inspect(pid, Protocol::Tcp).unwrap();
        // Most processes have a parent (except init/launchd)
        // The test process should definitely have one
        assert!(info.parent_pid.is_some());
    }

    #[test]
    fn test_process_start_time_zero_returns_none() {
        // The internal helper should return None for timestamp 0
        let result = process_start_time(0);
        assert!(result.is_none());
    }

    #[test]
    fn test_process_start_time_valid() {
        // A reasonable Unix timestamp (Jan 1, 2020)
        let result = process_start_time(1577836800);
        assert!(result.is_some());

        let dt = result.unwrap();
        // Verify the datetime is reasonable (year 2020)
        assert!(dt.format("%Y").to_string() == "2020");
    }
}
