use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use eidetic_core::Project;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::persistence;
use crate::vector_store::VectorStore;
use crate::ydoc::{self, DocCommand, DocUpdate};

/// Server configuration constants.
pub mod constants {
    /// Maximum number of undo snapshots to retain.
    pub const UNDO_STACK_DEPTH: usize = 50;
    /// Default AI model identifier.  "auto" means detect whatever Ollama has loaded.
    pub const DEFAULT_AI_MODEL: &str = "auto";
    /// Default AI temperature.
    pub const DEFAULT_TEMPERATURE: f32 = 0.7;
    /// Default max tokens for generation.
    pub const DEFAULT_MAX_TOKENS: usize = 4096;
    /// Default Ollama base URL.
    pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
    /// Minimum gap duration in ms for gap detection.
    pub const GAP_THRESHOLD_MS: u64 = 30_000;
    /// Reference document chunk size in characters.
    pub const REFERENCE_CHUNK_SIZE: usize = 500;
    /// Reference document chunk overlap in characters.
    pub const REFERENCE_CHUNK_OVERLAP: usize = 50;
    /// Embedding model name.
    pub const EMBEDDING_MODEL: &str = "nomic-embed-text";
    /// Number of top RAG results to include.
    pub const RAG_TOP_K: usize = 3;
}

/// Events broadcast to all connected WebSocket clients after mutations.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    TimelineChanged,
    HierarchyChanged,
    NodeUpdated { node_id: uuid::Uuid },
    StoryChanged,
    ProjectMutated,
    GenerationContext {
        node_id: uuid::Uuid,
        system_prompt: String,
        user_prompt: String,
    },
    GenerationProgress {
        node_id: uuid::Uuid,
        token: String,
        tokens_generated: usize,
    },
    GenerationComplete {
        node_id: uuid::Uuid,
    },
    GenerationError {
        node_id: uuid::Uuid,
        error: String,
    },
    ConsistencySuggestion {
        source_node_id: uuid::Uuid,
        target_node_id: uuid::Uuid,
        original_text: String,
        suggested_text: String,
        reason: String,
    },
    ConsistencyComplete {
        source_node_id: uuid::Uuid,
        suggestion_count: usize,
    },
    UndoRedoChanged {
        can_undo: bool,
        can_redo: bool,
    },
    BibleChanged,
    EntityExtractionComplete {
        node_id: uuid::Uuid,
        new_entity_count: usize,
        snapshot_count: usize,
    },
}

/// Snapshot-based undo/redo stack.
///
/// Before each mutation, the current `Project` is cloned onto the undo stack.
/// Undo restores the previous state; redo re-applies undone changes.
/// Capped at `max_depth` entries (~50 snapshots ≈ 2.5MB for a typical project).
pub struct UndoStack {
    undo: Vec<Project>,
    redo: Vec<Project>,
    max_depth: usize,
}

impl UndoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_depth,
        }
    }

    /// Push the current state onto the undo stack before a mutation.
    /// Clears the redo stack (new branch of history).
    pub fn push(&mut self, snapshot: Project) {
        if self.undo.len() >= self.max_depth {
            self.undo.remove(0);
        }
        self.undo.push(snapshot);
        self.redo.clear();
    }

    /// Undo: restore the most recent snapshot. Caller provides current state
    /// which is pushed onto the redo stack.
    pub fn undo(&mut self, current: Project) -> Option<Project> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    /// Redo: re-apply the most recently undone state. Caller provides current
    /// state which is pushed onto the undo stack.
    pub fn redo(&mut self, current: Project) -> Option<Project> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
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
            model: constants::DEFAULT_AI_MODEL.into(),
            temperature: constants::DEFAULT_TEMPERATURE,
            max_tokens: constants::DEFAULT_MAX_TOKENS,
            base_url: constants::DEFAULT_OLLAMA_URL.into(),
            api_key: None,
        }
    }
}

