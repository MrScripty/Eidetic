use std::path::PathBuf;

use eidetic_core::contracts::{
    AffectDependency, AffectProjection, AffectTarget, AffectValueId, CommandEnvelope,
    DeleteAffectValueCommand, ProjectionEnvelope, RecordAffectDependencyCommand,
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

pub async fn record_affect_dependency(
    state: &AppState,
    command: CommandEnvelope<RecordAffectDependencyCommand>,
) -> Result<Vec<AffectDependency>, BackendError> {
    let path = active_project_path(state)?;
    let affect_id = command.payload.dependency.affect_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        affect_store::record_affect_dependency(&mut conn, &command, 0)
            .map_err(map_history_error)?;
        affect_store::load_affect_dependencies_for_affect(&conn, affect_id)
            .map_err(map_history_error)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("affect dependency record task failed: {error}"))
    })?
}

pub async fn affect_dependencies_for_affect(
    state: &AppState,
    affect_id: AffectValueId,
) -> Result<Vec<AffectDependency>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        affect_store::load_affect_dependencies_for_affect(&conn, affect_id)
            .map_err(map_history_error)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("affect dependency projection task failed: {error}"))
    })?
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

#[cfg(test)]
mod tests {
    use eidetic_core::Template;
    use eidetic_core::contracts::{
        AffectConfidence, AffectProvenance, AffectTarget, AffectValueId, Arousal, CommandEnvelope,
        CommandId, EmotionalIntensity, MoodLabel, SetAffectValueCommand, Valence,
    };
    use eidetic_core::timeline::node::NodeId;

    #[tokio::test]
    async fn affect_projection_replays_after_project_reload() {
        let path = std::env::temp_dir().join(format!(
            "eidetic-affect-service-{}.db",
            uuid::Uuid::new_v4()
        ));
        let target = AffectTarget::TimelineNode {
            node_id: NodeId::new(),
        };
        let state = crate::state::AppState::new().await;
        state.project_database.set_active_path(path.clone());
        *state.project.lock() = Some(Template::MultiCam.build_project("Affect Service Test"));
        let command_id = CommandId::new();
        let command = CommandEnvelope {
            id: command_id,
            payload: SetAffectValueCommand {
                command_id,
                affect_id: AffectValueId::new(),
                target: target.clone(),
                valence: Valence::new(-250).unwrap(),
                arousal: Arousal::new(650).unwrap(),
                intensity: EmotionalIntensity::new(700).unwrap(),
                confidence: AffectConfidence::new(900).unwrap(),
                mood_labels: vec![MoodLabel::new("uneasy").unwrap()],
                provenance: AffectProvenance::UserAuthored,
                rationale: Some("Opening mood".to_string()),
            },
        };

        let initial_projection = super::set_affect_value(&state, command).await.unwrap();
        state.shutdown_tasks();
        let reloaded_state = crate::state::AppState::new().await;
        reloaded_state
            .project_database
            .set_active_path(path.clone());
        *reloaded_state.project.lock() =
            Some(Template::MultiCam.build_project("Affect Service Test"));

        let reloaded_projection = super::affect_projection(&reloaded_state, target)
            .await
            .unwrap();

        assert_eq!(initial_projection.payload, reloaded_projection.payload);
        reloaded_state.shutdown_tasks();
        let _ = std::fs::remove_file(path);
    }
}
