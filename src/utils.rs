use std::fs;
use std::io::{self};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn find_executable(cmd: &str) -> Option<std::path::PathBuf> {
    let path_os = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_os) {
        let full_path = dir.join(cmd);
        if is_executable(&full_path) {
            return Some(full_path);
        }
    }
    None
}

pub fn is_executable(path: &std::path::Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        metadata.is_file() && (metadata.mode() & 0o111) != 0
    } else {
        false
    }
}

pub fn get_all_commands() -> Vec<String> {
    let mut result = Vec::new();
    let path_os = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return result,
    };

    for dir in std::env::split_paths(&path_os) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if is_executable(&path) {
                if let Some(name_str) = path.file_name().and_then(|n| n.to_str()) {
                    result.push(name_str.to_string());
                }
            }
        }
    }
    result.push("exit".to_string());
    result.push("echo".to_string());
    result
}

pub fn get_files_and_directories(path: &Path) -> io::Result<Vec<String>> {
    if !path.exists() || !path.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(path)?;

    let file_list = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            if entry.path().is_dir() {
                Some(name + "/")
            } else {
                Some(name)
            }
        })
        .collect();
    Ok(file_list)
}
