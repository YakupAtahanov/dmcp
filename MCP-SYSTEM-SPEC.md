# MCP System Integration Spec

This document specifies the layout, formats, and expected behavior of a system-level MCP (Model Context Protocol) server manager. It is intended as a reference for implementing a standalone package (e.g. `dmcp`, `libmcp`, or `mcp-manager`) that discovers, manages, and invokes MCP servers installed on the system.

**Status:** Specification only. Implementation is separate.

---

## 1. Overview

MCP servers can be installed in two scopes:

- **User scope:** Per-user, no root required
- **System scope:** System-wide, visible to all users, requires root for install/remove

A system package should provide:

1. **Discovery** — List all installed MCP servers (user + system)
2. **Invocation** — Spawn stdio servers with correct command, args, working dir, and config
3. **Config resolution** — Read and merge config from manifests
4. **Path abstraction** — Single API for user vs system scope and precedence

---

## 2. Paths (XDG Conventions)

All paths follow XDG Base Directory Specification where applicable.

### 2.1 Registry Sources (Config)

| Scope | Path | Notes |
|-------|------|-------|
| User | `$XDG_CONFIG_HOME/mcp/sources.list` | Default: `~/.config/mcp/sources.list` |
| System | `/etc/mcp/sources.list` | Admin-managed |

**Priority:** User sources are read first, then system. Both are merged; duplicates are deduplicated (user wins).

### 2.2 Installed Servers (Data)

| Scope | Base path | Index | Manifest |
|-------|-----------|-------|----------|
| User | `$XDG_DATA_HOME/mcp/installed/` | `index.json` | `<id>/manifest.json` |
| System | `/usr/share/mcp/installed/` | `index.json` | `<id>/manifest.json` |

Default for `$XDG_DATA_HOME`: `~/.local/share`.

**Load order:** User index first, then system. If the same `id` appears in both, implementation may choose user over system (user override).

### 2.3 Registry Cache (Optional)

| Path | Purpose |
|------|---------|
| `$XDG_CACHE_HOME/discover/mcp-registries/` | Cached registry JSON files (Discover-specific) |

A generic MCP manager may use `$XDG_CACHE_HOME/mcp/registries/` or similar.

---

## 3. sources.list Format

Plain text file. One URL per line. Lines starting with `#` and empty lines are ignored.

```
# MCP Registry Sources
# Each line is a URL to a registry JSON file

https://raw.githubusercontent.com/example/mcp-registry/main/registry.json
https://example.com/other-registry.json
```

---

## 4. Registry Format

A registry is a JSON file fetched from a URL. Structure:

```json
{
  "version": "1.0",
  "updated": "2025-02-03T00:00:00Z",
  "servers": [
    { "id": "...", "name": "...", ... }
  ]
}
```

| Field | Type | Description |
|-------|------|--------------|
| `version` | string | Format version (use `"1.0"`) |
| `updated` | string | ISO 8601 timestamp |
| `servers` | array | Array of server entry objects |

### 4.1 Server Entry (Registry)

Each server object in `servers`:

**Required:** `id`, `name`, `summary`, `version`, `transports`, `source` (for stdio)

**Optional:** `description`, `author`, `homepage`, `bugUrl`, `donationUrl`, `icon`, `categories`, `capabilities`, `permissions`, `tools`, `configurableProperties`, `license`, `releaseDate`, `size`, `screenshots`, `changelog`, `scope`

**Icon:** Freedesktop icon name (e.g. `"utilities-terminal"`) or URL to image (e.g. `https://example.com/logo.png`).

---

## 5. Index Format

`index.json` lives at `<base>/mcp/installed/index.json`. It stores pointers only; full metadata is in each manifest.

```json
{
  "servers": {
    "com.example.my-server": {
      "location": "/home/user/.local/share/mcp/installed/com.example.my-server/manifest.json"
    }
  }
}
```

| Field | Type | Description |
|-------|------|--------------|
| `servers` | object | Map of server `id` → `{ "location": "<absolute path to manifest.json>" }` |

---

## 6. Manifest Format

Each installed server has `manifest.json` in its install directory. The manifest is the full server metadata plus runtime config. MCP servers read their configuration from this file.

Structure matches the registry server entry, plus:

- `config` — Object of `key` → `value` for configurable properties (user-provided values)
- `installDir` — Absolute path to the install directory (for servers to resolve paths)

### 6.1 Transport Types

**stdio (local process):**

