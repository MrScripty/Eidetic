use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
use eidetic_core::contracts::{CommandEnvelope, DeleteBibleGraphNodeCommand};
use eidetic_server::command_service;
use eidetic_server::projection_service;
use eidetic_server::state::{AppState, ServerEvent};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::broadcast;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
use crate::bevy_timeline_host::DesktopTimelineRendererOwner;
use crate::graph_renderer_projection::{
    refresh_active_graph_renderer_projection, update_active_graph_renderer_selected_node,
};
use crate::timeline_renderer_command_bridge::spawn_timeline_renderer_command_bridge;

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
                spawn_timeline_renderer_projection_bridge(app.clone(), state),
                spawn_graph_renderer_command_bridge(app.clone(), state.clone()),
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

fn spawn_timeline_renderer_projection_bridge(
    app: tauri::AppHandle,
    state: &AppState,
) -> tauri::async_runtime::JoinHandle<()> {
    let mut events = state.events_tx.subscribe();

    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if should_refresh_timeline_renderer_projection(&event) {
                        refresh_timeline_renderer_projection(&app).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("timeline renderer projection bridge skipped {skipped} events");
                    refresh_timeline_renderer_projection(&app).await;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn spawn_graph_renderer_command_bridge(
    app: tauri::AppHandle,
    state: AppState,
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
                let follow_up_command =
                    match handle_graph_renderer_command(&app, &state, command.clone()).await {
                        Ok(follow_up_command) => follow_up_command,
                        Err(error) => {
                            tracing::warn!(
                                "failed to apply graph renderer command {command:?}: {error}"
                            );
                            None
                        }
                    };
                if should_emit_graph_renderer_command(&command) {
                    emit_graph_renderer_command(&app, command);
                }
                if let Some(follow_up_command) = follow_up_command {
                    emit_graph_renderer_command(&app, follow_up_command);
                }
            }
        }
    })
}

async fn handle_graph_renderer_command(
    app: &tauri::AppHandle,
    state: &AppState,
    command: BibleGraphRendererCommand,
) -> Result<Option<BibleGraphRendererCommand>, String> {
    apply_graph_renderer_command_projection_update(app, &command)
        .await
        .map_err(|error| format!("{error:?}"))?;
    apply_graph_renderer_mutation_command(app, state, command).await
}

async fn apply_graph_renderer_command_projection_update(
    app: &tauri::AppHandle,
    command: &BibleGraphRendererCommand,
) -> Result<(), crate::error::CommandError> {
    if let Some(selected_node_id) = graph_renderer_command_selected_node_update(command) {
        update_active_graph_renderer_selected_node(app, selected_node_id).await?;
    }

    Ok(())
}

async fn apply_graph_renderer_mutation_command(
    app: &tauri::AppHandle,
    state: &AppState,
    command: BibleGraphRendererCommand,
) -> Result<Option<BibleGraphRendererCommand>, String> {
    match graph_renderer_mutation_command(command) {
        Some(GraphRendererMutationCommand::DeleteNode(command)) => {
            command_service::delete_bible_graph_node(state, command)
                .await
                .map_err(|error| error.to_string())?;
            update_active_graph_renderer_selected_node(app, None)
                .await
                .map_err(|error| format!("{error:?}"))?;
            Ok(Some(BibleGraphRendererCommand::ClearSelection))
        }
        Some(GraphRendererMutationCommand::CreateConnectedNode { parent_id }) => {
            let response = command_service::create_connected_bible_graph_node(state, parent_id)
                .await
                .map_err(|error| error.to_string())?;
            let node_id = response.node_id().clone();
            update_active_graph_renderer_selected_node(app, Some(node_id.clone()))
                .await
                .map_err(|error| format!("{error:?}"))?;
            Ok(Some(BibleGraphRendererCommand::SelectNode { node_id }))
        }
        None => Ok(None),
    }
}

fn emit_graph_renderer_command(app: &tauri::AppHandle, command: BibleGraphRendererCommand) {
    if let Err(error) = app.emit(
        SERVER_EVENT_TOPIC,
        DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(command),
        },
    ) {
        tracing::warn!("failed to emit graph renderer command: {error}");
    }
}

