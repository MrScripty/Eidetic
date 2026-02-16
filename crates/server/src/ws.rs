use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::StreamExt;
use tokio::sync::{broadcast, oneshot};
use yrs::updates::encoder::Encode;

use crate::state::AppState;
use crate::ydoc::DocCommand;

/// Monotonically increasing client ID counter.
static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

/// WebSocket upgrade handler.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, client_id: u64) {
    let mut event_rx = state.events_tx.subscribe();
    let mut doc_update_rx = state.doc_update_tx.subscribe();

    // Send a welcome message so clients know the connection is alive.
    if socket
        .send(Message::Text(
            serde_json::json!({ "type": "connected", "client_id": client_id })
                .to_string()
                .into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    // Y-sync handshake: send the full doc state to the new client so it
    // starts with a complete copy of the CRDT document.
    if let Some(state_vec) = crate::ydoc::get_state_vector(&state.doc_tx).await {
        // Send our state vector so the client can compute what it's missing.
        // For the initial sync, we just send the full state as an update.
        let empty_sv = yrs::StateVector::default();
        let full_state = crate::ydoc::get_diff(&state.doc_tx, empty_sv.encode_v1()).await;
        if let Some(data) = full_state {
            if socket.send(Message::Binary(data.into())).await.is_err() {
                return;
            }
        }
        // Also drop the state_vec we fetched (used for future incremental sync).
        let _ = state_vec;
    }

    loop {
        tokio::select! {
            // Forward broadcast events to the WebSocket client (text frames).
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client {client_id} lagged, skipped {n} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Forward Y.Doc updates to this client (binary frames).
            result = doc_update_rx.recv() => {
                match result {
                    Ok(update) => {
                        // Don't echo updates back to the client that sent them.
                        if update.origin_client != client_id {
                            if socket.send(Message::Binary(update.data.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client {client_id} lagged, skipped {n} doc updates — requesting full resync");
                        // On lag, send a full state update so the client recovers.
                        let empty_sv = yrs::StateVector::default();
                        if let Some(data) = crate::ydoc::get_diff(&state.doc_tx, empty_sv.encode_v1()).await {
                            if socket.send(Message::Binary(data.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Listen for client messages.
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        // CRDT update from client → send to doc manager.
                        let (reply_tx, reply_rx) = oneshot::channel();
                        let cmd = DocCommand::ApplyUpdate {
                            client_id,
                            update: data.to_vec(),
                            reply: reply_tx,
                        };
                        if state.doc_tx.send(cmd).await.is_err() {
                            tracing::error!("doc manager channel closed");
                            break;
                        }
                        // Wait for acknowledgment (ensures ordering).
                        match reply_rx.await {
                            Ok(Ok(())) => {}
                            Ok(Err(e)) => {
                                tracing::warn!("client {client_id} sent invalid CRDT update: {e}");
                            }
                            Err(_) => {
                                tracing::error!("doc manager dropped reply");
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Text(_))) => {
                        // Text messages from client (awareness, subscribe, etc.)
                        // — acknowledged but not filtered yet.
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {} // Ping/Pong handled by axum
                    Some(Err(_)) => break,
                }
            }
        }
    }
}
