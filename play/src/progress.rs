use std::{collections::HashSet, path::Path};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Progress {
    pub completed: HashSet<String>,
}

const FILE: &str = ".play-progress.json";

pub fn load(workspace: &Path) -> Progress {
    let path = workspace.join(FILE);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(workspace: &Path, progress: &Progress) {
    let path = workspace.join(FILE);
    if let Ok(json) = serde_json::to_string_pretty(progress) {
        let _ = std::fs::write(path, json);
    }
}
