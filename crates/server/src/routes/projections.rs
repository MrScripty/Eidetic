use crate::error::{ApiError, ApiJson};
#[cfg(test)]
use crate::history_store;
use crate::state::AppState;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projections/object-field",
            get(get_object_field_projection),
        )
        .route(
            "/projections/bible-graph/node",
            get(get_bible_graph_node_projection),
        )
        .route(
            "/projections/bible-graph/nodes",
            get(get_bible_graph_node_list_projection),
        )
        .route(
            "/projections/bible-graph/schemas",
            get(get_bible_graph_schema_list_projection),
        )
        .route(
            "/projections/bible-graph/render",
            get(get_bible_render_graph_projection),
        )
        .route(
            "/projections/script/document",
            get(get_script_document_projection),
        )
        .route(
            "/projections/story/arcs",
            get(get_story_arc_list_projection),
        )
        .route(
            "/projections/story/arc-progression",
            get(get_story_arc_progression_projection),
        )
        .route(
            "/projections/timeline/render",
            get(get_timeline_render_projection),
        )
        .route(
            "/projections/timeline/selected-node",
            get(get_selected_node_editor_projection),
        )
        .route(
            "/projections/history/changes",
            get(get_change_review_projection),
        )
}

async fn get_object_field_projection(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::ObjectFieldProjectionRequest>,
) -> ApiJson {
    crate::projection_service::object_field_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .map(axum::Json)
}

async fn get_bible_graph_node_projection(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::BibleGraphNodeProjectionRequest>,
) -> ApiJson {
    crate::projection_service::bible_graph_node_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_bible_graph_node_list_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::bible_graph_node_list_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_bible_graph_schema_list_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::bible_graph_schema_list_projection(&state)
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_bible_render_graph_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::bible_render_graph_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_change_review_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::change_review_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_script_document_projection(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::ScriptDocumentProjectionRequest>,
) -> ApiJson {
    crate::projection_service::script_document_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_timeline_render_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::timeline_render_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_selected_node_editor_projection(
    State(state): State<AppState>,
    Query(query): Query<crate::projection_service::SelectedNodeEditorProjectionRequest>,
) -> ApiJson {
    crate::projection_service::selected_node_editor_projection(&state, query)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_story_arc_list_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::story_arc_list_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn get_story_arc_progression_projection(State(state): State<AppState>) -> ApiJson {
    crate::projection_service::story_arc_progression_projection(&state)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

#[cfg(test)]
#[path = "projections_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "projections_story_tests.rs"]
mod story_tests;

#[cfg(test)]
#[path = "projections_history_tests.rs"]
mod history_tests;
