use crate::error::{ApiError, ApiJson};
use crate::state::AppState;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projections/semantic/bible-reference-proposals",
            get(get_bible_reference_proposal_list),
        )
        .route(
            "/projections/semantic/dependencies",
            get(get_semantic_dependencies),
        )
        .route(
            "/projections/semantic/propagation-proposals",
            get(get_propagation_proposal_list),
        )
        .route(
            "/projections/semantic/child-plans",
            get(get_child_plan_list),
        )
}

async fn get_bible_reference_proposal_list(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::bible_reference_proposal_list_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_semantic_dependencies(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::SemanticDependencyProjectionRequest>,
) -> ApiJson {
    crate::projection_service::semantic_dependency_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_propagation_proposal_list(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::propagation_proposal_list_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_child_plan_list(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::child_plan_list_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

#[cfg(test)]
#[path = "projections_semantic_tests.rs"]
mod tests;
