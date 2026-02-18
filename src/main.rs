//! dmcp - MCP Manager CLI

use clap::{Parser, Subcommand};
use dmcp::config;
use dmcp::elevation::{is_elevated, is_system_scope, re_exec_with_pkexec};
use dmcp::{add_source, discovery, fetch_server_from_registry, get_server, install, list_registry_servers, list_registry_servers_from_url, list_servers, list_sources, remove_source, scope_from_registry_server, set_config_value, uninstall, Paths};

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
    /// List installed MCP servers (default: both user and system)
    List {
        /// Include user-scope servers only
        #[arg(long)]
        user: bool,

        /// Include system-scope servers only
        #[arg(long)]
        system: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show detailed info for a server
    Info {
        /// Server ID (e.g. com.example.calculator)
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get or set server configuration
    Config {
        /// Server ID
        id: String,

        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage registry sources
    Sources {
        #[command(subcommand)]
        action: SourcesAction,
    },

    /// Install an MCP server from registry
    Install {
        /// Server ID to install
        id: String,

        /// Install to system scope (requires elevation)
        #[arg(long)]
        system: bool,
    },

    /// Uninstall an MCP server
    Uninstall {
        /// Server ID to uninstall
        id: String,
    },

    /// Browse servers available in registry sources (or a specific registry URL)
    Browse {
        /// Registry URL to browse (omit to use configured sources)
        url: Option<String>,

        /// Show user-scope sources only (ignored when URL is given)
        #[arg(long)]
        user: bool,

        /// Show system-scope sources only (ignored when URL is given)
        #[arg(long)]
        system: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show resolved paths (for debugging)
    Paths,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get config value(s)
    Get {
        /// Specific key (omit for all)
        key: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set a config value
    Set {
        /// Config key
        key: String,

        /// Config value
        value: String,
    },
}

#[derive(Subcommand)]
enum SourcesAction {
    /// List registry source URLs
    List {
        /// Show user-scope sources only
        #[arg(long)]
        user: bool,

        /// Show system-scope sources only
        #[arg(long)]
        system: bool,
    },

    /// Add a registry source URL
    Add {
        /// URL of the registry JSON file
        url: String,

        /// Add to user scope (default)
        #[arg(long)]
        user: bool,

        /// Add to system scope (requires elevation)
        #[arg(long)]
        system: bool,
    },

    /// Remove a registry source URL
    Remove {
        /// URL to remove
        url: String,

        /// Remove from user scope
        #[arg(long)]
        user: bool,

        /// Remove from system scope (requires elevation)
        #[arg(long)]
        system: bool,
    },
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
                print_list_table(&servers);
            }
        }
        Commands::Info { id, json } => {
            match get_server(&paths, &id) {
                Some((manifest, scope)) => {
                    let scope_str = match scope {
                        dmcp::discovery::Scope::User => "user",
                        dmcp::discovery::Scope::System => "system",
                    };
                    if json {
                        let output = serde_json::to_string_pretty(&manifest).unwrap();
                        println!("{output}");
                    } else {
                        print_info_output(&manifest, scope_str);
                    }
                }
                None => {
                    eprintln!("Server not found: {}", id);
                    std::process::exit(1);
                }
            }
        }
        Commands::Config { id, action } => match action {
            ConfigAction::Set { key, value } => {
                match set_config_value(&paths, &id, &key, &value) {
                    Ok(()) => println!("Set {} = {}", key, value),
                    Err(config::SetConfigError::WriteFailed(_, manifest_path)) if !is_elevated() => {
                        if is_system_scope(&manifest_path, paths.system_install_dir()) {
                            re_exec_with_pkexec();
                        } else {
                            eprintln!("Error: Failed to write manifest (permission denied)");
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            ConfigAction::Get { key, json } => {
                match get_server(&paths, &id) {
                    Some((manifest, _)) => {
                        if let Some(k) = key {
                            match manifest.config.get(&k) {
                                Some(v) => {
                                    if json {
                                        println!("{}", serde_json::to_string_pretty(v).unwrap());
                                    } else {
                                        let val: String = v.as_str().map(String::from).unwrap_or_else(|| v.to_string());
                                        println!("{}", val);
                                    }
                                }
                                None => {
                                    eprintln!("Config key not found: {}", k);
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            if json {
                                let output = serde_json::to_string_pretty(&manifest.config).unwrap();
                                println!("{output}");
                            } else {
                                if manifest.config.is_empty() {
                                    println!("No config set.");
                                } else {
                                    for (k, v) in &manifest.config {
                                        let val: String = v.as_str().map(String::from).unwrap_or_else(|| v.to_string());
                                        println!("{} = {}", k, val);
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        eprintln!("Server not found: {}", id);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Sources { action } => match action {
            SourcesAction::List { user, system } => {
                let include_user = user || (!user && !system);
                let include_system = system || (!user && !system);
                let sources = list_sources(&paths, include_user, include_system);
                if sources.is_empty() {
                    println!("No registry sources configured.");
                    println!("Add URLs to ~/.config/mcp/sources.list or /etc/mcp/sources.list");
                    return;
                }
                println!("{:<8} {}", "SCOPE", "URL");
                println!("{}", "-".repeat(80));
                for (url, scope) in sources {
                    let scope_str = match scope {
                        dmcp::SourceScope::User => "user",
                        dmcp::SourceScope::System => "system",
                    };
                    println!("{:<8} {}", scope_str, url);
                }
            }
            SourcesAction::Add { url, system, .. } => {
                let scope = if system {
                    dmcp::SourceScope::System
                } else {
                    dmcp::SourceScope::User
                };
                // System scope needs root for create_dir + write; re-exec upfront
                if scope == dmcp::SourceScope::System && !is_elevated() {
                    re_exec_with_pkexec();
                }
                match add_source(&paths, &url, scope) {
                    Ok(()) => println!("Added {}", url),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            SourcesAction::Remove { url, system, .. } => {
                let scope = if system {
                    dmcp::SourceScope::System
                } else {
                    dmcp::SourceScope::User
                };
                // System scope needs root; re-exec upfront
                if scope == dmcp::SourceScope::System && !is_elevated() {
                    re_exec_with_pkexec();
                }
                match remove_source(&paths, &url, scope) {
                    Ok(()) => println!("Removed {}", url),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Install { id, system } => {
            let server = match fetch_server_from_registry(&paths, &id) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            let scope = if system {
                dmcp::discovery::Scope::System
            } else {
                scope_from_registry_server(&server)
            };
            if scope == dmcp::discovery::Scope::System && !is_elevated() {
                re_exec_with_pkexec();
            }
            match install(&paths, &id, scope, Some(server)) {
                Ok(()) => println!("Installed {}", id),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Uninstall { id } => {
            if let Some((_, _, scope)) = discovery::get_uninstall_info(&paths, &id) {
                if scope == dmcp::discovery::Scope::System && !is_elevated() {
                    re_exec_with_pkexec();
                }
            }
            match uninstall(&paths, &id) {
                Ok(()) => println!("Uninstalled {}", id),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Browse { url, user, system, json } => {
            let (servers, errors): (Vec<_>, Vec<_>) = if let Some(ref u) = url {
                match list_registry_servers_from_url(u) {
                    Ok(s) => (s, vec![]),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let include_user = user || (!user && !system);
                let include_system = system || (!user && !system);
                list_registry_servers(&paths, include_user, include_system)
            };

            for e in &errors {
                eprintln!("Warning: {}", e);
            }

            if json {
                let output = serde_json::to_string_pretty(&servers).unwrap();
                println!("{output}");
            } else {
                if servers.is_empty() && errors.is_empty() && url.is_none() {
                    println!("No registry sources configured. Add one with: dmcp sources add <url>");
                    return;
                }
                if servers.is_empty() {
                    println!("No servers found in registries.");
                    return;
                }
                print_browse_table(&servers);
            }
        }
    }
}

fn format_tools(tools: &[serde_json::Value]) -> String {
    tools
        .iter()
        .map(|t| {
            if let Some(obj) = t.as_object() {
                obj.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("?")
            } else {
                t.as_str().unwrap_or("?")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_transports(transports: &[dmcp::models::Transport]) -> String {
    transports
        .iter()
        .map(|t| match t {
            dmcp::models::Transport::Stdio { command, args, .. } => {
                let args_str = args
                    .as_ref()
                    .map(|a| a.join(" "))
                    .unwrap_or_default();
                format!("stdio ({command} {args_str})")
            }
            dmcp::models::Transport::Sse { url, .. } => format!("sse ({url})"),
            dmcp::models::Transport::WebSocket { ws_url, .. } => format!("websocket ({ws_url})"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn print_info_output(manifest: &dmcp::Manifest, scope_str: &str) {
    const INDENT: &str = "        ";

    println!("{}", manifest.id.as_deref().unwrap_or("?"));
    println!("{}Name:        {}", INDENT, manifest.name.as_deref().unwrap_or("?"));
    println!("{}Version:     {}", INDENT, manifest.version.as_deref().unwrap_or("?"));
    println!("{}Scope:       {}", INDENT, scope_str);
    if let Some(s) = manifest.summary.as_deref().filter(|x| !x.is_empty()) {
        println!("{}Summary:     {}", INDENT, s);
    }
    if let Some(d) = manifest.description.as_deref().filter(|x| !x.is_empty()) {
        println!("{}Description:", INDENT);
        for line in d.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                println!("{}{}{}", INDENT, INDENT, trimmed);
            }
        }
    }
    if let Some(a) = manifest.author.as_deref().filter(|x| !x.is_empty()) {
        println!("{}Author:      {}", INDENT, a);
    }
    if let Some(h) = manifest.homepage.as_deref().filter(|x| !x.is_empty()) {
        println!("{}Homepage:    {}", INDENT, h);
    }
    if !manifest.categories.is_empty() {
        println!("{}Categories:  {}", INDENT, manifest.categories.join(", "));
    }
    if !manifest.capabilities.is_empty() {
        println!("{}Capabilities: {}", INDENT, manifest.capabilities.join(", "));
    }
    if !manifest.tools.is_empty() {
        println!("{}Tools:       {}", INDENT, format_tools(&manifest.tools));
    }
    if let Some(ref t) = manifest.transports {
        println!("{}Transports:  {}", INDENT, format_transports(t));
    }
    if let Some(ref dir) = manifest.install_dir {
        println!("{}Install:     {}", INDENT, dir);
    }
    if !manifest.config.is_empty() {
        for (k, v) in &manifest.config {
            let val: String = v.as_str().map(String::from).unwrap_or_else(|| v.to_string());
            println!("{}Config.{}:   {}", INDENT, k, val);
        }
    }
}

fn print_list_table(servers: &[dmcp::ServerInfo]) {
    const INDENT: &str = "        ";

    for s in servers {
        let scope = match s.scope {
            dmcp::discovery::Scope::User => "user",
            dmcp::discovery::Scope::System => "system",
        };
        println!("{}", s.id);
        println!("{}Name:      {}", INDENT, s.name);
        println!("{}Version:   {}", INDENT, s.version);
        println!("{}Transport: {}", INDENT, s.transport_type);
        println!("{}Scope:     {}", INDENT, scope);
        println!("{}Install:   {}", INDENT, s.install_dir);
        println!();
    }
}

fn print_browse_table(servers: &[dmcp::RegistryServer]) {
    const INDENT: &str = "        ";

    for s in servers {
        println!("{}", s.id);
        println!("{}Name:      {}", INDENT, s.name);
        println!("{}Version:   {}", INDENT, s.version);
        println!("{}Transport: {}", INDENT, s.transport);
        if !s.summary.is_empty() {
            println!("{}Summary:   {}", INDENT, s.summary.lines().next().unwrap_or("").trim());
        }
        println!("{}Source:    {}", INDENT, s.source);
        println!();
    }
}
