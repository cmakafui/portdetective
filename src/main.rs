//! Port Detective — What's running on this port?
//!
//! A tiny CLI that answers: "What's running on port 3000 right now, and how do I safely kill it?"

mod cli;
mod error;
mod model;
mod net;
mod output;
mod proc;

use clap::Parser;
use cli::{Cli, Commands, ProtocolFilter};
use error::{PortDetectiveError, Result};
use model::{PortEntry, PortReport, Protocol};
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Kill { port, force, no_prompt }) => {
            run_kill(*port, *force, *no_prompt, cli.protocol_filter(), cli.json)
        }
        Some(Commands::List) => run_list(cli.protocol_filter(), cli.json),
        Some(Commands::Inspect { port }) => run_inspect(*port, cli.protocol_filter(), cli.json),
        None => {
            // Default: if port provided, inspect it
            if let Some(port) = cli.port {
                run_inspect(port, cli.protocol_filter(), cli.json)
            } else {
                // No port provided, show help hint
                eprintln!("Usage: portdetective <PORT>");
                eprintln!("       portdetective list");
                eprintln!("       portdetective kill <PORT>");
                eprintln!();
                eprintln!("Run `portdetective --help` for more options.");
                return ExitCode::from(1);
            }
        }
    };

    match result {
        Ok(code) => code,
        Err(e) => {
            output::print_error(&e.to_string());
            match e {
                PortDetectiveError::PortFree(_) => ExitCode::from(0),
                PortDetectiveError::PermissionDenied(_) => ExitCode::from(2),
                PortDetectiveError::ProcessNotFound(_) => ExitCode::from(3),
                PortDetectiveError::Cancelled => ExitCode::from(4),
                _ => ExitCode::from(1),
            }
        }
    }
}

/// Inspect what's running on a port
fn run_inspect(port: u16, filter: ProtocolFilter, json: bool) -> Result<ExitCode> {
    let protocol = match filter {
        ProtocolFilter::TcpOnly => Protocol::Tcp,
        ProtocolFilter::UdpOnly => Protocol::Udp,
        ProtocolFilter::Both => Protocol::Both,
    };

    let sockets = net::find_processes_by_port(port, filter)?;

    if sockets.is_empty() {
        let report = PortReport::free(port, protocol);
        if json {
            output::print_report_json(&report);
        } else {
            output::print_report(&report);
        }
        return Ok(ExitCode::from(0));
    }

    // Gather process info for each bound socket, deduplicating by PID
    let mut processes = Vec::new();
    let mut seen_pids = std::collections::HashSet::new();
    for socket in &sockets {
        if seen_pids.contains(&socket.pid) {
            continue;
        }
        match proc::inspect(socket.pid, socket.protocol) {
            Ok(info) => {
                seen_pids.insert(socket.pid);
                processes.push(info);
            }
            Err(_) => {
                // Process may have exited between discovery and inspection
                continue;
            }
        }
    }

    if processes.is_empty() {
        let report = PortReport::free(port, protocol);
        if json {
            output::print_report_json(&report);
        } else {
            output::print_report(&report);
        }
        return Ok(ExitCode::from(0));
    }

    let report = PortReport::in_use(port, protocol, processes);
    if json {
        output::print_report_json(&report);
    } else {
        output::print_report(&report);
    }

    Ok(ExitCode::from(1)) // Port is in use
}

/// Kill the process on a port
fn run_kill(port: u16, force: bool, no_prompt: bool, filter: ProtocolFilter, json: bool) -> Result<ExitCode> {
    let sockets = net::find_processes_by_port(port, filter)?;

    if sockets.is_empty() {
        if json {
            let protocol = match filter {
                ProtocolFilter::TcpOnly => Protocol::Tcp,
                ProtocolFilter::UdpOnly => Protocol::Udp,
                ProtocolFilter::Both => Protocol::Both,
            };
            let report = PortReport::free(port, protocol);
            output::print_report_json(&report);
        } else {
            output::print_report(&PortReport::free(port, Protocol::Both));
        }
        return Ok(ExitCode::from(0));
    }

    // Get first socket's process info
    let socket = &sockets[0];
    let info = proc::inspect(socket.pid, socket.protocol)?;

    if !no_prompt {
        output::print_kill_prompt(&info);
        
        print!("Are you sure you want to kill PID {}? [y/N]: ", info.pid);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if !input.trim().eq_ignore_ascii_case("y") {
            output::print_kill_cancelled();
            return Err(PortDetectiveError::Cancelled);
        }
    }

    proc::kill_process(info.pid, force)?;
    output::print_kill_success(info.pid, force);

    Ok(ExitCode::from(0))
}

/// List all listening ports
fn run_list(filter: ProtocolFilter, json: bool) -> Result<ExitCode> {
    let ports_map = net::get_listening_ports(filter)?;
    
    let mut entries: Vec<PortEntry> = Vec::new();
    let mut seen: std::collections::HashSet<(u16, u32)> = std::collections::HashSet::new();

    for (port, sockets) in ports_map {
        for socket in sockets {
            // Deduplicate by (port, pid)
            if seen.contains(&(port, socket.pid)) {
                continue;
            }
            if let Ok(info) = proc::inspect(socket.pid, socket.protocol) {
                seen.insert((port, socket.pid));
                let cmd = if info.command.is_empty() {
                    info.name.clone()
                } else {
                    info.command.join(" ")
                };
                
                entries.push(PortEntry {
                    port,
                    protocol: socket.protocol,
                    pid: socket.pid,
                    name: info.name,
                    user: info.user,
                    command: cmd,
                });
            }
        }
    }

    // Sort by port number
    entries.sort_by_key(|e| e.port);

    if json {
        output::print_port_list_json(&entries);
    } else {
        output::print_port_list(&entries);
    }

    Ok(ExitCode::from(0))
}
