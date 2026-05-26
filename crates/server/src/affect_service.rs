use std::path::PathBuf;

use eidetic_core::contracts::{
    AffectProjection, AffectTarget, CommandEnvelope, DeleteAffectValueCommand, ProjectionEnvelope,
    SetAffectValueCommand,
};

use crate::affect_store;
use crate::backend_error::BackendError;
use crate::history_store::HistoryStoreError;
use crate::state::AppState;

pub async fn set_affect_value(
    state: &AppState,
    command: CommandEnvelope<SetAffectValueCommand>,
) -> Result<ProjectionEnvelope<AffectProjection>, BackendError> {
    let path = active_project_path(state)?;
    let target = command.payload.target.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        affect_store::record_set_affect_value(&mut conn, &command, 0).map_err(map_history_error)?;
        affect_store::load_affect_projection(&conn, target).map_err(map_history_error)
    })
    .await
    .map_err(|error| BackendError::internal(format!("affect set task failed: {error}")))?
}

pub async fn delete_affect_value(
    state: &AppState,
    command: CommandEnvelope<DeleteAffectValueCommand>,
    target: AffectTarget,
) -> Result<ProjectionEnvelope<AffectProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        affect_store::record_delete_affect_value(&mut conn, &command, 0)
            .map_err(map_history_error)?;
        affect_store::load_affect_projection(&conn, target).map_err(map_history_error)
    })
    .await
    .map_err(|error| BackendError::internal(format!("affect delete task failed: {error}")))?
}

pub async fn affect_projection(
    state: &AppState,
    target: AffectTarget,
) -> Result<ProjectionEnvelope<AffectProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        affect_store::load_affect_projection(&conn, target).map_err(map_history_error)
    })
    .await
    .map_err(|error| BackendError::internal(format!("affect projection task failed: {error}")))?
}

fn active_project_path(state: &AppState) -> Result<PathBuf, BackendError> {
    state
        .project_database
        .active_path()
        .ok_or_else(BackendError::no_project)
}

fn map_history_error(error: HistoryStoreError) -> BackendError {
    BackendError::internal(error.to_string())
}
