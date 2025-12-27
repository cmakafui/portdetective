//! CLI definitions using clap derive

use clap::{Parser, Subcommand};

/// 🔎 Port Detective — What's running on this port?
#[derive(Parser, Debug)]
#[command(
    name = "portdetective",
    author,
    version,
    about = "🔎 What's running on this port?",
    long_about = "A tiny CLI that answers: \"What's running on port 3000 right now, and how do I safely kill it?\"\n\nReplaces lsof/netstat/ps incantations with one clear command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Port number to inspect (shorthand for `portdetective inspect <PORT>`)
    #[arg(value_name = "PORT")]
    pub port: Option<u16>,

    /// Output as JSON
    #[arg(long, short, global = true)]
    pub json: bool,

    /// Only show TCP connections
    #[arg(long, global = true, conflicts_with = "udp")]
    pub tcp: bool,

    /// Only show UDP connections
    #[arg(long, global = true, conflicts_with = "tcp")]
    pub udp: bool,
}

impl Cli {
    pub fn protocol_filter(&self) -> ProtocolFilter {
        match (self.tcp, self.udp) {
            (true, false) => ProtocolFilter::TcpOnly,
            (false, true) => ProtocolFilter::UdpOnly,
            _ => ProtocolFilter::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFilter {
    TcpOnly,
    UdpOnly,
    Both,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect what's running on a specific port
    #[command(visible_alias = "i")]
    Inspect {
        /// Port number to inspect
        port: u16,
    },

    /// Kill the process running on a specific port
    #[command(visible_alias = "k")]
    Kill {
        /// Port number to kill
        port: u16,

        /// Send SIGKILL instead of SIGTERM
        #[arg(long, short)]
        force: bool,

        /// Don't prompt for confirmation (for scripting)
        #[arg(long, short = 'y')]
        no_prompt: bool,
    },

    /// List all listening ports
    #[command(visible_alias = "l", visible_alias = "ls")]
    List,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_bare_port_argument() {
        let cli = Cli::parse_from(["portdetective", "3000"]);
        assert_eq!(cli.port, Some(3000));
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_inspect_subcommand() {
        let cli = Cli::parse_from(["portdetective", "inspect", "8080"]);
        assert!(matches!(cli.command, Some(Commands::Inspect { port: 8080 })));
    }

    #[test]
    fn test_inspect_alias() {
        let cli = Cli::parse_from(["portdetective", "i", "8080"]);
        assert!(matches!(cli.command, Some(Commands::Inspect { port: 8080 })));
    }

    #[test]
    fn test_kill_subcommand_defaults() {
        let cli = Cli::parse_from(["portdetective", "kill", "3000"]);
        match cli.command {
            Some(Commands::Kill { port, force, no_prompt }) => {
                assert_eq!(port, 3000);
                assert!(!force);
                assert!(!no_prompt);
            }
            _ => panic!("Expected Kill command"),
        }
    }

    #[test]
    fn test_kill_with_force_flag() {
        let cli = Cli::parse_from(["portdetective", "kill", "3000", "--force"]);
        match cli.command {
            Some(Commands::Kill { force, .. }) => assert!(force),
            _ => panic!("Expected Kill command"),
        }
    }

    #[test]
    fn test_kill_with_no_prompt_flag() {
        let cli = Cli::parse_from(["portdetective", "kill", "3000", "-y"]);
        match cli.command {
            Some(Commands::Kill { no_prompt, .. }) => assert!(no_prompt),
            _ => panic!("Expected Kill command"),
        }
    }

    #[test]
    fn test_list_subcommand() {
        let cli = Cli::parse_from(["portdetective", "list"]);
        assert!(matches!(cli.command, Some(Commands::List)));
    }

    #[test]
    fn test_list_aliases() {
        let cli_l = Cli::parse_from(["portdetective", "l"]);
        let cli_ls = Cli::parse_from(["portdetective", "ls"]);
        assert!(matches!(cli_l.command, Some(Commands::List)));
        assert!(matches!(cli_ls.command, Some(Commands::List)));
    }

    #[test]
    fn test_json_flag_global() {
        let cli = Cli::parse_from(["portdetective", "--json", "3000"]);
        assert!(cli.json);

        let cli2 = Cli::parse_from(["portdetective", "list", "--json"]);
        assert!(cli2.json);
    }

    #[test]
    fn test_tcp_only_filter() {
        let cli = Cli::parse_from(["portdetective", "--tcp", "3000"]);
        assert!(cli.tcp);
        assert!(!cli.udp);
        assert_eq!(cli.protocol_filter(), ProtocolFilter::TcpOnly);
    }

    #[test]
    fn test_udp_only_filter() {
        let cli = Cli::parse_from(["portdetective", "--udp", "3000"]);
        assert!(!cli.tcp);
        assert!(cli.udp);
        assert_eq!(cli.protocol_filter(), ProtocolFilter::UdpOnly);
    }

    #[test]
    fn test_default_filter_is_both() {
        let cli = Cli::parse_from(["portdetective", "3000"]);
        assert_eq!(cli.protocol_filter(), ProtocolFilter::Both);
    }

    #[test]
    fn test_tcp_udp_conflict() {
        let result = Cli::try_parse_from(["portdetective", "--tcp", "--udp", "3000"]);
        assert!(result.is_err());
    }
}
