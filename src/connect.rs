//! Connect to remote (SSE/WebSocket) MCP servers by URL, without a registry.

use crate::discovery;
use crate::elevation::is_elevated;
use crate::paths::Paths;

/// Connect to a remote MCP server by URL. Creates manifest and adds to installed.
/// All metadata is optional; id is auto-generated as com.user.connected.server1, server2, ...
pub fn connect(
    paths: &Paths,
    url: &str,
    name: Option<&str>,
    summary: Option<&str>,
    version: Option<&str>,
    config: &[(String, String)],
    scope: crate::discovery::Scope,
) -> Result<String, ConnectError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(ConnectError::InvalidUrl);
    }

    let transport_type = if url.starts_with("wss://") || url.starts_with("ws://") {
        "websocket"
    } else {
        "sse"
    };

    let id = next_connected_server_id(paths, scope)?;
    let install_dir = match scope {
        crate::discovery::Scope::User => paths.user_install_dir().join(&id),
        crate::discovery::Scope::System => paths.system_install_dir().join(&id),
    };

    std::fs::create_dir_all(&install_dir).map_err(ConnectError::CreateDir)?;

    let transport = if transport_type == "websocket" {
        serde_json::json!({
            "type": "websocket",
            "wsUrl": url
        })
    } else {
        serde_json::json!({
            "type": "sse",
            "url": url
        })
    };

    let mut config_obj = serde_json::Map::new();
    for (k, v) in config {
        config_obj.insert(k.clone(), serde_json::Value::String(v.clone()));
    }

    let manifest = serde_json::json!({
        "id": id,
        "name": name.unwrap_or(&id),
        "summary": summary.unwrap_or("Connected via dmcp connect"),
        "version": version.unwrap_or("1.0.0"),
        "transports": [transport],
        "installDir": install_dir.to_string_lossy(),
        "config": config_obj
    });

    let manifest_path = install_dir.join("manifest.json");
    let output = serde_json::to_string_pretty(&manifest).map_err(ConnectError::Serialize)?;
    std::fs::write(&manifest_path, output).map_err(ConnectError::WriteManifest)?;

    crate::install::update_index_add(paths, &id, &manifest_path, scope)
        .map_err(|e| ConnectError::IndexError(e.to_string()))?;

    Ok(id)
}

fn next_connected_server_id(paths: &Paths, scope: crate::discovery::Scope) -> Result<String, ConnectError> {
    let index_path = match scope {
        crate::discovery::Scope::User => paths.user_install_dir().join("index.json"),
        crate::discovery::Scope::System => paths.system_install_dir().join("index.json"),
    };

    let content = std::fs::read_to_string(&index_path).unwrap_or_else(|_| r#"{"servers":{},"version":"1.0"}"#.to_string());
    let index: serde_json::Value = serde_json::from_str(&content).map_err(ConnectError::ParseIndex)?;

    let empty = serde_json::Map::new();
    let servers = index.get("servers").and_then(|s| s.as_object()).unwrap_or(&empty);

    let mut max_n = 0u32;
    for (id, _) in servers {
        if let Some(n) = id.strip_prefix("com.user.connected.server").and_then(|s| s.parse::<u32>().ok()) {
            if n > max_n {
                max_n = n;
            }
        }
    }

    Ok(format!("com.user.connected.server{}", max_n + 1))
}

#[derive(Debug)]
pub enum ConnectError {
    InvalidUrl,
    CreateDir(std::io::Error),
    Serialize(serde_json::Error),
    WriteManifest(std::io::Error),
    ParseIndex(serde_json::Error),
    IndexError(String),
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectError::InvalidUrl => write!(f, "Invalid or empty URL"),
            ConnectError::CreateDir(e) => write!(f, "Failed to create directory: {}", e),
            ConnectError::Serialize(e) => write!(f, "Failed to serialize: {}", e),
            ConnectError::WriteManifest(e) => write!(f, "Failed to write manifest: {}", e),
            ConnectError::ParseIndex(e) => write!(f, "Failed to parse index: {}", e),
            ConnectError::IndexError(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for ConnectError {}
