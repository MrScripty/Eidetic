use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use eidetic_core::timeline::node::NodeId;
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::{AppState, ServerEvent};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tokio::sync::broadcast;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;

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
                    if let Some(request) = graph_projection_request_for_event(&event) {
                        refresh_graph_renderer_projection(&app, &state, request).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("graph renderer projection bridge skipped {skipped} events");
                    refresh_graph_renderer_projection(&app, &state, Default::default()).await;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn graph_projection_request_for_event(
    event: &ServerEvent,
) -> Option<BibleRenderGraphProjectionRequest> {
    matches!(
        event,
        ServerEvent::BibleChanged
            | ServerEvent::HierarchyChanged
            | ServerEvent::StoryChanged
            | ServerEvent::TimelineChanged
            | ServerEvent::SemanticProposalsChanged
            | ServerEvent::ContextInfluenceChanged { .. }
    )
    .then(|| match event {
        ServerEvent::ContextInfluenceChanged { target_node_id } => {
            BibleRenderGraphProjectionRequest {
                selected_timeline_node_id: Some(NodeId(*target_node_id)),
                ..BibleRenderGraphProjectionRequest::default()
            }
        }
        _ => BibleRenderGraphProjectionRequest::default(),
    })
}

async fn refresh_graph_renderer_projection(
    app: &tauri::AppHandle,
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) {
    if !is_graph_renderer_open(app) {
        return;
    }

    let envelope =
        match bible_render_graph_projection::bible_render_graph_projection(state, request).await {
            Ok(envelope) => envelope,
            Err(error) => {
                tracing::warn!("failed to refresh graph renderer projection: {error}");
                return;
            }
        };

    if let Some(graph_owner) = app.try_state::<DesktopBibleGraphRendererOwner>()
        && let Err(error) = graph_owner.set_projection(envelope.payload)
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
    use super::{DesktopServerEvent, SERVER_EVENT_TOPIC, graph_projection_request_for_event};
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
        assert!(graph_projection_request_for_event(&ServerEvent::BibleChanged).is_some());
        assert!(graph_projection_request_for_event(&ServerEvent::TimelineChanged).is_some());
        assert!(
            graph_projection_request_for_event(&ServerEvent::SemanticProposalsChanged).is_some()
        );
        assert!(
            graph_projection_request_for_event(&ServerEvent::ContextInfluenceChanged {
                target_node_id: uuid::Uuid::nil(),
            })
            .is_some()
        );
        assert!(
            graph_projection_request_for_event(&ServerEvent::GenerationProgress {
                node_id: uuid::Uuid::nil(),
                token: "draft".to_string(),
                tokens_generated: 1,
            })
            .is_none()
        );
        assert!(graph_projection_request_for_event(&ServerEvent::ScriptChanged).is_none());
    }

    #[test]
    fn graph_projection_bridge_scopes_context_influence_refreshes() {
        let target_node_id = uuid::uuid!("f4bb67a9-c87e-40c8-a9a1-99e5a9bfaf24");
        let request = graph_projection_request_for_event(&ServerEvent::ContextInfluenceChanged {
            target_node_id,
        })
        .unwrap();

        assert_eq!(request.selected_timeline_node_id.unwrap().0, target_node_id);
        assert_eq!(request.neighborhood_depth, 1);
        assert_eq!(request.max_nodes, 200);
    }
}
