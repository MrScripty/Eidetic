use std::sync::Arc;

use eidetic_core::Project;
use parking_lot::Mutex;

/// Shared application state, wrapped in an Arc for use as axum state.
///
/// Currently holds a single in-memory project. Persistence (file save/load)
/// is deferred to Sprint 4.
#[derive(Clone)]
pub struct AppState {
    pub project: Arc<Mutex<Option<Project>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project: Arc::new(Mutex::new(None)),
        }
    }
}
