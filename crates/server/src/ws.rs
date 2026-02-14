use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::StreamExt;

use crate::state::AppState;

/// WebSocket upgrade handler.
pub async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
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

    // Echo loop — Sprint 2 will replace this with real event dispatch.
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                // Parse and handle subscribe / beat_notes_edit / etc.
                // For now, echo back.
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