```json
{
  "type": "stdio",
  "command": "python3",
  "args": ["server.py"],
  "description": "Main interface"
}
```

- `command`: Executable (e.g. `python3`, `node`)
- `args`: Arguments, relative to project root (install dir)
- Process is spawned with `cwd` = install dir

**sse (remote):**

```json
{
  "type": "sse",
  "url": "https://api.example.com/mcp/sse",
  "description": "Cloud endpoint"
}
```

**websocket (remote):**

```json
{
  "type": "websocket",
  "wsUrl": "wss://api.example.com/mcp/ws"
}
```

### 6.2 Source (for stdio)

```json
{
  "source": {
    "type": "git",
    "url": "https://github.com/example/repo.git",
    "path": "servers/my-server"
  }
}
```

- `path`: Project root within repo (optional; empty = repo root)
- Install dir contains the cloned/extracted project; `command` + `args` run from there

### 6.3 Configurable Properties

```json
{
  "configurableProperties": [
    {
      "key": "api_key",
      "label": "API Key",
      "description": "Your API key",
      "sensitive": true,
      "required": true
    },
    {
      "key": "timeout",
      "label": "Timeout (seconds)",
      "default": "30",
      "sensitive": false,
      "required": false
    }
  ],
  "config": {
    "api_key": "user-provided-value",
    "timeout": "60"
  }
}
```

- `config` holds user-provided values; optional properties get `default` if not set
- MCP servers read `manifest.json` to get their config

### 6.4 Scope

- `"scope": "user"` (default) → `$XDG_DATA_HOME/mcp/installed/<id>/`
- `"scope": "system"` → `/usr/share/mcp/installed/<id>/`

---

## 7. Directory Layout After Install

**User scope:**

```
~/.local/share/mcp/installed/
├── index.json
├── com.example.calculator/          (stdio — Git clone)
│   ├── manifest.json
│   ├── server.py
│   └── ...                          (project files)
└── com.example.remote-api/          (SSE — metadata only)
    └── manifest.json
```

**System scope:** Same structure under `/usr/share/mcp/installed/`.

---

## 8. API Requirements (for Implementation)

A reference implementation should provide at least:

### 8.1 Discovery

- `list_servers()` — Return all installed servers (user + system, with precedence)
- `get_server(id)` — Return metadata for a server by id, or null if not found
- `get_manifest_path(id)` — Return path to manifest.json for a server

### 8.2 Invocation (stdio only)

- `spawn_server(id)` — Start the stdio process for a server
  - Working directory: install dir
  - Environment: inherit + any env vars from manifest (if specified)
  - Command + args from primary transport
  - Config available to server via manifest.json in cwd

### 8.3 Config

- `get_config(id)` — Return merged config (defaults + user values)
- `set_config_value(id, key, value)` — Update config and persist to manifest (requires write access)

### 8.4 Path Helpers

- `user_install_dir()` — `$XDG_DATA_HOME/mcp/installed/`
- `system_install_dir()` — `/usr/share/mcp/installed/`
- `user_sources_path()` — `$XDG_CONFIG_HOME/mcp/sources.list`
- `system_sources_path()` — `/etc/mcp/sources.list`

---

## 9. Invocation Behavior (stdio)

When spawning a stdio server:

1. Resolve manifest path from index
2. Load manifest.json
3. Get primary transport (first in `transports` array) with `type == "stdio"`
4. `cwd` = `installDir` from manifest (or dir containing manifest.json)
5. Execute `command` with `args`
6. Server reads `manifest.json` from cwd for config

Environment: Inherit from parent. Future: support `env` in manifest for overrides.

---

## 10. Backward Compatibility (Planned)

For robustness, an implementation may support:

1. **Missing index:** Scan `<base>/mcp/installed/` for subdirs with `manifest.json`; build index in memory
2. **Legacy index:** If `servers` is an array of full objects instead of id→location map, parse and optionally migrate
3. **Unknown servers:** Log and optionally surface; do not silently drop

See `backward_compatibility.md` in this backend for details.

---

## 11. References

- **KDE Discover MCP Backend:** This repo (`libdiscover/backends/MCPBackend/`)
- **MCP Registry Guide:** `MCP-REGISTRY-GUIDE.md` — registry format, transports, scope
- **Backward Compatibility:** `backward_compatibility.md` — migration strategy
- **XDG Base Directory:** https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
- **Model Context Protocol:** https://modelcontextprotocol.io/
