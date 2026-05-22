use eidetic_core::contracts::{
    BibleGraphNodeListProjection, BibleGraphSchemaListProjection, BibleNodeDetailProjection,
    BibleRenderGraphProjection, BibleRenderGraphProjectionRequest, ProjectionEnvelope,
};
use eidetic_server::bible_render_graph_projection;
use eidetic_server::projection_service::{self, BibleGraphNodeProjectionRequest};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn projection_bible_graph_node(
    app: tauri::AppHandle,
    query: BibleGraphNodeProjectionRequest,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_graph_node_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_bible_graph_nodes(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::bible_graph_node_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn projection_bible_graph_schemas(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<BibleGraphSchemaListProjection>, CommandError> {
    let state = app.state::<AppState>();
    projection_service::bible_graph_schema_list_projection(&state).map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_bible_render_graph(
    app: tauri::AppHandle,
    query: Option<BibleRenderGraphProjectionRequest>,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    bible_render_graph_projection::bible_render_graph_projection(&state, query.unwrap_or_default())
        .await
        .map_err(CommandError::from)
}
