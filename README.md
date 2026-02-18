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
./target/release/dmcp list --user --system
```

Or with `cargo run`:

```bash
cargo run -- list --user --system
cargo run -- list --json   # JSON output
```

## Project Structure

```
src/
├── main.rs      # CLI entry point
├── lib.rs       # Library root
├── paths.rs     # Path resolution (env, XDG)
├── discovery.rs # List servers, load index/manifests
└── models.rs    # Index, Manifest, Transport structs
```

## Status

Initial implementation. `dmcp list` works. Install, config, spawn coming next.

## References

- [Model Context Protocol](https://modelcontextprotocol.io/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
