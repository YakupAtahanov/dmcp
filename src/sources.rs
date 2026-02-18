//! Registry sources (sources.list).

use std::collections::HashSet;
use std::path::Path;

use crate::paths::Paths;

/// Load registry URLs from sources.list files.
/// User sources first, then system. Duplicates are deduplicated (user wins).
pub fn list_sources(paths: &Paths, include_user: bool, include_system: bool) -> Vec<(String, SourceScope)> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    if include_user {
        for url in read_sources_file(paths.user_sources_path()) {
            if seen.insert(url.clone()) {
                result.push((url, SourceScope::User));
            }
        }
    }

    if include_system {
        for url in read_sources_file(paths.system_sources_path()) {
            if seen.insert(url.clone()) {
                result.push((url, SourceScope::System));
            }
        }
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceScope {
    User,
    System,
}

fn read_sources_file(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}
