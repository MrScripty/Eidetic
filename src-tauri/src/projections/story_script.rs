use eidetic_core::contracts::{
    ChangeReviewProjection, ProjectionEnvelope, ScriptDocumentProjection, StoryArcListProjection,
    StoryArcProgressionProjection,
};
use eidetic_server::projection_service::{
    self, ObjectFieldProjectionRequest, ScriptDocumentProjectionRequest,
};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn projection_object_field(
    app: tauri::AppHandle,
    query: ObjectFieldProjectionRequest,
) -> Result<serde_json::Value, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::object_field_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_script_document(
    app: tauri::AppHandle,
    query: ScriptDocumentProjectionRequest,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::script_document_projection(&state, query)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_story_arcs(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::story_arc_list_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_story_arc_progression(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<StoryArcProgressionProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::story_arc_progression_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_change_review(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<ChangeReviewProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::change_review_projection(&state)
        .await
        .map_err(CommandError::from)
}
