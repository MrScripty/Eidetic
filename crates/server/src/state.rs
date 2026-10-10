use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use eidetic_core::Project;
use eidetic_core::timeline::node::NodeId;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::backend_task::BackendTaskSupervisor;
use crate::persistence;
use crate::project_database::ProjectDatabase;
use crate::vector_store::VectorStore;
use crate::ydoc::{self, DocCommand, DocUpdate};
use pumas_library::ModelLibrary;

/// Server configuration constants.
pub mod constants {
    /// Default AI model identifier. "auto" means detect the external server's available model.
    pub const DEFAULT_AI_MODEL: &str = "auto";
    /// Default AI temperature.
    pub const DEFAULT_TEMPERATURE: f32 = 0.7;
    /// Default max tokens for generation.
    pub const DEFAULT_MAX_TOKENS: usize = 4096;
    /// Default Pumas llama.cpp OpenAI-compatible base URL.
    pub const DEFAULT_LLAMACPP_URL: &str = "http://127.0.0.1:18080/v1";
    /// Reference document chunk size in characters.
    pub const REFERENCE_CHUNK_SIZE: usize = 500;
    /// Reference document chunk overlap in characters.
    pub const REFERENCE_CHUNK_OVERLAP: usize = 50;
    /// Number of top RAG results to include.
    pub const RAG_TOP_K: usize = 3;
}

/// Events broadcast to desktop event subscribers after mutations.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    TimelineChanged,
    HierarchyChanged,
    NodeUpdated {
        node_id: uuid::Uuid,
    },
    StoryChanged,
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
    ReferenceRetrieval {
        node_id: uuid::Uuid,
        sources: Vec<String>,
        error: Option<String>,
    },
    BibleChanged,
    ScriptChanged,
    SemanticProposalsChanged,
    ContextInfluenceChanged {
        target_node_id: uuid::Uuid,
    },
    TimelineSelectionChanged {
        node_id: Option<NodeId>,
    },
    TimelinePlayheadChanged {
        position_ms: u64,
    },
}

/// Which AI backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendType {
    LlamaCpp,
    Pumas,
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
    #[serde(default)]
    pub pumas_profile: String,
    #[serde(default)]
    pub embedding: crate::embeddings::EmbeddingConfig,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            backend_type: BackendType::LlamaCpp,
            model: constants::DEFAULT_AI_MODEL.into(),
            temperature: constants::DEFAULT_TEMPERATURE,
            max_tokens: constants::DEFAULT_MAX_TOKENS,
            base_url: constants::DEFAULT_LLAMACPP_URL.into(),
            api_key: None,
            pumas_profile: String::new(),
            embedding: crate::embeddings::EmbeddingConfig::default(),
        }
    }
}

/// Shared application state, wrapped in an Arc for desktop command adapters.
#[derive(Clone)]
pub struct AppState {
    /// Loaded project mirror for structural data that has not moved to SQLite stores yet.
    pub project: Arc<Mutex<Option<Project>>>,
    /// Serializes timeline work with project/document transitions and save snapshots.
    pub(crate) project_session_gate: Arc<tokio::sync::Mutex<()>>,
    /// Renewed on replacement or save-as, including reopening identical node IDs.
    pub(crate) project_session_id: Arc<Mutex<uuid::Uuid>>,
    pub events_tx: broadcast::Sender<ServerEvent>,
    /// Channel to the Y.Doc manager task (single source of truth for text content).
    pub doc_tx: tokio::sync::mpsc::Sender<DocCommand>,
    /// Broadcasts Y.Doc binary updates to document update subscribers.
    pub doc_update_tx: broadcast::Sender<DocUpdate>,
    pub ai_config: Arc<Mutex<AiConfig>>,
    /// Node IDs currently being generated — prevents duplicate requests.
    pub generating: Arc<Mutex<HashSet<uuid::Uuid>>>,
    /// Transitional test access to the active project path while older fixtures
    /// are moved onto `ProjectDatabase`.
    #[cfg(test)]
    pub project_path: Arc<Mutex<Option<PathBuf>>>,
    /// Active SQLite database owner for command and projection services.
    pub project_database: ProjectDatabase,
    /// In-memory vector store for RAG reference material.
    pub vector_store: Arc<Mutex<VectorStore>>,
    /// Channel to signal the auto-save background task.
    save_tx: tokio::sync::mpsc::Sender<()>,
    /// Model library from Pumas for listing available local models.
    pub model_library: Option<Arc<ModelLibrary>>,
    pub(crate) pumas_runtime: Option<Arc<crate::pumas_runtime::PumasRuntime>>,
    /// Backend-owned transient timeline selection projected to renderers and UI.
    pub selected_timeline_node_id: Arc<Mutex<Option<NodeId>>>,
    /// Backend-owned transient playhead position projected across timeline surfaces.
    pub timeline_playhead_ms: Arc<Mutex<u64>>,
    /// Owns long-running backend tasks so desktop shutdown can stop them.
    pub task_supervisor: BackendTaskSupervisor,
}

