mod ai;
mod commands;
mod commands_bible;
mod commands_semantic;
mod commands_semantic_child_plan;
mod commands_timeline;
mod export;
mod models;
mod project;
mod projections;
mod projections_semantic;
mod reference;
mod support;

use axum::Router;

use crate::state::AppState;

/// Build the `/api` router with all sub-routes.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(project::router())
        .merge(ai::router())
        .merge(commands::router())
        .merge(commands_semantic::router())
        .merge(commands_semantic_child_plan::router())
        .merge(projections::router())
        .merge(projections_semantic::router())
        .merge(export::router())
        .merge(reference::router())
        .merge(models::router())
}
