//! dmcp - MCP Manager
//!
//! Discovers, manages, and invokes MCP servers at user and system scope.

pub mod config;
pub mod discovery;
pub mod models;
pub mod paths;
pub mod sources;

pub use config::set_config_value;
pub use discovery::{get_manifest_path, get_server, list_servers, ServerInfo};
pub use models::{Index, Manifest};
pub use paths::Paths;
pub use sources::{list_sources, SourceScope};