fn should_emit_graph_renderer_command(command: &BibleGraphRendererCommand) -> bool {
    !matches!(
        command,
        BibleGraphRendererCommand::DeleteNode { .. }
            | BibleGraphRendererCommand::CreateConnectedNode { .. }
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GraphRendererMutationCommand {
    DeleteNode(CommandEnvelope<DeleteBibleGraphNodeCommand>),
    CreateConnectedNode {
        parent_id: eidetic_core::contracts::BibleGraphNodeId,
    },
}

fn graph_renderer_mutation_command(
    command: BibleGraphRendererCommand,
) -> Option<GraphRendererMutationCommand> {
    match command {
        BibleGraphRendererCommand::DeleteNode { node_id } => {
            Some(GraphRendererMutationCommand::DeleteNode(
                CommandEnvelope::new(DeleteBibleGraphNodeCommand { node_id }),
            ))
        }
        BibleGraphRendererCommand::CreateConnectedNode { parent_id } => {
            Some(GraphRendererMutationCommand::CreateConnectedNode { parent_id })
        }
        BibleGraphRendererCommand::SelectNode { .. }
        | BibleGraphRendererCommand::SelectEdge { .. }
        | BibleGraphRendererCommand::SelectInfluence { .. }
        | BibleGraphRendererCommand::InspectNode { .. }
        | BibleGraphRendererCommand::FocusNode { .. }
        | BibleGraphRendererCommand::NavigateToNode { .. }
        | BibleGraphRendererCommand::ClearSelection => None,
    }
}

fn graph_renderer_command_selected_node_update(
    command: &BibleGraphRendererCommand,
) -> Option<Option<eidetic_core::contracts::BibleGraphNodeId>> {
    match command {
        BibleGraphRendererCommand::SelectNode { node_id } => Some(Some(node_id.clone())),
        BibleGraphRendererCommand::ClearSelection => Some(None),
        BibleGraphRendererCommand::SelectEdge { .. }
        | BibleGraphRendererCommand::SelectInfluence { .. }
        | BibleGraphRendererCommand::InspectNode { .. }
        | BibleGraphRendererCommand::FocusNode { .. }
        | BibleGraphRendererCommand::NavigateToNode { .. }
        | BibleGraphRendererCommand::DeleteNode { .. }
        | BibleGraphRendererCommand::CreateConnectedNode { .. } => None,
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
            | ServerEvent::TimelineSelectionChanged { .. }
            | ServerEvent::TimelinePlayheadChanged { .. }
            | ServerEvent::ContextInfluenceChanged { .. }
    )
}

fn should_refresh_timeline_renderer_projection(event: &ServerEvent) -> bool {
    matches!(
        event,
        ServerEvent::TimelineChanged
            | ServerEvent::HierarchyChanged
            | ServerEvent::ContextInfluenceChanged { .. }
            | ServerEvent::TimelineSelectionChanged { .. }
            | ServerEvent::TimelinePlayheadChanged { .. }
    )
}

async fn refresh_graph_renderer_projection(app: &tauri::AppHandle) {
    if let Err(error) = refresh_active_graph_renderer_projection(app).await {
        tracing::warn!("failed to update graph renderer projection: {error:?}");
    }
}

async fn refresh_timeline_renderer_projection(app: &tauri::AppHandle) {
    let Some(owner) = app.try_state::<DesktopTimelineRendererOwner>() else {
        return;
    };
    let status = match owner.status() {
        Ok(status) => status,
        Err(error) => {
            tracing::warn!("failed to read timeline renderer status: {error:?}");
            return;
        }
    };
    if !status.running {
        return;
    }

    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let state = state.inner().clone();
    let projection = match projection_service::timeline_render_projection(&state).await {
        Ok(projection) => projection,
        Err(error) => {
            tracing::warn!("failed to project timeline renderer refresh: {error:?}");
            return;
        }
    };
    if let Err(error) = owner.set_projection(projection.payload) {
        tracing::warn!("failed to refresh timeline renderer projection: {error:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopEventBridgeOwner, DesktopServerEvent, DesktopServerEventPayload,
        GraphRendererMutationCommand, SERVER_EVENT_TOPIC,
        graph_renderer_command_selected_node_update, graph_renderer_mutation_command,
        should_emit_graph_renderer_command, should_refresh_graph_renderer_projection,
        should_refresh_timeline_renderer_projection,
    };
    use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
    use eidetic_core::contracts::{
        BibleGraphEdgeId, BibleGraphNodeId, DeleteBibleGraphNodeCommand,
    };
    use eidetic_core::timeline::node::NodeId;
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
    fn graph_renderer_commands_project_native_node_selection() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();

        assert_eq!(
            graph_renderer_command_selected_node_update(&BibleGraphRendererCommand::SelectNode {
                node_id: node_id.clone()
            }),
            Some(Some(node_id.clone()))
        );
        assert_eq!(
            graph_renderer_command_selected_node_update(&BibleGraphRendererCommand::ClearSelection),
            Some(None)
        );
        assert_eq!(
            graph_renderer_command_selected_node_update(&BibleGraphRendererCommand::SelectEdge {
                edge_id: BibleGraphEdgeId::new("edge.character.ada.knows.character.grace").unwrap()
            }),
            None
        );
        assert_eq!(
            graph_renderer_command_selected_node_update(&BibleGraphRendererCommand::FocusNode {
                node_id
            }),
            None
        );
    }

    #[test]
    fn graph_renderer_mutations_are_backend_owned_and_not_forwarded_to_svelte() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let delete_command = BibleGraphRendererCommand::DeleteNode {
            node_id: node_id.clone(),
        };
        let create_command = BibleGraphRendererCommand::CreateConnectedNode {
            parent_id: node_id.clone(),
        };

        assert!(matches!(
            graph_renderer_mutation_command(delete_command.clone()),
            Some(GraphRendererMutationCommand::DeleteNode(command))
                if command.payload == DeleteBibleGraphNodeCommand {
                    node_id: node_id.clone()
                }
        ));
        assert!(matches!(
            graph_renderer_mutation_command(create_command.clone()),
            Some(GraphRendererMutationCommand::CreateConnectedNode { parent_id })
                if parent_id == node_id
        ));
        assert!(!should_emit_graph_renderer_command(&delete_command));
        assert!(!should_emit_graph_renderer_command(&create_command));
        assert!(should_emit_graph_renderer_command(
            &BibleGraphRendererCommand::SelectNode { node_id }
        ));
    }

    #[test]
    fn serializes_graph_renderer_graph_mutation_intents_inside_stable_desktop_payload() {
        let node_id = BibleGraphNodeId::new("node.character.ada").unwrap();
        let delete_value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(
                BibleGraphRendererCommand::DeleteNode {
                    node_id: node_id.clone(),
                },
            ),
        })
        .unwrap();
        let create_value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::GraphRendererCommand(
                BibleGraphRendererCommand::CreateConnectedNode { parent_id: node_id },
            ),
        })
        .unwrap();

        assert_eq!(
            delete_value,
            json!({
                "event": {
                    "type": "delete_node",
                    "node_id": "node.character.ada"
                }
            })
        );
        assert_eq!(
            create_value,
            json!({
                "event": {
                    "type": "create_connected_node",
                    "parent_id": "node.character.ada"
                }
            })
        );
    }

    #[test]
    fn serializes_backend_timeline_selection_events_inside_stable_desktop_payload() {
        let node_id = NodeId::new();
        let event = ServerEvent::TimelineSelectionChanged {
            node_id: Some(node_id),
        };
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::Backend(event),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "timeline_selection_changed",
                    "node_id": node_id.0.to_string()
                }
            })
        );
    }

    #[test]
    fn serializes_backend_timeline_playhead_events_inside_stable_desktop_payload() {
        let event = ServerEvent::TimelinePlayheadChanged {
            position_ms: 42_500,
        };
        let value = serde_json::to_value(DesktopServerEvent {
            event: DesktopServerEventPayload::Backend(event),
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "event": {
                    "type": "timeline_playhead_changed",
                    "position_ms": 42500
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
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::TimelineSelectionChanged {
                node_id: Some(NodeId::new()),
            }
        ));
        assert!(should_refresh_graph_renderer_projection(
            &ServerEvent::TimelinePlayheadChanged { position_ms: 1_000 }
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
    fn timeline_projection_bridge_refreshes_only_timeline_renderer_events() {
        assert!(should_refresh_timeline_renderer_projection(
            &ServerEvent::TimelineChanged
        ));
        assert!(should_refresh_timeline_renderer_projection(
            &ServerEvent::HierarchyChanged
        ));
        assert!(should_refresh_timeline_renderer_projection(
            &ServerEvent::ContextInfluenceChanged {
                target_node_id: uuid::Uuid::nil(),
            }
        ));
        assert!(should_refresh_timeline_renderer_projection(
            &ServerEvent::TimelineSelectionChanged {
                node_id: Some(NodeId::new()),
            }
        ));
        assert!(should_refresh_timeline_renderer_projection(
            &ServerEvent::TimelinePlayheadChanged { position_ms: 1_000 }
        ));
        assert!(!should_refresh_timeline_renderer_projection(
            &ServerEvent::BibleChanged
        ));
        assert!(!should_refresh_timeline_renderer_projection(
            &ServerEvent::ScriptChanged
        ));
        assert!(!should_refresh_timeline_renderer_projection(
            &ServerEvent::GenerationProgress {
                node_id: uuid::Uuid::nil(),
                token: "draft".to_string(),
                tokens_generated: 1,
            }
        ));
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
