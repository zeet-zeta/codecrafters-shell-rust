use std::os::unix::fs::MetadataExt;

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

pub fn get_all_external_commands() -> Option<Vec<String>> {
    let mut result = Vec::new();
    let path_os = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_os) {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries {
            let path = entry.ok()?.path();
            if is_executable(&path) {
                result.push(path.file_name()?.to_str()?.to_string());
            }
        }
    }
    Some(result)
}
