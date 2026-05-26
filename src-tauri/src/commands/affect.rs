use eidetic_core::contracts::{
    AffectProjection, CommandEnvelope, ProjectionEnvelope, SetAffectValueCommand,
};
use eidetic_server::affect_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_affect_set(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetAffectValueCommand>,
) -> Result<ProjectionEnvelope<AffectProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::set_affect_value(&state, command)
        .await
        .map_err(CommandError::from)
}
