//! Output rendering for human and JSON formats

use crate::model::{PortEntry, PortReport, PortStatus, ProcessInfo};
use owo_colors::OwoColorize;

/// Print a port report in human-readable format
pub fn print_report(report: &PortReport) {
    match report.status {
        PortStatus::Free => print_free_port(report.port),
        PortStatus::InUse => print_in_use_port(report),
    }
}

fn print_free_port(port: u16) {
    println!(
        "{} Port {} is {} (no listening process found)",
        "✅".green(),
        port.to_string().cyan().bold(),
        "free".green().bold()
    );
}

fn print_in_use_port(report: &PortReport) {
    println!(
        "{} Port {} ({}) is {}",
        "🔎".yellow(),
        report.port.to_string().cyan().bold(),
        report.protocol.to_string().dimmed(),
        "in use".red().bold()
    );
    println!();

    for process in &report.processes {
        print_process_details(process);
    }
}

fn print_process_details(info: &ProcessInfo) {
    // Process name
    println!("{}    {}", "Process:".bold(), info.name.green().bold());

    // PID
    println!("{}        {}", "PID:".bold(), info.pid.to_string().yellow());

    // User
    println!("{}       {}", "User:".bold(), info.user.cyan());

    // Command
    let cmd = if info.command.is_empty() {
        info.name.clone()
    } else {
        info.command.join(" ")
    };
    println!("{}    {}", "Command:".bold(), cmd);

    // Working directory
    if let Some(cwd) = &info.cwd {
        println!(
            "{}        {}",
            "CWD:".bold(),
            cwd.display().to_string().dimmed()
        );
    }

    // Parent
    if let (Some(ppid), Some(pname)) = (&info.parent_pid, &info.parent_name) {
        println!(
            "{}     {} (PID {})",
            "Parent:".bold(),
            pname.blue(),
            ppid.to_string().dimmed()
        );
    }

    // Start time
    if let Some(started) = &info.started {
        println!(
            "{}    {}",
            "Started:".bold(),
            started.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
        );
    }

    println!();
    print_kill_hints(info.pid);
}

fn print_kill_hints(pid: u32) {
    println!("{}", "Suggested kill:".bold().underline());
    println!("  {} {}", "kill".dimmed(), pid.to_string().yellow());
    println!("  {}", "# or force if needed:".dimmed().italic());
    println!("  {} {}", "kill -9".dimmed(), pid.to_string().yellow());
}

/// Print a port report as JSON
pub fn print_report_json(report: &PortReport) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
    println!("{}", json);
}

/// Print a list of ports in table format
pub fn print_port_list(entries: &[PortEntry]) {
    if entries.is_empty() {
        println!("{} No listening ports found", "✅".green());
        return;
    }

    // Header
    println!(
        "{:<7} {:<6} {:<8} {:<12} {:<10} {}",
        "PORT".bold().underline(),
        "PROTO".bold().underline(),
        "PID".bold().underline(),
        "PROCESS".bold().underline(),
        "USER".bold().underline(),
        "COMMAND".bold().underline()
    );

    for entry in entries {
        let cmd_display = if entry.command.len() > 50 {
            format!("{}...", &entry.command[..47])
        } else {
            entry.command.clone()
        };

        println!(
            "{:<7} {:<6} {:<8} {:<12} {:<10} {}",
            entry.port.to_string().cyan(),
            entry.protocol.to_string().dimmed(),
            entry.pid.to_string().yellow(),
            entry.name.green(),
            entry.user.blue(),
            cmd_display.dimmed()
        );
    }

    println!();
    println!(
        "{} {} listening port(s) found",
        "📊".blue(),
        entries.len().to_string().bold()
    );
}

/// Print port list as JSON
pub fn print_port_list_json(entries: &[PortEntry]) {
    let json = serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[]".to_string());
    println!("{}", json);
}

/// Print kill confirmation prompt
pub fn print_kill_prompt(info: &ProcessInfo) {
    println!(
        "{} Port {} ({}) is in use by:",
        "🔎".yellow(),
        info.protocol.to_string().dimmed(),
        info.protocol.to_string().dimmed()
    );
    println!(
        "  {} (PID {})",
        info.name.green().bold(),
        info.pid.to_string().yellow()
    );

    let cmd = if info.command.is_empty() {
        info.name.clone()
    } else {
        info.command.join(" ")
    };
    println!("  Command: {}", cmd);

    if let Some(cwd) = &info.cwd {
        println!("  CWD:     {}", cwd.display().to_string().dimmed());
    }
    println!();
}

/// Print kill success message
pub fn print_kill_success(pid: u32, force: bool) {
    let signal = if force { "SIGKILL" } else { "SIGTERM" };
    println!(
        "{} Sent {} to PID {}",
        "✅".green(),
        signal.yellow(),
        pid.to_string().bold()
    );
}

/// Print kill cancelled message
pub fn print_kill_cancelled() {
    println!("{} Kill cancelled", "❌".red());
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}
