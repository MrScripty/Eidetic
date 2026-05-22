use eidetic_core::contracts::{
    CommandEnvelope, ContextInfluenceProjection, ProjectionEnvelope, RecordContextEvaluationCommand,
};
use eidetic_server::context_influence_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_context_evaluation(
    app: tauri::AppHandle,
    command: CommandEnvelope<RecordContextEvaluationCommand>,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    context_influence_service::record_context_evaluation(&state, command)
        .await
        .map_err(CommandError::from)
}
