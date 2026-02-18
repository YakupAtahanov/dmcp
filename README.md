# dmcp

**MCP Manager** — a modular, system- and user-level manager for MCP (Model Context Protocol) servers.

## What it does

dmcp discovers, manages, and invokes MCP servers installed on your system. It works at two scopes:

- **User scope** — per-user, no root required (`~/.local/share/mcp/`, `~/.config/mcp/`)
- **System scope** — system-wide, visible to all users (`/usr/share/mcp/`, `/etc/mcp/`)

It supports both **local** (stdio) and **remote** (SSE, WebSocket) servers. Local servers are cloned and run from disk; remote servers are metadata-only, with connection endpoints stored in manifests.

## Features (planned)

- **Discovery** — List installed servers (user + system)
- **Registry** — Fetch from configurable registry URLs, search/browse by keywords, categories, tools
- **Install / Connect** — Install local servers (Git clone) or connect remote servers (metadata)
- **Config** — Get and set per-server configuration (API keys, endpoints, etc.)
- **Invocation** — Spawn stdio servers or provide connection info for SSE/WebSocket

## Configuration

Paths are configurable via environment variables. Copy `.env.example` to `.env` and adjust as needed:

```bash
cp .env.example .env
```

See [MCP-SYSTEM-SPEC.md](MCP-SYSTEM-SPEC.md) for the full specification and [MCP-REGISTRY-GUIDE.md](MCP-REGISTRY-GUIDE.md) for registry format and install flow.

## Build & Run

Requires [Rust](https://rustup.rs/).

```bash
cargo build --release
cargo install --path .   # Install to ~/.cargo/bin
```

## Commands

| Command | Description |
|---------|-------------|
| `dmcp list [--user] [--system] [--json]` | List installed MCP servers (default: both) |
| `dmcp info <id> [--json]` | Show detailed info for a server |
| `dmcp config <id> get [key] [--json]` | Get config value(s) |
| `dmcp config <id> set <key> <value>` | Set a config value (uses pkexec for system scope) |
| `dmcp sources list [--user] [--system]` | List registry source URLs |
| `dmcp sources add <url> [--system]` | Add a registry source (default: user) |
| `dmcp sources remove <url> [--system]` | Remove a registry source |
| `dmcp paths` | Show resolved paths (debug) |

## Project Structure

```
src/
├── main.rs      # CLI entry point
├── lib.rs       # Library root
├── paths.rs     # Path resolution (env, XDG)
├── discovery.rs # List servers, get_server, load index/manifests
├── sources.rs   # Registry sources (sources.list)
└── models.rs    # Index, Manifest, Transport structs
```

## Status

Initial implementation. `dmcp list` works. Install, config, spawn coming next.

## References

- [Model Context Protocol](https://modelcontextprotocol.io/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
