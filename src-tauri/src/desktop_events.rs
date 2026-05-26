use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
use eidetic_bevy_timeline::TimelineRendererCommand;
use eidetic_core::contracts::{
    CommandEnvelope, CreateTimelineNodeCommand, DeleteTimelineNodeCommand,
    SetTimelineNodeRangeCommand, SplitTimelineNodeCommand,
};
use eidetic_server::command_service;
use eidetic_server::state::{AppState, ServerEvent};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::broadcast;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
use crate::bevy_timeline_host::DesktopTimelineRendererOwner;
use crate::graph_renderer_projection::refresh_active_graph_renderer_projection;

pub const SERVER_EVENT_TOPIC: &str = "eidetic://server-event";

#[derive(Clone, Debug, Serialize)]
pub struct DesktopServerEvent {
    event: DesktopServerEventPayload,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum DesktopServerEventPayload {
    Backend(ServerEvent),
    GraphRendererCommand(BibleGraphRendererCommand),
    TimelineRendererFocus(TimelineRendererFocusEvent),
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TimelineRendererFocusEvent {
    SelectTimelineNode {
        node_id: eidetic_core::timeline::node::NodeId,
    },
}

pub struct DesktopEventBridgeOwner {
    handles: Mutex<Vec<tauri::async_runtime::JoinHandle<()>>>,
}

impl DesktopEventBridgeOwner {
    pub fn spawn(app: tauri::AppHandle, state: &AppState) -> Self {
        Self {
            handles: Mutex::new(vec![
                spawn_server_event_bridge(app.clone(), state),
                spawn_graph_renderer_projection_bridge(app.clone(), state),
                spawn_graph_renderer_command_bridge(app.clone()),
                spawn_timeline_renderer_command_bridge(app, state.clone()),
            ]),
        }
    }

    pub fn stop(&self) {
        if let Ok(mut handles) = self.handles.lock() {
            for handle in handles.drain(..) {
                handle.abort();
            }
        }
    }
}

impl Drop for DesktopEventBridgeOwner {
    fn drop(&mut self) {
        self.stop();
    }
}

fn spawn_server_event_bridge(
    app: tauri::AppHandle,
    state: &AppState,
) -> tauri::async_runtime::JoinHandle<()> {
    let mut events = state.events_tx.subscribe();

    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let Err(error) = app.emit(
                        SERVER_EVENT_TOPIC,
                        DesktopServerEvent {
                            event: DesktopServerEventPayload::Backend(event),
                        },
                    ) {
                        tracing::warn!("failed to emit desktop server event: {error}");
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("desktop server event bridge skipped {skipped} events");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn spawn_graph_renderer_projection_bridge(
    app: tauri::AppHandle,
    state: &AppState,
) -> tauri::async_runtime::JoinHandle<()> {
    let mut events = state.events_tx.subscribe();

    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if should_refresh_graph_renderer_projection(&event) {
                        refresh_graph_renderer_projection(&app).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("graph renderer projection bridge skipped {skipped} events");
                    refresh_graph_renderer_projection(&app).await;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn spawn_graph_renderer_command_bridge(
    app: tauri::AppHandle,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            let Some(owner) = app.try_state::<DesktopBibleGraphRendererOwner>() else {
                continue;
            };
            let commands = match owner.drain_commands() {
                Ok(commands) => commands,
                Err(error) => {
                    tracing::warn!("failed to drain graph renderer commands: {error:?}");
                    continue;
                }
            };

            for command in commands {
                if let Err(error) = app.emit(
                    SERVER_EVENT_TOPIC,
                    DesktopServerEvent {
                        event: DesktopServerEventPayload::GraphRendererCommand(command),
                    },
                ) {
                    tracing::warn!("failed to emit graph renderer command: {error}");
                }
            }
        }
    })
}

fn spawn_timeline_renderer_command_bridge(
    app: tauri::AppHandle,
    state: AppState,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            let Some(owner) = app.try_state::<DesktopTimelineRendererOwner>() else {
                continue;
            };
            let commands = match owner.drain_commands() {
                Ok(commands) => commands,
                Err(error) => {
                    tracing::warn!("failed to drain timeline renderer commands: {error:?}");
                    continue;
                }
            };

            for command in commands {
                if let Err(error) =
                    handle_timeline_renderer_command(&app, &state, command.clone()).await
                {
                    tracing::warn!(
                        "failed to apply timeline renderer command {command:?}: {error:?}"
                    );
                }
            }
        }
    })
}

