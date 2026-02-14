use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::StreamExt;
use tokio::sync::broadcast;

use crate::state::{AppState, ServerEvent};

/// WebSocket upgrade handler.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.events_tx.subscribe();
    ws.on_upgrade(|socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<ServerEvent>) {
    // Send a welcome message so clients know the connection is alive.
    if socket
        .send(Message::Text(
            serde_json::json!({ "type": "connected" }).to_string().into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            // Forward broadcast events to the WebSocket client.
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, skipped {n} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Listen for client messages (subscribe, etc.).
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {
                        // Client messages (subscribe, etc.) — acknowledged but not
                        // filtered yet. All events are forwarded to all clients.
                    }
                    Some(Err(_)) => break,
                }
            }
        }
    }
}