/// Shared application state, wrapped in an Arc for use as axum state.
#[derive(Clone)]
pub struct AppState {
    /// Structural data — single source of truth for hierarchy, timing, arcs.
    pub project: Arc<Mutex<Option<Project>>>,
    pub events_tx: broadcast::Sender<ServerEvent>,
    /// Channel to the Y.Doc manager task (single source of truth for text content).
    pub doc_tx: tokio::sync::mpsc::Sender<DocCommand>,
    /// Broadcasts Y.Doc binary updates to WebSocket clients.
    pub doc_update_tx: broadcast::Sender<DocUpdate>,
    pub ai_config: Arc<Mutex<AiConfig>>,
    /// Node IDs currently being generated — prevents duplicate requests.
    pub generating: Arc<Mutex<HashSet<uuid::Uuid>>>,
    /// Node IDs currently undergoing entity extraction — prevents duplicate/concurrent runs.
    pub extracting: Arc<Mutex<HashSet<uuid::Uuid>>>,
    /// Path where the current project is saved on disk.
    pub project_path: Arc<Mutex<Option<PathBuf>>>,
    /// Snapshot-based undo/redo stack for project mutations.
    pub undo_stack: Arc<Mutex<UndoStack>>,
    /// In-memory vector store for RAG reference material.
    pub vector_store: Arc<Mutex<VectorStore>>,
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

        // Spawn the Y.Doc manager task (owns the CRDT doc, receives commands via channel).
        // The change_rx will be consumed by the AI reactor in a later phase.
        let (doc_tx, doc_update_tx, _change_rx) = ydoc::spawn_doc_manager();

        // Start auto-save (needs doc_tx to serialize Y.Doc state).
        let save_doc_tx = doc_tx.clone();
        tokio::spawn(auto_save_task(save_rx, save_project, save_path, save_doc_tx));

        Self {
            project,
            events_tx,
            doc_tx,
            doc_update_tx,
            ai_config: Arc::new(Mutex::new(AiConfig::default())),
            generating: Arc::new(Mutex::new(HashSet::new())),
            extracting: Arc::new(Mutex::new(HashSet::new())),
            project_path,
            undo_stack: Arc::new(Mutex::new(UndoStack::new(constants::UNDO_STACK_DEPTH))),
            vector_store: Arc::new(Mutex::new(VectorStore::new())),
            save_tx,
        }
    }

    /// Signal that the project has been mutated and should be auto-saved.
    pub fn trigger_save(&self) {
        let _ = self.save_tx.try_send(());
    }

    /// Snapshot the current project state before a mutation for undo support.
    ///
    /// Call this at the start of every mutation handler, before acquiring
    /// the project lock for writing.
    pub fn snapshot_for_undo(&self) {
        let project_guard = self.project.lock();
        if let Some(p) = project_guard.as_ref() {
            let snapshot = p.clone();
            drop(project_guard);
            let mut undo = self.undo_stack.lock();
            undo.push(snapshot);
            let can_undo = undo.can_undo();
            let can_redo = undo.can_redo();
            drop(undo);
            let _ = self.events_tx.send(ServerEvent::UndoRedoChanged {
                can_undo,
                can_redo,
            });
        }
    }
}

/// Background task that debounces save signals and writes to disk.
async fn auto_save_task(
    mut rx: tokio::sync::mpsc::Receiver<()>,
    project: Arc<Mutex<Option<Project>>>,
    project_path: Arc<Mutex<Option<PathBuf>>>,
    doc_tx: tokio::sync::mpsc::Sender<ydoc::DocCommand>,
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

        // Serialize Y.Doc state alongside structural data.
        let ydoc_state = ydoc::serialize_doc(&doc_tx).await;

        if let Err(e) = persistence::save_project(&proj_json, &path, ydoc_state).await {
            tracing::error!("auto-save failed: {e}");
        }
    }
}
