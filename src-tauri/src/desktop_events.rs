use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::{AppState, ServerEvent};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::broadcast;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
use crate::graph_renderer_commands::GraphRendererProjectionRequestState;

pub const SERVER_EVENT_TOPIC: &str = "eidetic://server-event";

#[derive(Clone, Debug, Serialize)]
pub struct DesktopServerEvent {
    event: ServerEvent,
}

pub fn spawn_server_event_bridge(app: tauri::AppHandle, state: &AppState) {
    let mut events = state.events_tx.subscribe();

    let _event_bridge = tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let Err(error) = app.emit(SERVER_EVENT_TOPIC, DesktopServerEvent { event }) {
                        tracing::warn!("failed to emit desktop server event: {error}");
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("desktop server event bridge skipped {skipped} events");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

pub fn spawn_graph_renderer_projection_bridge(app: tauri::AppHandle, state: &AppState) {
    let mut events = state.events_tx.subscribe();
    let state = state.clone();

    let _projection_bridge = tauri::async_runtime::spawn(async move {
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
    });
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
    if !is_graph_renderer_open(app) {
        return;
    }

    let request = app
        .try_state::<GraphRendererProjectionRequestState>()
        .map(|request_state| request_state.current())
        .unwrap_or_default();
    let envelope =
        match bible_render_graph_projection::bible_render_graph_projection(state, request).await {
            Ok(envelope) => envelope,
            Err(error) => {
                tracing::warn!("failed to refresh graph renderer projection: {error}");
                return;
            }
        };

    if let Some(graph_owner) = app.try_state::<DesktopBibleGraphRendererOwner>()
        && let Err(error) = graph_owner.update_projection_if_open(envelope.payload)
    {
        tracing::warn!("failed to update graph renderer projection: {error:?}");
    }
}

fn is_graph_renderer_open(app: &tauri::AppHandle) -> bool {
    app.try_state::<DesktopBibleGraphRendererOwner>()
        .and_then(|graph_owner| graph_owner.status().ok())
        .map(|status| status.renderer_window_open)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{DesktopServerEvent, SERVER_EVENT_TOPIC, should_refresh_graph_renderer_projection};
    use eidetic_server::state::ServerEvent;
    use serde_json::json;

    #[test]
    fn serializes_backend_events_inside_stable_desktop_payload() {
        let event = ServerEvent::TimelineChanged;
        let value = serde_json::to_value(DesktopServerEvent { event }).unwrap();

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
        let value = serde_json::to_value(DesktopServerEvent { event }).unwrap();

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
}
