use eidetic_core::contracts::{
    CommandEnvelope, DeleteTimelineNodeCommand, ObjectKind, ProjectionEnvelope,
    SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand, TimelineRenderProjection,
};
use eidetic_core::timeline::Timeline;
use rusqlite::Connection;
use serde::Serialize;

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::history_store::{self, RecordChangeOutcome};
use crate::state::{AppState, ServerEvent};
use crate::timeline_command::{self, TimelineCommandError};
use crate::ydoc::DocCommand;
use crate::{timeline_node_store, timeline_relationship_store};

#[derive(Debug, Serialize)]
pub struct TimelineCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<TimelineRenderProjection>,
}

pub async fn set_timeline_node_lock(
    state: &AppState,
    command: CommandEnvelope<SetTimelineNodeLockCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let node_id = command.payload.node_id;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_lock_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        Ok::<_, BackendError>(TimelineCommandResponse {
            outcome,
            projection,
        })
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("timeline node lock command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state
            .events_tx
            .send(ServerEvent::NodeUpdated { node_id: node_id.0 });
        state.trigger_save();
    }
    Ok(response)
}

pub async fn set_timeline_node_notes(
    state: &AppState,
    command: CommandEnvelope<SetTimelineNodeNotesCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let node_id = command.payload.node_id;
    let notes = command.payload.notes.clone();
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_notes_history(
            &mut conn, &project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        Ok::<_, BackendError>(TimelineCommandResponse {
            outcome,
            projection,
        })
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("timeline node notes command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
            node_id,
            field: crate::ydoc::ContentField::Notes,
            text: notes,
            author: "human:command".into(),
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state
            .events_tx
            .send(ServerEvent::NodeUpdated { node_id: node_id.0 });
        state.trigger_save();
    }
    Ok(response)
}

pub async fn delete_timeline_node(
    state: &AppState,
    command: CommandEnvelope<DeleteTimelineNodeCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let removed_node_id = command.payload.node_id;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_delete_timeline_node_history(&mut conn, &project, &command, 0)
                .map_err(map_timeline_command_error)?;
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        Ok::<_, BackendError>(TimelineCommandResponse {
            outcome,
            projection,
        })
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("timeline node delete command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::RemoveNode {
            node_id: removed_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    Ok(response)
}

async fn timeline_command_project(
    state: &AppState,
    path: &std::path::Path,
) -> Result<eidetic_core::Project, BackendError> {
    if state.project.lock().is_none() {
        return Err(BackendError::no_project());
    }
    match crate::persistence::load_project(path).await {
        Ok((project, _)) => Ok(project),
        Err(_) => state
            .project
            .lock()
            .clone()
            .ok_or_else(BackendError::no_project),
    }
}

fn timeline_render_projection_from_current_state(
    conn: &Connection,
    fallback: &Timeline,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, TimelineCommandError> {
    let mut timeline = fallback.clone();
    let use_persisted_nodes = timeline_nodes_are_authoritative(conn)?;
    let use_persisted_relationships = timeline_relationships_are_authoritative(conn)?;
    let nodes = timeline_node_store::load_nodes(conn)?;
    if use_persisted_nodes || !nodes.is_empty() {
        timeline.nodes = nodes;
        timeline.node_arcs = timeline_node_store::load_node_arcs(conn)?;
    }
    let relationships = timeline_relationship_store::load_relationships(conn)?;
    if use_persisted_relationships || !relationships.is_empty() || fallback.relationships.is_empty()
    {
        timeline.relationships = relationships;
    }

    Ok(ProjectionEnvelope::initial(
        TimelineRenderProjection::from_timeline(&timeline),
    ))
}

fn timeline_nodes_are_authoritative(conn: &Connection) -> Result<bool, TimelineCommandError> {
    let node_revisions =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::TimelineNode)?
            .revision_count;
    Ok(node_revisions > 0)
}

fn timeline_relationships_are_authoritative(
    conn: &Connection,
) -> Result<bool, TimelineCommandError> {
    let relationship_revisions =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::TimelineRelationship)?
            .revision_count;
    Ok(relationship_revisions > 0)
}

fn map_timeline_command_error(error: TimelineCommandError) -> BackendError {
    match error {
        TimelineCommandError::Core(error) => BackendError::bad_request(error.to_string()),
        TimelineCommandError::History(error) => map_history_error(error),
    }
}
