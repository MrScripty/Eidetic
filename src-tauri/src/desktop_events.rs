use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
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
use crate::graph_renderer_projection::refresh_active_graph_renderer_projection;
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
        DesktopEventBridgeOwner, DesktopServerEvent, DesktopServerEventPayload, SERVER_EVENT_TOPIC,
        should_refresh_graph_renderer_projection, should_refresh_timeline_renderer_projection,
    };
    use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
    use eidetic_core::contracts::BibleGraphNodeId;
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
