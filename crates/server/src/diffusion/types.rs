//! Pure Rust types for the diffusion manager — no PyO3 dependency.
//!
//! Defines the command enum, update struct, error type, and status
//! used by the channel-based diffusion manager.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::oneshot;

/// Errors originating from the diffusion engine or its Python bridge.
#[derive(Debug, Error)]
pub enum DiffusionError {
    #[error("model not loaded — call POST /ai/diffusion/load first")]
    ModelNotLoaded,
    #[error("model loading failed: {0}")]
    LoadFailed(String),
    #[error("infill failed: {0}")]
    InfillFailed(String),
    #[error("Python error: {0}")]
    PythonError(String),
    #[error("manager channel closed")]
    ChannelClosed,
}

/// Commands sent to the diffusion manager thread via `tokio::sync::mpsc`.
///
/// Each command that expects a response carries a `oneshot::Sender` for the reply.
pub enum DiffuseCmd {
    /// Load model weights into VRAM.
    LoadModel {
        model_path: String,
        device: String,
        reply: oneshot::Sender<Result<(), DiffusionError>>,
    },
    /// Release model from VRAM (symmetric with LoadModel).
    UnloadModel {
        reply: oneshot::Sender<Result<(), DiffusionError>>,
    },
    /// Run diffusion infilling between a prefix and suffix.
    Infill {
        prefix: String,
        suffix: String,
        mask_count: usize,
        steps_per_block: usize,
        block_length: usize,
        temperature: f32,
        dynamic_threshold: f32,
        /// Receives the final generated text on completion.
        reply: oneshot::Sender<Result<String, DiffusionError>>,
    },
    /// Query engine status.
    Status {
        reply: oneshot::Sender<DiffusionStatus>,
    },
    /// Graceful shutdown — unloads model and exits the manager thread.
    Shutdown,
}

/// Progress update streamed between denoising steps.
///
/// Broadcast to WebSocket clients so the UI can show step-by-step progress.
#[derive(Debug, Clone)]
pub struct DiffuseUpdate {
    pub step: usize,
    pub total_steps: usize,
    pub current_text: String,
}

/// Snapshot of the diffusion engine's status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionStatus {
    pub model_loaded: bool,
    pub model_path: Option<String>,
    pub device: String,
}
