//! Diffusion manager: dedicated OS thread owning Python GIL + GPU model.
//!
//! Thread requirements:
//! - Runs on a `std::thread` (NOT a Tokio task).
//! - Owns the Python GIL for the lifetime of the thread.
//! - All PyO3 calls happen exclusively on this thread.
//! - Uses `tokio::sync::mpsc::Receiver::blocking_recv()` for command intake.
//!
//! The manager processes commands sequentially. Only one infill or load
//! operation runs at a time — the bounded mpsc channel (capacity 16)
//! provides natural backpressure.

use tokio::sync::{broadcast, mpsc};
use pyo3::Python;

use super::bridge;
use super::types::{DiffuseCmd, DiffuseUpdate, DiffusionError, DiffusionStatus};

/// Main loop of the diffusion manager thread.
///
/// Called from `std::thread::spawn` — blocks indefinitely until
/// `DiffuseCmd::Shutdown` is received or the command channel closes.
pub fn run(mut rx: mpsc::Receiver<DiffuseCmd>, update_tx: broadcast::Sender<DiffuseUpdate>) {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let engine = match bridge::create_engine(py) {
            Ok(e) => e,
            Err(e) => {
                // Optional infrastructure — server continues without diffusion.
                tracing::error!("Failed to initialize Python diffusion engine: {e}");
                drain_commands_with_error(rx, &e);
                return;
            }
        };

        tracing::info!("Diffusion manager thread started");

        let mut model_path: Option<String> = None;
        let mut device = "cpu".to_string();

        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                DiffuseCmd::LoadModel {
                    model_path: path,
                    device: dev,
                    reply,
                } => {
                    let result = bridge::load_model(py, &engine, &path, &dev);
                    if result.is_ok() {
                        model_path = Some(path);
                        device = dev;
                        tracing::info!(
                            "Diffusion model loaded: {}",
                            model_path.as_deref().unwrap_or("?")
                        );
                    }
                    let _ = reply.send(result);
                }

                DiffuseCmd::UnloadModel { reply } => {
                    let result = bridge::unload_model(py, &engine);
                    if result.is_ok() {
                        model_path = None;
                        tracing::info!("Diffusion model unloaded");
                    }
                    let _ = reply.send(result);
                }

                DiffuseCmd::Infill {
                    prefix,
                    suffix,
                    mask_count,
                    steps_per_block,
                    block_length,
                    temperature,
                    dynamic_threshold,
                    reply,
                } => {
                    if !bridge::is_loaded(py, &engine) {
                        let _ = reply.send(Err(DiffusionError::ModelNotLoaded));
                        continue;
                    }

                    match bridge::run_infill(
                        py,
                        &engine,
                        &prefix,
                        &suffix,
                        mask_count,
                        steps_per_block,
                        block_length,
                        temperature,
                        dynamic_threshold,
                    ) {
                        Ok(updates) => {
                            let mut final_text = String::new();
                            for update in updates {
                                final_text.clone_from(&update.current_text);
                                let _ = update_tx.send(update);
                            }
                            let _ = reply.send(Ok(final_text));
                        }
                        Err(e) => {
                            let _ = reply.send(Err(e));
                        }
                    }
                }

                DiffuseCmd::Status { reply } => {
                    let loaded = bridge::is_loaded(py, &engine);
                    let _ = reply.send(DiffusionStatus {
                        model_loaded: loaded,
                        model_path: model_path.clone(),
                        device: device.clone(),
                    });
                }

                DiffuseCmd::Shutdown => {
                    let _ = bridge::unload_model(py, &engine);
                    tracing::info!("Diffusion manager shutting down");
                    break;
                }
            }
        }
    });
}

/// When the Python engine fails to initialize, drain pending commands
/// with an error so callers don't hang on their oneshot receivers.
fn drain_commands_with_error(mut rx: mpsc::Receiver<DiffuseCmd>, init_error: &DiffusionError) {
    let msg = format!("diffusion engine unavailable: {init_error}");
    while let Some(cmd) = rx.blocking_recv() {
        match cmd {
            DiffuseCmd::LoadModel { reply, .. } => {
                let _ = reply.send(Err(DiffusionError::PythonError(msg.clone())));
            }
            DiffuseCmd::UnloadModel { reply } => {
                let _ = reply.send(Err(DiffusionError::PythonError(msg.clone())));
            }
            DiffuseCmd::Infill { reply, .. } => {
                let _ = reply.send(Err(DiffusionError::PythonError(msg.clone())));
            }
            DiffuseCmd::Status { reply } => {
                let _ = reply.send(DiffusionStatus {
                    model_loaded: false,
                    model_path: None,
                    device: "none".into(),
                });
            }
            DiffuseCmd::Shutdown => break,
        }
    }
}
