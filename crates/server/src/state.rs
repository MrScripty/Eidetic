use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use eidetic_core::Project;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::persistence;

/// Events broadcast to all connected WebSocket clients after mutations.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    TimelineChanged,
    ScenesChanged,
    BeatUpdated { clip_id: uuid::Uuid },
    StoryChanged,
    GenerationProgress {
        clip_id: uuid::Uuid,
        token: String,
        tokens_generated: usize,
    },
    GenerationComplete {
        clip_id: uuid::Uuid,
    },
    GenerationError {
        clip_id: uuid::Uuid,
        error: String,
    },
    ConsistencySuggestion {
        source_clip_id: uuid::Uuid,
        target_clip_id: uuid::Uuid,
        original_text: String,
        suggested_text: String,
        reason: String,
    },
    ConsistencyComplete {
        source_clip_id: uuid::Uuid,
        suggestion_count: usize,
    },
}

/// Which AI backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendType {
    Ollama,
    OpenRouter,
}

/// Configuration for the active AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub backend_type: BackendType,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::Ollama,
            model: "qwen3:30b-a3b".into(),
            temperature: 0.7,
            max_tokens: 4096,
            base_url: "http://localhost:11434".into(),
            api_key: None,
        }
    }
}

/// Shared application state, wrapped in an Arc for use as axum state.
#[derive(Clone)]
pub struct AppState {
    pub project: Arc<Mutex<Option<Project>>>,
    pub events_tx: broadcast::Sender<ServerEvent>,
    pub ai_config: Arc<Mutex<AiConfig>>,
    /// Clip IDs currently being generated — prevents duplicate requests.
    pub generating: Arc<Mutex<HashSet<uuid::Uuid>>>,
    /// Path where the current project is saved on disk.
    pub project_path: Arc<Mutex<Option<PathBuf>>>,
    /// Channel to signal the auto-save background task.
    save_tx: tokio::sync::mpsc::Sender<()>,
}

impl AppState {
    pub fn new() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        let (save_tx, save_rx) = tokio::sync::mpsc::channel(16);

        let project = Arc::new(Mutex::new(None));
        let project_path = Arc::new(Mutex::new(None::<PathBuf>));

        // Spawn the debounced auto-save background task.
        let save_project = project.clone();
        let save_path = project_path.clone();
        tokio::spawn(auto_save_task(save_rx, save_project, save_path));

        Self {
            project,
            events_tx,
            ai_config: Arc::new(Mutex::new(AiConfig::default())),
            generating: Arc::new(Mutex::new(HashSet::new())),
            project_path,
            save_tx,
        }
    }

    /// Signal that the project has been mutated and should be auto-saved.
    pub fn trigger_save(&self) {
        let _ = self.save_tx.try_send(());
    }
}

/// Background task that debounces save signals and writes to disk.
async fn auto_save_task(
    mut rx: tokio::sync::mpsc::Receiver<()>,
    project: Arc<Mutex<Option<Project>>>,
    project_path: Arc<Mutex<Option<PathBuf>>>,
) {
    loop {
        // Wait for the first save signal.
        if rx.recv().await.is_none() {
            break;
        }

        // Debounce: wait 2 seconds, draining any additional signals.
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        while rx.try_recv().is_ok() {}

        // Perform the save.
        let (proj_json, path) = {
            let guard = project.lock();
            let path_guard = project_path.lock();
            match (guard.as_ref(), path_guard.clone()) {
                (Some(p), Some(path)) => (p.clone(), path),
                _ => continue,
            }
        };

        if let Err(e) = persistence::save_project(&proj_json, &path).await {
            tracing::error!("auto-save failed: {e}");
        }
    }
}
