use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    ApplyTimelineChildrenCommand, CommandEnvelope, CreateTimelineNodeCommand,
    CreateTimelineRelationshipCommand, DeleteTimelineNodeCommand,
    DeleteTimelineRelationshipCommand, ObjectKind, ProjectionEnvelope, SetTimelineNodeLockCommand,
    SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
    TimelineRenderProjection,
};
use eidetic_core::timeline::Timeline;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store::{self, RecordChangeOutcome};
use crate::state::{AppState, ServerEvent};
use crate::timeline_command::{self, TimelineCommandError};
use crate::validation;
use crate::ydoc::DocCommand;
use crate::{timeline_node_store, timeline_relationship_store};

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/commands/timeline/node-range",
            post(set_timeline_node_range),
        )
        .route("/commands/timeline/create-node", post(create_timeline_node))
        .route(
            "/commands/timeline/create-relationship",
            post(create_timeline_relationship),
        )
        .route(
            "/commands/timeline/delete-relationship",
            post(delete_timeline_relationship),
        )
        .route(
            "/commands/timeline/apply-children",
            post(apply_timeline_children),
        )
        .route("/commands/timeline/split-node", post(split_timeline_node))
        .route("/commands/timeline/node-lock", post(set_timeline_node_lock))
        .route(
            "/commands/timeline/node-notes",
            post(set_timeline_node_notes),
        )
        .route("/commands/timeline/delete-node", post(delete_timeline_node))
}

#[derive(Debug, Serialize)]
struct TimelineCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<TimelineRenderProjection>,
}

async fn set_timeline_node_range(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeRangeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_range_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_set_timeline_node_range(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn create_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateTimelineNodeCommand>>,
) -> ApiJson {
    validation::validate_name(&command.payload.name, "node name")?;
    let path = active_project_path(&state)?;
    let created_node_id = command.payload.node_id;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_create_timeline_node_history(&mut conn, project, &command, 0)
                .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_create_timeline_node(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::EnsureNode {
            node_id: created_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn apply_timeline_children(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<ApplyTimelineChildrenCommand>>,
) -> ApiJson {
    for child in &command.payload.children {
        validation::validate_name(&child.name, "child node name")?;
    }
    let path = active_project_path(&state)?;
    let children = command.payload.children.clone();
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_apply_timeline_children_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        let projection = if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_timeline_children(project, &command)
                .map_err(map_timeline_command_error)?
        } else {
            ProjectionEnvelope::initial(TimelineRenderProjection::from_timeline(&project.timeline))
        };
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
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
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn create_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<CreateTimelineRelationshipCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_create_timeline_relationship_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_create_timeline_relationship(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn delete_timeline_relationship(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineRelationshipCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_delete_timeline_relationship_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_delete_timeline_relationship(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn split_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SplitTimelineNodeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_split_timeline_node_history(&mut conn, project, &command, 0)
                .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_split_timeline_node(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn set_timeline_node_lock(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeLockCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_lock_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_set_timeline_node_lock(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::NodeUpdated {
            node_id: command.payload.node_id.0,
        });
        state.trigger_save();
    }
    crate::error::json_value(response)
}

async fn set_timeline_node_notes(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetTimelineNodeNotesCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let node_id = command.payload.node_id;
    let notes = command.payload.notes.clone();
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = timeline_command::record_set_timeline_node_notes_history(
            &mut conn, project, &command, 0,
        )
        .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_set_timeline_node_notes(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

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
    crate::error::json_value(response)
}

async fn delete_timeline_node(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteTimelineNodeCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let removed_node_id = command.payload.node_id;
    let response = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        history_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            timeline_command::record_delete_timeline_node_history(&mut conn, project, &command, 0)
                .map_err(map_timeline_command_error)?;
        if outcome == RecordChangeOutcome::Recorded {
            timeline_command::apply_delete_timeline_node(project, &command)
                .map_err(map_timeline_command_error)?;
        }
        let projection = timeline_render_projection_from_current_state(&conn, &project.timeline)
            .map_err(map_timeline_command_error)?;
        TimelineCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.doc_tx.try_send(DocCommand::RemoveNode {
            node_id: removed_node_id,
        });
        let _ = state.events_tx.send(ServerEvent::TimelineChanged);
        let _ = state.events_tx.send(ServerEvent::HierarchyChanged);
        state.trigger_save();
    }
    crate::error::json_value(response)
}

fn map_timeline_command_error(error: TimelineCommandError) -> ApiError {
    match error {
        TimelineCommandError::Core(error) => ApiError::bad_request(error.to_string()),
        TimelineCommandError::History(error) => map_history_error(error),
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

#[cfg(test)]
#[path = "commands_timeline_range_tests.rs"]
mod range_tests;

#[cfg(test)]
#[path = "commands_timeline_create_tests.rs"]
mod create_tests;

#[cfg(test)]
#[path = "commands_timeline_children_tests.rs"]
mod children_tests;

#[cfg(test)]
#[path = "commands_timeline_relationship_tests.rs"]
mod relationship_tests;

#[cfg(test)]
#[path = "commands_timeline_split_tests.rs"]
mod split_tests;

#[cfg(test)]
#[path = "commands_timeline_state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "commands_timeline_delete_tests.rs"]
mod delete_tests;
