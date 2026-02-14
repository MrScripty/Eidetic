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
/// Handles migration from v1 format (characters field) to v2 (bible field).
pub async fn load_project(path: &Path) -> Result<Project, String> {
    let data = fs::read_to_string(path)
        .await
        .map_err(|e| format!("read error: {e}"))?;

    let mut project: Project =
        serde_json::from_str(&data).map_err(|e| format!("deserialize error: {e}"))?;

    // Migrate v1 characters into bible entities if bible is empty and JSON had characters.
    if project.bible.entities.is_empty() {
        if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(chars) = raw.get("characters").and_then(|v| v.as_array()) {
                migrate_v1_characters(&mut project, chars);
                if !project.bible.entities.is_empty() {
                    tracing::info!(
                        "migrated {} v1 characters to bible entities",
                        project.bible.entities.len()
                    );
                }
            }
        }
    }

    tracing::debug!("loaded project from {}", path.display());
    Ok(project)
}

/// Convert v1 Character objects into Entity objects in the story bible.
fn migrate_v1_characters(project: &mut Project, chars: &[serde_json::Value]) {
    use eidetic_core::story::arc::Color;
    use eidetic_core::story::bible::{Entity, EntityCategory, EntityDetails};

    for ch in chars {
        let name = ch
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let description = ch
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let voice_notes = ch
            .get("voice_notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let color = ch
            .get("color")
            .and_then(|v| {
                let r = v.get("r").and_then(|x| x.as_u64())? as u8;
                let g = v.get("g").and_then(|x| x.as_u64())? as u8;
                let b = v.get("b").and_then(|x| x.as_u64())? as u8;
                Some(Color::new(r, g, b))
            })
            .unwrap_or(Color::new(200, 200, 200));

        let mut entity = Entity::new(name, EntityCategory::Character, color);
        entity.description = description.clone();
        entity.tagline = if description.len() > 100 {
            format!("{}...", &description[..97])
        } else {
            description
        };
        entity.details = EntityDetails::Character {
            traits: Vec::new(),
            voice_notes,
            character_relations: Vec::new(),
            audience_knowledge: String::new(),
        };

        project.bible.add_entity(entity);
    }
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
