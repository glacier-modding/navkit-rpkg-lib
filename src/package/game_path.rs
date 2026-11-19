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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_game_path_simple() {
        let base = Path::new("/base");
        let result = resolve_game_path(base, "dir/file.txt");
        assert_eq!(result, PathBuf::from("/base/dir/file.txt"));
    }

    #[test]
    fn test_resolve_game_path_with_parent() {
        let base = Path::new("/base/subdir");
        let result = resolve_game_path(base, "../other/file.txt");
        assert_eq!(result, PathBuf::from("/base/other/file.txt"));
    }

    #[test]
    fn test_resolve_game_path_with_current() {
        let base = Path::new("/base");
        let result = resolve_game_path(base, "./dir/file.txt");
        assert_eq!(result, PathBuf::from("/base/dir/file.txt"));
    }

    #[test]
    fn test_resolve_game_path_backslash() {
        let base = Path::new("/base");
        let result = resolve_game_path(base, "dir\\file.txt");
        assert_eq!(result, PathBuf::from("/base/dir/file.txt"));
    }

    #[test]
    fn test_resolve_game_path_empty_component() {
        let base = Path::new("/base");
        let result = resolve_game_path(base, "dir//file.txt");
        assert_eq!(result, PathBuf::from("/base/dir/file.txt"));
    }

    #[test]
    fn test_resolve_game_path_multiple_parent() {
        let base = Path::new("/base/a/b/c");
        let result = resolve_game_path(base, "../../file.txt");
        assert_eq!(result, PathBuf::from("/base/a/file.txt"));
    }

    #[test]
    fn test_find_entry_ignoring_case_exact_match() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_exact_match");
        fs::create_dir_all(&test_dir).ok();

        let test_file = test_dir.join("TestFile.txt");
        fs::write(&test_file, "content").unwrap();

        let result = find_entry_ignoring_case(&test_dir, "TestFile.txt");
        assert_eq!(result, test_file);

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_find_entry_ignoring_case_different_case() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_case_insensitive");
        fs::create_dir_all(&test_dir).ok();

        let test_file = test_dir.join("TestFile.txt");
        fs::write(&test_file, "content").unwrap();

        // Search with different case
        let result = find_entry_ignoring_case(&test_dir, "testfile.txt");
        assert_eq!(result, test_file);

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_find_entry_ignoring_case_uppercase() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_uppercase");
        fs::create_dir_all(&test_dir).ok();

        let test_file = test_dir.join("lowercase.txt");
        fs::write(&test_file, "content").unwrap();

        // Search with uppercase
        let result = find_entry_ignoring_case(&test_dir, "LOWERCASE.TXT");
        assert_eq!(result, test_file);

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_find_entry_ignoring_case_not_found() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_not_found");
        fs::create_dir_all(&test_dir).ok();

        // File doesn't exist
        let result = find_entry_ignoring_case(&test_dir, "nonexistent.txt");
        assert_eq!(result, test_dir.join("nonexistent.txt"));

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_find_entry_ignoring_case_directory() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_directory");
        fs::create_dir_all(&test_dir).ok();

        let sub_dir = test_dir.join("SubDir");
        fs::create_dir_all(&sub_dir).ok();

        // Search for directory with different case
        let result = find_entry_ignoring_case(&test_dir, "subdir");
        assert_eq!(result, sub_dir);

        // Cleanup
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_find_entry_ignoring_case_nonexistent_parent() {
        let nonexistent = Path::new("/nonexistent_directory_12345");
        let result = find_entry_ignoring_case(nonexistent, "file.txt");
        assert_eq!(result, nonexistent.join("file.txt"));
    }
}
