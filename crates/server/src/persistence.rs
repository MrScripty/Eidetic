use std::path::{Path, PathBuf};

use eidetic_core::Project;
use serde::Serialize;
use tokio::fs;

/// Metadata for a saved project on disk.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
    pub modified: String,
}

/// Default base directory for project storage.
pub fn default_project_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eidetic")
        .join("projects")
}

/// Save a project to disk as JSON. Uses atomic write (tmp + rename).
pub async fn save_project(project: &Project, path: &Path) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(project).map_err(|e| format!("serialize error: {e}"))?;

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("mkdir error: {e}"))?;
    }

    // Atomic write: write to .tmp, then rename.
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, json.as_bytes())
        .await
        .map_err(|e| format!("write error: {e}"))?;
    fs::rename(&tmp_path, path)
        .await
        .map_err(|e| format!("rename error: {e}"))?;

    tracing::debug!("saved project to {}", path.display());
    Ok(())
}

/// Load a project from a JSON file on disk.
pub async fn load_project(path: &Path) -> Result<Project, String> {
    let data = fs::read_to_string(path)
        .await
        .map_err(|e| format!("read error: {e}"))?;
    let project: Project =
        serde_json::from_str(&data).map_err(|e| format!("deserialize error: {e}"))?;
    tracing::debug!("loaded project from {}", path.display());
    Ok(project)
}

/// List saved projects under a base directory.
pub async fn list_projects(base_dir: &Path) -> Vec<ProjectEntry> {
    let mut entries = Vec::new();

    let Ok(mut read_dir) = fs::read_dir(base_dir).await else {
        return entries;
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        let project_file = path.join("project.json");
        if !project_file.exists() {
            continue;
        }

        let modified = entry
            .metadata()
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs().to_string())
            })
            .unwrap_or_default();

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        entries.push(ProjectEntry {
            name,
            path: project_file,
            modified,
        });
    }

    entries
}

/// Generate a save path for a project based on its name.
pub fn project_save_path(name: &str) -> PathBuf {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    default_project_dir().join(sanitized).join("project.json")
}
