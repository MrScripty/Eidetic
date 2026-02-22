//! Thin PyO3 wrapper around `diffusion_engine.py`.
//!
//! Validates data at the Rust <-> Python boundary and converts Python
//! exceptions into [`DiffusionError`] values.
//!
//! Thread requirements: All functions in this module MUST be called from
//! the diffusion manager thread while holding the Python GIL.
//! Do NOT call from async Tokio tasks.

use std::ffi::CStr;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use super::types::{DiffuseUpdate, DiffusionError};

/// Embedded Python source — compiled into the binary.
const PYTHON_SOURCE: &str = include_str!("../../python/diffusion_engine.py");

/// Create a new `DiffusionEngine` Python instance.
///
/// The returned `Py<PyAny>` is an owned reference safe to hold across
/// GIL releases on the same thread.
pub fn create_engine(py: Python<'_>) -> Result<Py<PyAny>, DiffusionError> {
    let module = PyModule::from_code(
        py,
        CStr::from_bytes_with_nul(
            // PyO3 0.23 requires &CStr for from_code arguments.
            // We leak a CString since this runs once at startup.
            std::ffi::CString::new(PYTHON_SOURCE)
                .map_err(|e| {
                    DiffusionError::PythonError(format!("Python source contains null byte: {e}"))
                })?
                .into_bytes_with_nul()
                .leak(),
        )
        .unwrap(),
        c"diffusion_engine.py",
        c"diffusion_engine",
    )
    .map_err(|e| DiffusionError::PythonError(format!("failed to load Python module: {e}")))?;

    let engine_class = module
        .getattr("DiffusionEngine")
        .map_err(|e| {
            DiffusionError::PythonError(format!("DiffusionEngine class not found: {e}"))
        })?;

    let engine = engine_class
        .call0()
        .map_err(|e| {
            DiffusionError::PythonError(format!("failed to instantiate engine: {e}"))
        })?;

    Ok(engine.unbind())
}

/// Load a model into the engine. Validates inputs at the boundary.
pub fn load_model(
    py: Python<'_>,
    engine: &Py<PyAny>,
    model_path: &str,
    device: &str,
) -> Result<(), DiffusionError> {
    // Boundary validation.
    if model_path.is_empty() {
        return Err(DiffusionError::LoadFailed("model_path is empty".into()));
    }
    if device != "cuda" && device != "cpu" {
        return Err(DiffusionError::LoadFailed(format!(
            "invalid device '{device}', expected 'cuda' or 'cpu'"
        )));
    }

    engine
        .call_method1(py, "load_model", (model_path, device))
        .map_err(|e| DiffusionError::LoadFailed(format!("{e}")))?;

    Ok(())
}

/// Unload the model from the engine (symmetric with `load_model`).
pub fn unload_model(py: Python<'_>, engine: &Py<PyAny>) -> Result<(), DiffusionError> {
    engine
        .call_method0(py, "unload_model")
        .map_err(|e| DiffusionError::PythonError(format!("unload failed: {e}")))?;
    Ok(())
}

/// Check whether the engine has a model loaded.
pub fn is_loaded(py: Python<'_>, engine: &Py<PyAny>) -> bool {
    engine
        .call_method0(py, "is_loaded")
        .and_then(|r| r.extract::<bool>(py))
        .unwrap_or(false)
}

/// Run the infill generator and collect step results.
///
/// Each yielded tuple is immediately copied into Rust-owned data.
/// The Python generator is iterated to completion on the calling thread.
///
/// Returns a `Vec` of `DiffuseUpdate` values (one per denoising step).
pub fn run_infill(
    py: Python<'_>,
    engine: &Py<PyAny>,
    prefix: &str,
    suffix: &str,
    mask_count: usize,
    steps_per_block: usize,
    block_length: usize,
    temperature: f32,
    dynamic_threshold: f32,
) -> Result<Vec<DiffuseUpdate>, DiffusionError> {
    // Boundary validation.
    if mask_count == 0 {
        return Err(DiffusionError::InfillFailed(
            "mask_count must be > 0".into(),
        ));
    }
    if steps_per_block == 0 {
        return Err(DiffusionError::InfillFailed(
            "steps_per_block must be > 0".into(),
        ));
    }
    if block_length == 0 {
        return Err(DiffusionError::InfillFailed(
            "block_length must be > 0".into(),
        ));
    }

    // Call the Python infill generator.
    let generator = engine
        .call_method1(
            py,
            "infill",
            (
                prefix,
                suffix,
                mask_count,
                steps_per_block,
                block_length,
                temperature,
                dynamic_threshold,
            ),
        )
        .map_err(|e| DiffusionError::InfillFailed(format!("{e}")))?;

    // Iterate the generator, copying each yielded tuple into Rust memory.
    let mut updates = Vec::new();
    let iter = generator
        .bind(py)
        .try_iter()
        .map_err(|e| DiffusionError::InfillFailed(format!("generator not iterable: {e}")))?;

    for item in iter {
        let item =
            item.map_err(|e| DiffusionError::InfillFailed(format!("step error: {e}")))?;
        let (step, total_steps, text): (usize, usize, String) = item
            .extract()
            .map_err(|e| DiffusionError::InfillFailed(format!("bad yield shape: {e}")))?;

        updates.push(DiffuseUpdate {
            step,
            total_steps,
            current_text: text,
        });
    }

    Ok(updates)
}