async fn handle_timeline_renderer_command(
    app: &tauri::AppHandle,
    state: &AppState,
    command: TimelineRendererCommand,
) -> Result<(), String> {
    if let TimelineRendererCommand::SelectNode { node_id } = command {
        return app
            .emit(
                SERVER_EVENT_TOPIC,
                DesktopServerEvent {
                    event: DesktopServerEventPayload::TimelineRendererFocus(
                        TimelineRendererFocusEvent::SelectTimelineNode { node_id },
                    ),
                },
            )
            .map_err(|error| error.to_string());
    }

    apply_timeline_renderer_command(state, command).await
}

async fn apply_timeline_renderer_command(
    state: &AppState,
    command: TimelineRendererCommand,
) -> Result<(), String> {
    match timeline_renderer_mutation_command(command) {
        Some(TimelineRendererMutationCommand::SetNodeRange(command)) => {
            command_service::set_timeline_node_range(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::DeleteNode(command)) => {
            command_service::delete_timeline_node(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::CreateNode(command)) => {
            command_service::create_timeline_node_from_core_command(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        Some(TimelineRendererMutationCommand::SplitNode(command)) => {
            command_service::split_timeline_node_from_core_command(state, command)
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        None => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TimelineRendererMutationCommand {
    SetNodeRange(CommandEnvelope<SetTimelineNodeRangeCommand>),
    DeleteNode(CommandEnvelope<DeleteTimelineNodeCommand>),
    CreateNode(CommandEnvelope<CreateTimelineNodeCommand>),
    SplitNode(CommandEnvelope<SplitTimelineNodeCommand>),
}

fn timeline_renderer_mutation_command(
    command: TimelineRendererCommand,
) -> Option<TimelineRendererMutationCommand> {
    match command {
        TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms,
            end_ms,
        } => Some(TimelineRendererMutationCommand::SetNodeRange(
            CommandEnvelope::new(SetTimelineNodeRangeCommand {
                node_id,
                start_ms,
                end_ms,
            }),
        )),
        TimelineRendererCommand::DeleteNode { node_id } => {
            Some(TimelineRendererMutationCommand::DeleteNode(
                CommandEnvelope::new(DeleteTimelineNodeCommand { node_id }),
            ))
        }
        TimelineRendererCommand::CreateNode {
            node_id,
            parent_id,
            level,
            name,
            start_ms,
            end_ms,
            beat_type,
        } => Some(TimelineRendererMutationCommand::CreateNode(
            CommandEnvelope::new(CreateTimelineNodeCommand {
                node_id,
                parent_id,
                level,
                name,
                start_ms,
                end_ms,
                beat_type,
            }),
        )),
        TimelineRendererCommand::SplitNode {
            node_id,
            at_ms,
            left_node_id,
            right_node_id,
        } => Some(TimelineRendererMutationCommand::SplitNode(
            CommandEnvelope::new(SplitTimelineNodeCommand {
                node_id,
                at_ms,
                left_node_id,
                right_node_id,
            }),
        )),
        TimelineRendererCommand::SelectNode { .. } => None,
    }
}

fn should_refresh_graph_renderer_projection(event: &ServerEvent) -> bool {
    matches!(
        event,
        ServerEvent::BibleChanged
            | ServerEvent::HierarchyChanged
            | ServerEvent::StoryChanged
            | ServerEvent::TimelineChanged
            | ServerEvent::SemanticProposalsChanged
            | ServerEvent::ContextInfluenceChanged { .. }
    )
}

async fn refresh_graph_renderer_projection(app: &tauri::AppHandle) {
    if let Err(error) = refresh_active_graph_renderer_projection(app).await {
        tracing::warn!("failed to update graph renderer projection: {error:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopEventBridgeOwner, DesktopServerEvent, DesktopServerEventPayload, SERVER_EVENT_TOPIC,
        TimelineRendererFocusEvent, TimelineRendererMutationCommand,
        should_refresh_graph_renderer_projection, timeline_renderer_mutation_command,
    };
    use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
    use eidetic_bevy_timeline::TimelineRendererCommand;
    use eidetic_core::contracts::{BibleGraphNodeId, DeleteTimelineNodeCommand};
    use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
    use eidetic_server::state::ServerEvent;
    use serde_json::json;
    use std::sync::Mutex;

    #[test]
    fn serializes_backend_events_inside_stable_desktop_payload() {
        let event = ServerEvent::TimelineChanged;
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::Backend(event),
        })
        .unwrap();

        assert_eq!(SERVER_EVENT_TOPIC, "eidetic://server-event");
        assert_eq!(value, json!({ "event": { "type": "timeline_changed" } }));
    }

    #[test]
    fn preserves_event_fields_in_desktop_payload() {
        let node_id = uuid::uuid!("2f7f8d6d-7ce1-493f-90cc-5c79ab761eb5");
        let event = ServerEvent::GenerationProgress {
            node_id,
            token: "hello".into(),
            tokens_generated: 3,
        };
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::Backend(event),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "generation_progress",
                    "node_id": node_id.to_string(),
                    "token": "hello",
                    "tokens_generated": 3
                }
            })
        );
    }

    #[test]
    fn serializes_graph_renderer_commands_inside_stable_desktop_payload() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(
                BibleGraphRendererCommand::SelectNode { node_id },
            ),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "select_node",
                    "node_id": "node.character.ada"
                }
            })
        );
    }

    #[test]
    fn serializes_graph_renderer_navigation_commands_inside_stable_desktop_payload() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(
                BibleGraphRendererCommand::NavigateToNode { node_id },
            ),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "navigate_to_node",
                    "node_id": "node.character.ada"
                }
            })
        );
    }

    #[test]
    fn serializes_graph_renderer_clear_selection_inside_stable_desktop_payload() {
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(
                BibleGraphRendererCommand::ClearSelection,
            ),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "clear_selection"
                }
            })
        );
    }

    #[test]
    fn serializes_timeline_renderer_focus_events_with_distinct_type() {
        let node_id = NodeId::new();
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::TimelineRendererFocus(
                TimelineRendererFocusEvent::SelectTimelineNode { node_id },
            ),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "select_timeline_node",
                    "node_id": node_id.0.to_string()
                }
            })
        );
    }

    #[test]
    fn graph_projection_bridge_refreshes_only_structural_events() {
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::BibleChanged
        ));
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::TimelineChanged
        ));
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::SemanticProposalsChanged
        ));
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::ContextInfluenceChanged {
                target_node_id: uuid::Uuid::nil(),
            }
        ));
        assert!(!should_refresh_graph_renderer_projection(
            &ServerEvent::GenerationProgress {
                node_id: uuid::Uuid::nil(),
                token: "draft".to_string(),
                tokens_generated: 1,
            }
        ));
        assert!(!should_refresh_graph_renderer_projection(
            &ServerEvent::ScriptChanged
        ));
    }

    #[test]
    fn maps_timeline_renderer_range_commands_to_backend_commands() {
        let node_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::SetNodeRange(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::SetNodeRange {
                node_id,
                start_ms: 1_000,
                end_ms: 2_000,
            })
        else {
            panic!("expected set range mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.start_ms, 1_000);
        assert_eq!(command.payload.end_ms, 2_000);
    }

    #[test]
    fn maps_timeline_renderer_delete_commands_to_backend_commands() {
        let node_id = NodeId::new();

        assert!(matches!(
            timeline_renderer_mutation_command(TimelineRendererCommand::DeleteNode { node_id }),
            Some(TimelineRendererMutationCommand::DeleteNode(command))
                if command.payload == DeleteTimelineNodeCommand { node_id }
        ));
    }

    #[test]
    fn maps_timeline_renderer_create_commands_to_backend_commands() {
        let node_id = NodeId::new();
        let parent_id = Some(NodeId::new());

        let Some(TimelineRendererMutationCommand::CreateNode(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::CreateNode {
                node_id,
                parent_id,
                level: StoryLevel::Beat,
                name: "Argument escalates".to_string(),
                start_ms: 3_000,
                end_ms: 5_000,
                beat_type: Some(BeatType::Escalation),
            })
        else {
            panic!("expected create mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.parent_id, parent_id);
        assert_eq!(command.payload.level, StoryLevel::Beat);
        assert_eq!(command.payload.name, "Argument escalates");
        assert_eq!(command.payload.start_ms, 3_000);
        assert_eq!(command.payload.end_ms, 5_000);
        assert_eq!(command.payload.beat_type, Some(BeatType::Escalation));
    }

    #[test]
    fn maps_timeline_renderer_split_commands_to_backend_commands() {
        let node_id = NodeId::new();
        let left_node_id = NodeId::new();
        let right_node_id = NodeId::new();

        let Some(TimelineRendererMutationCommand::SplitNode(command)) =
            timeline_renderer_mutation_command(TimelineRendererCommand::SplitNode {
                node_id,
                at_ms: 4_000,
                left_node_id,
                right_node_id,
            })
        else {
            panic!("expected split mutation command");
        };

        assert_eq!(command.payload.node_id, node_id);
        assert_eq!(command.payload.at_ms, 4_000);
        assert_eq!(command.payload.left_node_id, left_node_id);
        assert_eq!(command.payload.right_node_id, right_node_id);
    }

    #[test]
    fn ignores_timeline_renderer_commands_without_backend_mutation() {
        let node_id = NodeId::new();

        assert_eq!(
            timeline_renderer_mutation_command(TimelineRendererCommand::SelectNode { node_id }),
            None
        );
    }

    #[test]
    fn desktop_event_bridge_owner_stop_is_idempotent_without_handles() {
        let owner = DesktopEventBridgeOwner {
            handles: Mutex::new(Vec::new()),
        };

        owner.stop();
        owner.stop();
    }
}
