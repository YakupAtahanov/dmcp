//! dmcp - MCP Manager
//!
//! Discovers, manages, and invokes MCP servers at user and system scope.

pub mod discovery;
pub mod models;
pub mod paths;

pub use discovery::{list_servers, ServerInfo};
pub use models::{Index, Manifest};
pub use paths::Paths;