impl AppState {
    pub async fn new() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        let (save_tx, save_rx) = tokio::sync::mpsc::channel(16);

        let project = Arc::new(Mutex::new(None));
        let project_session_gate = Arc::new(tokio::sync::Mutex::new(()));
        let project_path = Arc::new(Mutex::new(None::<PathBuf>));
        let project_database = ProjectDatabase::new(project_path.clone());
        let task_supervisor = BackendTaskSupervisor::default();

        // Spawn the debounced auto-save background task.
        let save_project = project.clone();
        let save_path = project_path.clone();

        // Spawn the Y.Doc manager task (owns the CRDT doc, receives commands via channel).
        let (doc_tx, doc_update_tx) = ydoc::spawn_doc_manager(&task_supervisor);

        // Start auto-save (needs doc_tx to serialize Y.Doc state).
        let save_doc_tx = doc_tx.clone();
        task_supervisor.spawn(
            "auto-save",
            auto_save_task(
                save_rx,
                save_project,
                save_path,
                save_doc_tx,
                project_session_gate.clone(),
            ),
        );

        // Initialize the Pumas model library (optional — best-effort).
        let model_library = Self::init_model_library().await;

        let pumas_runtime = match crate::pumas_runtime::PumasRuntime::from_environment().await {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!(%error,"Pumas connection initialization failed");
                None
            }
        };
        let mut ai_config = AiConfig::default();
        if let Some(runtime) = &pumas_runtime {
            ai_config.backend_type = BackendType::Pumas;
            ai_config.base_url = runtime.description.endpoint.as_str().to_owned();
            ai_config.model.clear();
            ai_config.embedding.base_url = ai_config.base_url.clone();
        }

        Self {
            project,
            project_session_gate,
            project_session_id: Arc::new(Mutex::new(uuid::Uuid::new_v4())),
            events_tx,
            doc_tx,
            doc_update_tx,
            ai_config: Arc::new(Mutex::new(ai_config)),
            generating: Arc::new(Mutex::new(HashSet::new())),
            #[cfg(test)]
            project_path,
            project_database,
            vector_store: Arc::new(Mutex::new(VectorStore::new())),
            save_tx,
            model_library,
            pumas_runtime,
            selected_timeline_node_id: Arc::new(Mutex::new(None)),
            timeline_playhead_ms: Arc::new(Mutex::new(0)),
            task_supervisor,
        }
    }

    pub fn select_timeline_node(&self, node_id: Option<NodeId>) {
        *self.selected_timeline_node_id.lock() = node_id;
        let _ = self
            .events_tx
            .send(ServerEvent::TimelineSelectionChanged { node_id });
    }

    pub fn set_timeline_playhead(&self, position_ms: u64) {
        *self.timeline_playhead_ms.lock() = position_ms;
        let _ = self
            .events_tx
            .send(ServerEvent::TimelinePlayheadChanged { position_ms });
    }

    pub fn shutdown_tasks(&self) {
        self.task_supervisor.abort_all();
    }

    pub async fn shutdown_tasks_async(&self) {
        self.task_supervisor.shutdown_all().await;
        if let Some(runtime) = &self.pumas_runtime {
            if let Err(error) = runtime.shutdown().await {
                tracing::error!(%error,"Owned Pumas shutdown failed");
            }
        }
    }

    /// Initialize the Pumas model library from env or sibling directory.
    ///
    /// Looks for `PUMAS_MODELS_DIR` env var first, then tries a sibling
    /// `Pumas-Library/shared-resources/models/` directory relative to the binary.
    async fn init_model_library() -> Option<Arc<ModelLibrary>> {
        // 1. Explicit env var
        if let Ok(dir) = std::env::var("PUMAS_MODELS_DIR") {
            let path = PathBuf::from(&dir);
            if path.is_dir() {
                match ModelLibrary::new(&path).await {
                    Ok(lib) => {
                        tracing::info!("Pumas model library loaded from PUMAS_MODELS_DIR: {dir}");
                        return Some(Arc::new(lib));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to open model library at {dir}: {e}");
                    }
                }
            }
        }

        // 2. Well-known sibling path (for co-located installs)
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        if let Some(exe_dir) = exe_dir {
            // Walk up to find the workspace root (contains Cargo.toml)
            let mut candidate = exe_dir.as_path();
            for _ in 0..6 {
                let sibling = candidate
                    .parent()
                    .map(|p| p.join("Pumas-Library/shared-resources/models"));
                if let Some(ref path) = sibling {
                    if path.is_dir() {
                        match ModelLibrary::new(path).await {
                            Ok(lib) => {
                                tracing::info!(
                                    "Pumas model library loaded from sibling: {}",
                                    path.display()
                                );
                                return Some(Arc::new(lib));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to open model library at {}: {e}",
                                    path.display()
                                );
                            }
                        }
                    }
                }
                match candidate.parent() {
                    Some(p) => candidate = p,
                    None => break,
                }
            }
        }

        tracing::info!("No Pumas model library found (set PUMAS_MODELS_DIR to enable)");
        None
    }

    /// Signal that the project has been mutated and should be auto-saved.
    pub fn trigger_save(&self) {
        let _ = self.save_tx.try_send(());
    }
}

