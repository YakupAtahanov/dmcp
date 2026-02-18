//! Discovery of installed MCP servers.

use std::collections::HashMap;
use std::path::Path;

use crate::models::{Index, Manifest};
use crate::paths::Paths;

/// Info about an installed server for display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub transport_type: String,
    pub scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    System,
}

/// List installed servers from the given scopes.
/// User takes precedence over system for duplicate IDs.
pub fn list_servers(paths: &Paths, user: bool, system: bool, debug: bool) -> Vec<ServerInfo> {
    let mut seen = HashMap::new();

    if user {
        if let Some(servers) = load_from_scope(paths.user_install_dir(), Scope::User, debug) {
            for s in servers {
                seen.insert(s.id.clone(), s);
            }
        } else if debug {
            eprintln!("[debug] load_from_scope(user) returned None");
        }
    }

    if system {
        if let Some(servers) = load_from_scope(paths.system_install_dir(), Scope::System, debug) {
            for s in servers {
                seen.entry(s.id.clone()).or_insert(s);
            }
        } else if debug {
            eprintln!("[debug] load_from_scope(system) returned None");
        }
    }

    let mut result: Vec<_> = seen.into_values().collect();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

fn load_from_scope(base: &Path, scope: Scope, debug: bool) -> Option<Vec<ServerInfo>> {
    let index_path = base.join("index.json");
    if debug {
        eprintln!("[debug] Reading index: {}", index_path.display());
    }
    let index: Index = match std::fs::read_to_string(&index_path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(idx) => idx,
            Err(e) => {
                if debug {
                    eprintln!("[debug] Index parse error: {}", e);
                }
                return None;
            }
        },
        Err(e) => {
            if debug {
                eprintln!("[debug] Index read error: {}", e);
            }
            return Some(vec![]);
        }
    };

    if debug {
        eprintln!("[debug] Index has {} servers", index.servers.len());
    }

    let mut servers = Vec::new();
    for (id, entry) in index.servers {
        let manifest_path = Path::new(&entry.location);
        if debug {
            eprintln!("[debug] Loading manifest: {}", manifest_path.display());
        }
        let manifest: Manifest = match std::fs::read_to_string(manifest_path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(m) => m,
                Err(e) => {
                    if debug {
                        eprintln!("[debug] Manifest parse error for {}: {}", id, e);
                    }
                    continue;
                }
            },
            Err(e) => {
                if debug {
                    eprintln!("[debug] Manifest read error for {}: {}", id, e);
                }
                continue;
            }
        };

        let transport_type = manifest
            .transports
            .as_ref()
            .and_then(|t| t.first())
            .map(transport_type_name)
            .unwrap_or_else(|| "unknown".to_string());

        servers.push(ServerInfo {
            id: manifest.id.unwrap_or(id),
            name: manifest.name.unwrap_or_else(|| "Unknown".to_string()),
            version: manifest.version.unwrap_or_else(|| "?".to_string()),
            transport_type,
            scope,
        });
    }

    if debug {
        eprintln!("[debug] Loaded {} servers from {:?} scope", servers.len(), scope);
    }

    Some(servers)
}

fn transport_type_name(t: &crate::models::Transport) -> String {
    match t {
        crate::models::Transport::Stdio { .. } => "stdio",
        crate::models::Transport::Sse { .. } => "sse",
        crate::models::Transport::WebSocket { .. } => "websocket",
    }
    .to_string()
}
