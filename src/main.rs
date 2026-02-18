//! dmcp - MCP Manager CLI

use clap::{Parser, Subcommand};
use dmcp::{list_servers, Paths};

#[derive(Parser)]
#[command(name = "dmcp")]
#[command(about = "MCP Manager - discover, manage, and invoke MCP servers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed MCP servers
    List {
        /// Include user-scope servers
        #[arg(long)]
        user: bool,

        /// Include system-scope servers
        #[arg(long)]
        system: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show resolved paths (for debugging)
    Paths,
}

fn main() {
    let cli = Cli::parse();
    let paths = Paths::resolve();
    let debug = cli.debug;

    match cli.command {
        Commands::Paths => {
            println!("User install dir:  {}", paths.user_install_dir().display());
            println!("System install dir: {}", paths.system_install_dir().display());
            let user_index = paths.user_install_dir().join("index.json");
            let system_index = paths.system_install_dir().join("index.json");
            println!("User index exists:  {}", user_index.exists());
            println!("System index exists: {}", system_index.exists());
        }
        Commands::List { user, system, json } => {
            let include_user = user || (!user && !system);
            let include_system = system || (!user && !system);

            let servers = list_servers(&paths, include_user, include_system, debug);

            if json {
                let output = serde_json::to_string_pretty(&servers).unwrap();
                println!("{output}");
            } else {
                if servers.is_empty() {
                    println!("No MCP servers installed.");
                    return;
                }
                println!("{:<36} {:<24} {:<10} {:<12} {}", "ID", "NAME", "VERSION", "TRANSPORT", "SCOPE");
                println!("{}", "-".repeat(95));
                for s in servers {
                    let scope = match s.scope {
                        dmcp::discovery::Scope::User => "user",
                        dmcp::discovery::Scope::System => "system",
                    };
                    println!("{:<36} {:<24} {:<10} {:<12} {}", s.id, truncate(&s.name, 22), s.version, s.transport_type, scope);
                }
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
