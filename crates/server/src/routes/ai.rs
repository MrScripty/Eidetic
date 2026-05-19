use axum::Router;
use axum::routing::{get, post, put};

use crate::state::AppState;

#[path = "ai_child_planning.rs"]
mod ai_child_planning;
#[path = "ai_config.rs"]
mod ai_config;
#[path = "ai_context.rs"]
mod ai_context;
#[path = "ai_generation.rs"]
mod ai_generation;
#[path = "ai_generation_runtime.rs"]
mod ai_generation_runtime;
#[path = "ai_support.rs"]
mod ai_support;

pub(crate) use ai_support::{
    active_sqlite_project, attach_ai_bible_context, attach_ai_bible_context_to_children,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(ai_generation::generate))
        .route(
            "/ai/generate-children",
            post(ai_child_planning::generate_children),
        )
        .route("/ai/generate-batch", post(ai_generation::generate_batch))
        .route("/ai/context/{id}", get(ai_context::preview_context))
        .route("/ai/status", get(ai_config::status))
        .route("/ai/config", put(ai_config::config))
}

#[cfg(test)]
#[path = "ai_context_tests.rs"]
mod ai_context_tests;
