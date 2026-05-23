use eidetic_core::contracts::{
    BibleGraphNodeListProjection, BibleGraphSchemaListProjection, BibleNodeDetailProjection,
    BibleRenderGraphProjection, BibleRenderGraphProjectionRequest, ProjectionEnvelope,
};
use eidetic_server::bible_render_graph_projection;
use eidetic_server::projection_service::{self, BibleGraphNodeProjectionRequest};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
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
    let envelope = bible_render_graph_projection::bible_render_graph_projection(
        &state,
        query.unwrap_or_default(),
    )
    .await
    .map_err(CommandError::from)?;
    mirror_bible_render_graph_projection(&app, envelope.payload.clone());
    Ok(envelope)
}

fn mirror_bible_render_graph_projection(
    app: &tauri::AppHandle,
    projection: BibleRenderGraphProjection,
) {
    if let Some(graph_owner) = app.try_state::<DesktopBibleGraphRendererOwner>()
        && let Err(error) = graph_owner.update_projection_if_open(projection)
    {
        tracing::warn!(
            "failed to mirror bible render graph projection to Bevy renderer: {error:?}"
        );
    }
}
