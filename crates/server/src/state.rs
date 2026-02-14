use std::collections::HashSet;
use std::sync::Arc;

use eidetic_core::Project;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

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
///
/// Currently holds a single in-memory project. Persistence (file save/load)
/// is deferred to Sprint 4.
#[derive(Clone)]
pub struct AppState {
    pub project: Arc<Mutex<Option<Project>>>,
    pub events_tx: broadcast::Sender<ServerEvent>,
    pub ai_config: Arc<Mutex<AiConfig>>,
    /// Clip IDs currently being generated — prevents duplicate requests.
    pub generating: Arc<Mutex<HashSet<uuid::Uuid>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            project: Arc::new(Mutex::new(None)),
            events_tx: tx,
            ai_config: Arc::new(Mutex::new(AiConfig::default())),
            generating: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
