use std::io::{self};
use std::os::unix::fs::MetadataExt;
use std::{env, fs};

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

pub fn get_cwd_files() -> io::Result<Vec<String>> {
    let cwd = env::current_dir()?;
    let entries = fs::read_dir(cwd)?;

    let file_list = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    Ok(file_list)
}
