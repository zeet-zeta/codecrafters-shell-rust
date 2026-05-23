use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

pub struct CompletionSpec {
    pub completer_path: String,
}

pub static COMPLETION_REGISTRY: LazyLock<RwLock<HashMap<String, CompletionSpec>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn register_completion(cmd: String, path: String) {
    if let Ok(mut guard) = COMPLETION_REGISTRY.write() {
        guard.insert(
            cmd,
            CompletionSpec {
                completer_path: path,
            },
        );
    }
}

pub fn find_completion(cmd: &str) -> Option<String> {
    if let Ok(guard) = COMPLETION_REGISTRY.read() {
        guard.get(cmd).map(|spec| spec.completer_path.clone())
    } else {
        None
    }
}
