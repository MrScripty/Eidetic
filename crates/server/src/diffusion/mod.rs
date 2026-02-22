//! Diffusion LLM engine: channel-based manager for TraDo-8B inference.
//!
//! Mirrors the Y.Doc manager pattern ([`crate::ydoc`]): a dedicated OS
//! thread owns the Python interpreter and GPU model, processing commands
//! received via a `tokio::sync::mpsc` channel.  Denoising progress is
//! streamed step-by-step via a `broadcast` channel.
//!
//! # Thread model
//!
//! The manager runs on a `std::thread` (not a Tokio task) so it can
//! hold the Python GIL indefinitely without starving the async runtime.
//! All PyO3/PyTorch calls happen exclusively on that thread.
//!
//! # Optional infrastructure
//!
//! The diffusion engine is optional — the server starts and operates
//! normally without a loaded model.  Users trigger model loading via
//! `POST /ai/diffusion/load`.  Infill requests when no model is loaded
//! return [`DiffusionError::ModelNotLoaded`].

pub mod types;
mod bridge;
mod manager;

#[cfg(test)]
mod tests;

pub use types::*;

/// Channel capacity for the diffusion command queue.
pub const DIFFUSE_CHANNEL_CAPACITY: usize = 16;

/// Channel capacity for the denoising progress broadcast.
pub const DIFFUSE_BROADCAST_CAPACITY: usize = 64;

/// Spawn the diffusion manager on a dedicated OS thread.
///
/// Returns:
/// - `mpsc::Sender<DiffuseCmd>` — send commands to the manager.
/// - `broadcast::Sender<DiffuseUpdate>` — subscribe for denoising progress.
/// - `std::thread::JoinHandle<()>` — must be joined on shutdown.
pub fn spawn_diffusion_manager() -> (
    tokio::sync::mpsc::Sender<DiffuseCmd>,
    tokio::sync::broadcast::Sender<DiffuseUpdate>,
    std::thread::JoinHandle<()>,
) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(DIFFUSE_CHANNEL_CAPACITY);
    let (update_tx, _) = tokio::sync::broadcast::channel(DIFFUSE_BROADCAST_CAPACITY);
    let update_tx_clone = update_tx.clone();

    let handle = std::thread::Builder::new()
        .name("diffusion-manager".into())
        .spawn(move || manager::run(cmd_rx, update_tx_clone))
        .expect("failed to spawn diffusion manager thread");

    (cmd_tx, update_tx, handle)
}
