use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_game_path(base: &Path, relative_path: &str) -> PathBuf {
    let mut resolved = base.to_path_buf();
    for component in relative_path.split(['\\', '/']) {
        match component {
            "" | "." => {}
            ".." => {
                resolved.pop();
            }
            name => resolved = find_entry_ignoring_case(&resolved, name),
        }
    }
    resolved
}

pub fn find_entry_ignoring_case(directory: &Path, name: &str) -> PathBuf {
    let exact_path = directory.join(name);
    if exact_path.exists() {
        return exact_path;
    }
    fs::read_dir(directory)
        .ok()
        .and_then(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name())
                .find(|entry_name| {
                    entry_name
                        .to_str()
                        .is_some_and(|entry_name| entry_name.eq_ignore_ascii_case(name))
                })
        })
        .map(|entry_name| directory.join(entry_name))
        .unwrap_or(exact_path)
}
