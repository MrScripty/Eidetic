use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
use eidetic_server::state::{AppState, ServerEvent};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::broadcast;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
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
                spawn_graph_renderer_command_bridge(app),
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
    let state = state.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if should_refresh_graph_renderer_projection(&event) {
                        refresh_graph_renderer_projection(&app, &state).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("graph renderer projection bridge skipped {skipped} events");
                    refresh_graph_renderer_projection(&app, &state).await;
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
            | ServerEvent::ContextInfluenceChanged { .. }
    )
}

async fn refresh_graph_renderer_projection(app: &tauri::AppHandle, state: &AppState) {
    if let Err(error) = refresh_active_graph_renderer_projection(app, state).await {
        tracing::warn!("failed to update graph renderer projection: {error:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopEventBridgeOwner, DesktopServerEvent, DesktopServerEventPayload, SERVER_EVENT_TOPIC,
        should_refresh_graph_renderer_projection,
    };
    use eidetic_bevy_bible_graph::BibleGraphRendererCommand;
    use eidetic_core::contracts::BibleGraphNodeId;
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
    fn desktop_event_bridge_owner_stop_is_idempotent_without_handles() {
        let owner = DesktopEventBridgeOwner {
            handles: Mutex::new(Vec::new()),
        };

        owner.stop();
        owner.stop();
    }
}