/// Once admitted, session work outlives a disconnected/cancelled caller. The
/// owned gate stays with the work through blocking persistence and publication.
/// Waiting for admission is still cancellable; backend shutdown aborts the task.
pub(crate) async fn complete_project_session_work<T>(
    state: &AppState,
    session: tokio::sync::OwnedMutexGuard<()>,
    work: impl std::future::Future<Output = Result<T, crate::backend_error::BackendError>>
    + Send
    + 'static,
) -> Result<T, crate::backend_error::BackendError>
where
    T: Send + 'static,
{
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    state
        .task_supervisor
        .spawn("project-session-work", async move {
            let result = work.await;
            drop(session);
            if let Err(Err(error)) = reply_tx.send(result) {
                tracing::error!("project session work failed after caller cancellation: {error}");
            }
        });
    reply_rx.await.map_err(|_| {
        crate::backend_error::BackendError::internal(
            "project session task stopped before completion",
        )
    })?
}

/// Background task that debounces save signals and writes to disk.
async fn auto_save_task(
    mut rx: tokio::sync::mpsc::Receiver<()>,
    project: Arc<Mutex<Option<Project>>>,
    project_path: Arc<Mutex<Option<PathBuf>>>,
    doc_tx: tokio::sync::mpsc::Sender<ydoc::DocCommand>,
    project_session_gate: Arc<tokio::sync::Mutex<()>>,
) {
    loop {
        // Wait for the first save signal.
        if rx.recv().await.is_none() {
            break;
        }

        // Debounce: wait 2 seconds, draining any additional signals.
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        while rx.try_recv().is_ok() {}

        // Keep mirror, path and serialized document in one participating session
        // until persistence completes; a project switch must not mix these inputs.
        let _session = project_session_gate.lock().await;

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
        let Some(ydoc_state) = ydoc::serialize_doc(&doc_tx).await else {
            tracing::error!("auto-save failed: document serialization unavailable");
            continue;
        };

        #[cfg(test)]
        crate::write_concurrency_probe::observe_autosave(
            &path,
            crate::write_concurrency_probe::AutosaveStage::BeforePersistence,
        );
        let saved = persistence::save_project(&proj_json, &path, Some(ydoc_state)).await;
        #[cfg(test)]
        crate::write_concurrency_probe::observe_autosave(
            &path,
            crate::write_concurrency_probe::AutosaveStage::Persisted(saved.clone()),
        );
        if let Err(e) = saved {
            tracing::error!("auto-save failed: {e}");
        }
    }
}

#[cfg(test)]
#[path = "state_autosave_custody_tests.rs"]
mod autosave_custody_tests;
