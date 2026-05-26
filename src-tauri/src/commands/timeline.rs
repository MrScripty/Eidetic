use eidetic_core::contracts::{
    CommandEnvelope, DeleteTimelineNodeCommand, DeleteTimelineRelationshipCommand,
    SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand,
};
use eidetic_server::command_service;
use eidetic_server::projection_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelinePlayheadCommandResponse {
    pub position_ms: u64,
}

#[tauri::command]
pub async fn command_timeline_create_node(
    app: tauri::AppHandle,
    command: command_service::CreateTimelineNodeRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_create_child_from_parent(
    app: tauri::AppHandle,
    command: command_service::CreateTimelineChildFromParentRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_timeline_child_from_parent(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_range(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeRangeCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_range(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_lock(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeLockCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_lock(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_notes(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeNotesCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_notes(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_delete_node(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteTimelineNodeCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_delete_relationship(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteTimelineRelationshipCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_timeline_relationship(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_create_relationship(
    app: tauri::AppHandle,
    command: command_service::CreateTimelineRelationshipRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_timeline_relationship(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_apply_children(
    app: tauri::AppHandle,
    command: command_service::ApplyTimelineChildrenRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::apply_timeline_children(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_split_node(
    app: tauri::AppHandle,
    command: command_service::SplitTimelineNodeRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::split_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_playhead(
    app: tauri::AppHandle,
    position_ms: u64,
) -> Result<TimelinePlayheadCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    let projection = projection_service::timeline_render_projection(&state)
        .await
        .map_err(CommandError::from)?;
    let position_ms = clamp_timeline_playhead(position_ms, projection.payload.total_duration_ms);
    state.set_timeline_playhead(position_ms);
    Ok(TimelinePlayheadCommandResponse { position_ms })
}

fn clamp_timeline_playhead(position_ms: u64, total_duration_ms: u64) -> u64 {
    position_ms.min(total_duration_ms)
}

#[cfg(test)]
mod tests {
    use super::TimelinePlayheadCommandResponse;

    #[test]
    fn timeline_playhead_command_response_serializes_stable_payload() {
        let value = serde_json::to_value(TimelinePlayheadCommandResponse {
            position_ms: 42_500,
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "position_ms": 42500 }));
    }

    #[test]
    fn timeline_playhead_command_clamps_to_projection_duration() {
        assert_eq!(super::clamp_timeline_playhead(42_500, 120_000), 42_500);
        assert_eq!(super::clamp_timeline_playhead(240_000, 120_000), 120_000);
    }
}
