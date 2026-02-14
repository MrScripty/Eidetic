use std::sync::Arc;

use eidetic_core::Project;
use parking_lot::Mutex;
use serde::Serialize;
use tokio::sync::broadcast;

/// Events broadcast to all connected WebSocket clients after mutations.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    TimelineChanged,
    ScenesChanged,
    BeatUpdated { clip_id: uuid::Uuid },
    StoryChanged,
}

/// Shared application state, wrapped in an Arc for use as axum state.
///
/// Currently holds a single in-memory project. Persistence (file save/load)
/// is deferred to Sprint 4.
#[derive(Clone)]
pub struct AppState {
    pub project: Arc<Mutex<Option<Project>>>,
    pub events_tx: broadcast::Sender<ServerEvent>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            project: Arc::new(Mutex::new(None)),
            events_tx: tx,
        }
    }
}
