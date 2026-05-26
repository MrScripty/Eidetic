use eidetic_core::contracts::{
    ApplyTimelineChildCommand, CommandEnvelope, CreateTimelineNodeCommand,
    DeleteTimelineNodeCommand, DeleteTimelineRelationshipCommand, ObjectKind, ProjectionEnvelope,
    SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand,
    SplitTimelineNodeCommand, TimelineRenderProjection,
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

pub use crate::command_service_timeline_requests::{
    ApplyTimelineChildrenRequestCommand, CreateTimelineNodeRequestCommand,
    CreateTimelineRelationshipRequestCommand, SplitTimelineNodeRequestCommand,
};

#[derive(Debug, Serialize)]
pub struct TimelineCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<TimelineRenderProjection>,
}

pub async fn create_timeline_node(
    state: &AppState,
    command: CreateTimelineNodeRequestCommand,
) -> Result<TimelineCommandResponse, BackendError> {
    command.validate()?;
    let command = command.into_core_command();
    create_timeline_node_from_core_command(state, command).await
}

pub async fn create_timeline_node_from_core_command(
    state: &AppState,
    command: CommandEnvelope<CreateTimelineNodeCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let created_node_id = command.payload.node_id;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_create_timeline_node_history(&mut conn, &project, &command, 0)
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
        BackendError::internal(format!("timeline node create command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
            node_id: created_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    Ok(response)
}

pub async fn set_timeline_node_range(
    state: &AppState,
    command: CommandEnvelope<SetTimelineNodeRangeCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_range_history(
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
        BackendError::internal(format!("timeline node range command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    Ok(response)
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

pub async fn delete_timeline_relationship(
    state: &AppState,
    command: CommandEnvelope<DeleteTimelineRelationshipCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_delete_timeline_relationship_history(
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
        BackendError::internal(format!(
            "timeline relationship delete command task failed: {error}"
        ))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    Ok(response)
}

pub async fn create_timeline_relationship(
    state: &AppState,
    command: CreateTimelineRelationshipRequestCommand,
) -> Result<TimelineCommandResponse, BackendError> {
    let command = command.into_core_command();
    let path = active_project_path(state)?;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_create_timeline_relationship_history(
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
        BackendError::internal(format!(
            "timeline relationship create command task failed: {error}"
        ))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    Ok(response)
}

pub async fn apply_timeline_children(
    state: &AppState,
    command: ApplyTimelineChildrenRequestCommand,
) -> Result<TimelineCommandResponse, BackendError> {
    command.validate()?;
    let command = command.into_core_command();
    let path = active_project_path(state)?;
    let children = command.payload.children.clone();
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_apply_timeline_children_history(
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
        BackendError::internal(format!(
            "timeline apply children command task failed: {error}"
        ))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let has_bible_references = children_have_bible_references(&children);
        for child in children {
            let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
                node_id: child.node_id,
            });
            if !child.outline.is_empty() {
                let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
                    node_id: child.node_id,
                    field: crate::ydoc::ContentField::Notes,
                    text: child.outline,
                    author: "human:command".into(),
                });
            }
        }
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        if has_bible_references {
            let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
        }
        state.trigger_save();
    }
    Ok(response)
}

fn children_have_bible_references(children: &[ApplyTimelineChildCommand]) -> bool {
    children.iter().any(|child| {
        child
            .characters
            .iter()
            .any(|value| !value.as_str().trim().is_empty())
            || child
                .location
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            || child
                .props
                .iter()
                .any(|value| !value.as_str().trim().is_empty())
    })
}

pub async fn split_timeline_node(
    state: &AppState,
    command: SplitTimelineNodeRequestCommand,
) -> Result<TimelineCommandResponse, BackendError> {
    let command = command.into_core_command();
    split_timeline_node_from_core_command(state, command).await
}

pub async fn split_timeline_node_from_core_command(
    state: &AppState,
    command: CommandEnvelope<SplitTimelineNodeCommand>,
) -> Result<TimelineCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let project = timeline_command_project(state, &path).await?;
    let response = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| BackendError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_split_timeline_node_history(&mut conn, &project, &command, 0)
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
        BackendError::internal(format!("timeline node split command task failed: {error}"))
    })??;

    if response.outcome == RecordChangeOutcome::Recorded {
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

    let mut projection = TimelineRenderProjection::from_timeline(&timeline);
    crate::timeline_affect_overlay::apply_timeline_affect_overlays(
        conn,
        &timeline,
        &mut projection,
    )?;

    Ok(ProjectionEnvelope::initial(projection))
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
