//! Y.Doc manager: single-owner async task for CRDT content.
//!
//! All text content (notes, scripts, outlines) lives in a Yrs `Doc`.
//! This module provides a channel-based interface so that concurrent
//! callers (HTTP handlers, WebSocket clients, AI generation) never
//! contend on the Doc directly — they send [`DocCommand`] messages
//! and optionally await a reply via a oneshot channel.
//!
//! ## Y.Doc Schema
//!
//! ```text
//! Y.Doc
//! ├── Y.Map("nodes")                 // keyed by node UUID string
//! │   └── {node_id}: Y.Map
//! │       ├── "notes":   Y.Text      // planning notes (with author attrs)
//! │       └── "content": Y.Text      // script/outline (with author attrs)
//! └── Y.Map("project_text")          // project-level text
//!     └── "premise": Y.Text
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing;
use yrs::types::Attrs;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Doc, GetString, Map, MapPrelim, MapRef, Options, ReadTxn, Text, TextPrelim, TextRef,
    Transact, Update, WriteTxn,
};

use eidetic_core::timeline::node::NodeId;

// ──────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────

/// Which text field within a node to read/write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentField {
    Notes,
    Content,
}

/// A snapshot of a node's text content read from Y.Doc.
#[derive(Debug, Clone)]
pub struct NodeTextSnapshot {
    pub notes: String,
    pub content: String,
    pub attributed_spans: Vec<AttributedSpan>,
}

/// A contiguous span of text with a single author.
#[derive(Debug, Clone)]
pub struct AttributedSpan {
    pub text: String,
    /// e.g. "human:{user_id}" or "ai:{generation_id}"
    pub author: String,
    pub start: usize,
    pub end: usize,
}

/// A binary update to broadcast to WebSocket clients.
#[derive(Debug, Clone)]
pub struct DocUpdate {
    /// Client that originated this update (0 = server/AI).
    pub origin_client: u64,
    /// Encoded Yrs update bytes (v1 encoding).
    pub data: Vec<u8>,
}

/// Notification that a human edited content — fed to the AI reactor.
#[derive(Debug, Clone)]
pub struct ContentChange {
    pub node_id: NodeId,
    pub field: ContentField,
    pub changed_by: String,
}

/// Commands sent to the Y.Doc manager task via channel.
pub enum DocCommand {
    /// Apply a binary update from a WebSocket client.
    ApplyUpdate {
        client_id: u64,
        update: Vec<u8>,
        reply: oneshot::Sender<Result<(), String>>,
    },
    /// Get the full state vector for sync handshake.
    GetStateVector {
        reply: oneshot::Sender<Vec<u8>>,
    },
    /// Get a diff from a given state vector (what the caller is missing).
    GetDiff {
        sv: Vec<u8>,
        reply: oneshot::Sender<Vec<u8>>,
    },
    /// Write text content for a node (used by AI generation, apply_children, etc.)
    WriteNodeContent {
        node_id: NodeId,
        field: ContentField,
        text: String,
        author: String,
    },
    /// Read text content for a node.
    ReadNodeContent {
        node_id: NodeId,
        reply: oneshot::Sender<NodeTextSnapshot>,
    },
    /// Read text content for all nodes (used during save).
    ReadAllNodes {
        reply: oneshot::Sender<HashMap<String, NodeTextSnapshot>>,
    },
    /// Ensure a node entry exists in Y.Doc (called when node created via REST).
    EnsureNode {
        node_id: NodeId,
    },
    /// Remove a node entry from Y.Doc (called when node deleted via REST).
    RemoveNode {
        node_id: NodeId,
    },
    /// Serialize full doc state for persistence.
    Serialize {
        reply: oneshot::Sender<Vec<u8>>,
    },
    /// Load doc state from persistence (replaces current doc content).
    Load {
        state: Vec<u8>,
        reply: oneshot::Sender<Result<(), String>>,
    },
    /// Flush AI token buffer into Y.Doc (appends to content field).
    FlushTokens {
        node_id: NodeId,
        tokens: String,
        author: String,
    },
    /// Replace a character range in a Y.Text field (used by diffusion infilling).
    ///
    /// Removes `[start..end)` and inserts `new_text` at `start` with author
    /// attribution. Clamps indices to the current Y.Text length.
    RewriteRegion {
        node_id: NodeId,
        field: ContentField,
        start: usize,
        end: usize,
        new_text: String,
        author: String,
    },
    /// Shutdown the manager task.
    Shutdown,
}

/// Channel capacity for the doc command queue.
pub const DOC_CHANNEL_CAPACITY: usize = 256;

/// Channel capacity for the content change queue (AI reactor feed).
pub const CHANGE_CHANNEL_CAPACITY: usize = 256;

/// Channel capacity for the doc update broadcast (WebSocket feed).
pub const UPDATE_BROADCAST_CAPACITY: usize = 256;

// ──────────────────────────────────────────────
// Doc manager task
// ──────────────────────────────────────────────

/// Spawn the doc manager and return the command sender.
///
/// Also returns the broadcast sender for doc updates (WebSocket clients
/// subscribe to this) and the mpsc sender for content changes (the AI
/// reactor consumes this).
pub fn spawn_doc_manager() -> (
    mpsc::Sender<DocCommand>,
    broadcast::Sender<DocUpdate>,
    mpsc::Receiver<ContentChange>,
) {
    let (cmd_tx, cmd_rx) = mpsc::channel(DOC_CHANNEL_CAPACITY);
    let (update_tx, _) = broadcast::channel(UPDATE_BROADCAST_CAPACITY);
    let (change_tx, change_rx) = mpsc::channel(CHANGE_CHANNEL_CAPACITY);

    let update_tx_clone = update_tx.clone();
    tokio::spawn(run_doc_manager(cmd_rx, update_tx_clone, change_tx));

    (cmd_tx, update_tx, change_rx)
}

/// The main loop of the Y.Doc manager task.
///
/// Owns the `Doc` exclusively — no external code touches it.
/// All interaction is via [`DocCommand`] messages.
async fn run_doc_manager(
    mut rx: mpsc::Receiver<DocCommand>,
    update_tx: broadcast::Sender<DocUpdate>,
    change_tx: mpsc::Sender<ContentChange>,
) {
    // Use client_id = 0 for the server's own doc.
    let doc = Doc::with_options(Options {
        client_id: 0,
        skip_gc: false,
        ..Options::default()
    });

    // Pre-create the root maps so they exist for all operations.
    {
        let mut txn = doc.transact_mut();
        let _ = txn.get_or_insert_map("nodes");
        let _ = txn.get_or_insert_map("project_text");
    }

    // Subscribe to doc updates for broadcasting to WebSocket clients.
    // We capture updates via observe_update_v1 and forward them.
    // However, since the doc is owned by this task and we process commands
    // sequentially, we track which command triggered the update to set the
    // correct origin_client.
    let pending_origin = Arc::new(std::sync::Mutex::new(0u64));
    let pending_origin_clone = pending_origin.clone();
    let update_tx_clone = update_tx.clone();

    let _update_sub = doc
        .observe_update_v1(move |_txn, event| {
            let origin = *pending_origin_clone.lock().unwrap();
            let _ = update_tx_clone.send(DocUpdate {
                origin_client: origin,
                data: event.update.clone(),
            });
        })
        .expect("failed to subscribe to doc updates");

    tracing::info!("Y.Doc manager started");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            DocCommand::ApplyUpdate {
                client_id,
                update,
                reply,
            } => {
                *pending_origin.lock().unwrap() = client_id;
                let result = apply_client_update(&doc, &update, client_id, &change_tx).await;
                *pending_origin.lock().unwrap() = 0;
                let _ = reply.send(result);
            }

            DocCommand::GetStateVector { reply } => {
                let txn = doc.transact();
                let sv = txn.state_vector().encode_v1();
                let _ = reply.send(sv);
            }

            DocCommand::GetDiff { sv, reply } => {
                match yrs::StateVector::decode_v1(&sv) {
                    Ok(state_vector) => {
                        let txn = doc.transact();
                        let diff = txn.encode_state_as_update_v1(&state_vector);
                        let _ = reply.send(diff);
                    }
                    Err(e) => {
                        tracing::error!("failed to decode state vector: {e}");
                        // Send empty update on error.
                        let _ = reply.send(Vec::new());
                    }
                }
            }

            DocCommand::WriteNodeContent {
                node_id,
                field,
                text,
                author,
            } => {
                // AI/server writes: set origin to 0, don't feed change_tx.
                *pending_origin.lock().unwrap() = 0;
                write_node_field(&doc, &node_id, field, &text, &author);
            }

            DocCommand::ReadNodeContent { node_id, reply } => {
                let snapshot = read_node_snapshot(&doc, &node_id);
                let _ = reply.send(snapshot);
            }

            DocCommand::ReadAllNodes { reply } => {
                let snapshots = read_all_node_snapshots(&doc);
                let _ = reply.send(snapshots);
            }

            DocCommand::EnsureNode { node_id } => {
                ensure_node_exists(&doc, &node_id);
            }

            DocCommand::RemoveNode { node_id } => {
                remove_node(&doc, &node_id);
            }

            DocCommand::Serialize { reply } => {
                let txn = doc.transact();
                let state =
                    txn.encode_state_as_update_v1(&yrs::StateVector::default());
                let _ = reply.send(state);
            }

            DocCommand::Load { state, reply } => {
                let result = load_doc_state(&doc, &state);
                let _ = reply.send(result);
            }

            DocCommand::FlushTokens {
                node_id,
                tokens,
                author,
            } => {
                // AI token flush: append to content field.
                *pending_origin.lock().unwrap() = 0;
                append_to_node_field(&doc, &node_id, ContentField::Content, &tokens, &author);
            }

            DocCommand::RewriteRegion {
                node_id,
                field,
                start,
                end,
                new_text,
                author,
            } => {
                *pending_origin.lock().unwrap() = 0;
                rewrite_region(&doc, &node_id, field, start, end, &new_text, &author);
            }

            DocCommand::Shutdown => {
                tracing::info!("Y.Doc manager shutting down");
                break;
            }
        }
    }
}

// ──────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────

/// Get or create the Y.Map for a node inside the "nodes" root map.
fn get_or_create_node_map(
    nodes_map: &MapRef,
    txn: &mut yrs::TransactionMut<'_>,
    node_id: &str,
) -> MapRef {
    if let Some(yrs::Out::YMap(existing)) = nodes_map.get(txn, node_id) {
        existing
    } else {
        nodes_map.insert(txn, node_id, MapPrelim::default())
    }
}

/// Get or create a Y.Text field within a node map.
fn get_or_create_text_field(
    node_map: &MapRef,
    txn: &mut yrs::TransactionMut<'_>,
    field_name: &str,
) -> TextRef {
    if let Some(yrs::Out::YText(existing)) = node_map.get(txn, field_name) {
        existing
    } else {
        node_map.insert(txn, field_name, TextPrelim::new(""))
    }
}

/// Ensure a node entry exists in the Y.Doc (both notes and content Y.Text).
fn ensure_node_exists(doc: &Doc, node_id: &NodeId) {
    let node_key = node_id.0.to_string();
    let mut txn = doc.transact_mut();
    let nodes = txn.get_or_insert_map("nodes");
    let node_map = get_or_create_node_map(&nodes, &mut txn, &node_key);
    let _ = get_or_create_text_field(&node_map, &mut txn, "notes");
    let _ = get_or_create_text_field(&node_map, &mut txn, "content");
}

/// Remove a node entry from the Y.Doc.
fn remove_node(doc: &Doc, node_id: &NodeId) {
    let node_key = node_id.0.to_string();
    let mut txn = doc.transact_mut();
    let nodes = txn.get_or_insert_map("nodes");
    nodes.remove(&mut txn, &node_key);
}

/// Write (replace) a text field for a node with author attribution.
fn write_node_field(doc: &Doc, node_id: &NodeId, field: ContentField, text: &str, author: &str) {
    let node_key = node_id.0.to_string();
    let field_name = match field {
        ContentField::Notes => "notes",
        ContentField::Content => "content",
    };

    let mut txn = doc.transact_mut();
    let nodes = txn.get_or_insert_map("nodes");
    let node_map = get_or_create_node_map(&nodes, &mut txn, &node_key);
    let ytext = get_or_create_text_field(&node_map, &mut txn, field_name);

    // Clear existing content and write new text with attribution.
    let len = ytext.len(&txn);
    if len > 0 {
        ytext.remove_range(&mut txn, 0, len);
    }

    if !text.is_empty() {
        let attrs = Attrs::from([("author".into(), Any::String(author.into()))]);
        ytext.insert_with_attributes(&mut txn, 0, text, attrs);
    }
}

/// Append text to a node field (used for AI token streaming).
fn append_to_node_field(
    doc: &Doc,
    node_id: &NodeId,
    field: ContentField,
    text: &str,
    author: &str,
) {
    if text.is_empty() {
        return;
    }

    let node_key = node_id.0.to_string();
    let field_name = match field {
        ContentField::Notes => "notes",
        ContentField::Content => "content",
    };

    let mut txn = doc.transact_mut();
    let nodes = txn.get_or_insert_map("nodes");
    let node_map = get_or_create_node_map(&nodes, &mut txn, &node_key);
    let ytext = get_or_create_text_field(&node_map, &mut txn, field_name);

    let len = ytext.len(&txn);
    let attrs = Attrs::from([("author".into(), Any::String(author.into()))]);
    ytext.insert_with_attributes(&mut txn, len, text, attrs);
}

/// Replace a character range in a Y.Text field with new text.
///
/// Clamps `start` and `end` to the current Y.Text length.
/// If `start >= end` after clamping, logs a warning and does nothing.
fn rewrite_region(
    doc: &Doc,
    node_id: &NodeId,
    field: ContentField,
    start: usize,
    end: usize,
    new_text: &str,
    author: &str,
) {
    let node_key = node_id.0.to_string();
    let field_name = match field {
        ContentField::Notes => "notes",
        ContentField::Content => "content",
    };

    let mut txn = doc.transact_mut();
    let nodes = txn.get_or_insert_map("nodes");
    let node_map = get_or_create_node_map(&nodes, &mut txn, &node_key);
    let ytext = get_or_create_text_field(&node_map, &mut txn, field_name);

    let len = ytext.len(&txn) as usize;
    let clamped_start = start.min(len);
    let clamped_end = end.min(len);

    if clamped_start >= clamped_end {
        tracing::warn!(
            "rewrite_region: start ({start}) >= end ({end}) after clamping to len {len} — no-op"
        );
        return;
    }

    // Remove the old range, then insert the replacement.
    let remove_len = (clamped_end - clamped_start) as u32;
    ytext.remove_range(&mut txn, clamped_start as u32, remove_len);

    if !new_text.is_empty() {
        let attrs = Attrs::from([("author".into(), Any::String(author.into()))]);
        ytext.insert_with_attributes(&mut txn, clamped_start as u32, new_text, attrs);
    }
}

/// Read a snapshot of a node's text content from Y.Doc.
fn read_node_snapshot(doc: &Doc, node_id: &NodeId) -> NodeTextSnapshot {
    let node_key = node_id.0.to_string();
    let txn = doc.transact();
    let nodes_map = txn.get_map("nodes");

    let (notes, content, spans) = match nodes_map {
        Some(nodes) => match nodes.get(&txn, &node_key) {
            Some(yrs::Out::YMap(node_map)) => {
                let notes = read_text_field(&node_map, &txn, "notes");
                let content = read_text_field(&node_map, &txn, "content");
                let spans = read_attributed_spans(&node_map, &txn, "content");
                (notes, content, spans)
            }
            _ => (String::new(), String::new(), Vec::new()),
        },
        None => (String::new(), String::new(), Vec::new()),
    };

    NodeTextSnapshot {
        notes,
        content,
        attributed_spans: spans,
    }
}

/// Read all node snapshots from Y.Doc (keyed by node UUID string).
fn read_all_node_snapshots(doc: &Doc) -> HashMap<String, NodeTextSnapshot> {
    let txn = doc.transact();
    let mut result = HashMap::new();

    if let Some(nodes) = txn.get_map("nodes") {
        for (key, value) in nodes.iter(&txn) {
            if let yrs::Out::YMap(node_map) = value {
                let notes = read_text_field(&node_map, &txn, "notes");
                let content = read_text_field(&node_map, &txn, "content");
                let spans = read_attributed_spans(&node_map, &txn, "content");
                result.insert(
                    key.to_string(),
                    NodeTextSnapshot {
                        notes,
                        content,
                        attributed_spans: spans,
                    },
                );
            }
        }
    }

    result
}

/// Read plain text from a Y.Text field within a node map.
fn read_text_field(node_map: &MapRef, txn: &yrs::Transaction<'_>, field_name: &str) -> String {
    match node_map.get(txn, field_name) {
        Some(yrs::Out::YText(text)) => text.get_string(txn),
        _ => String::new(),
    }
}

/// Read attributed spans from a Y.Text field (content field for author tracking).
fn read_attributed_spans(
    node_map: &MapRef,
    txn: &yrs::Transaction<'_>,
    field_name: &str,
) -> Vec<AttributedSpan> {
    let text_ref = match node_map.get(txn, field_name) {
        Some(yrs::Out::YText(t)) => t,
        _ => return Vec::new(),
    };

    let diff = text_ref.diff(txn, yrs::types::text::YChange::identity);
    let mut spans = Vec::new();
    let mut offset = 0usize;

    for chunk in diff {
        let chunk_text = match &chunk.insert {
            yrs::Out::Any(Any::String(s)) => s.to_string(),
            _ => continue,
        };
        let len = chunk_text.len();
        let author = chunk
            .attributes
            .as_ref()
            .and_then(|attrs| attrs.get("author"))
            .and_then(|v| match v {
                Any::String(s) => Some(s.to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string());

        spans.push(AttributedSpan {
            text: chunk_text,
            author,
            start: offset,
            end: offset + len,
        });
        offset += len;
    }

    spans
}

/// Apply a binary update from a WebSocket client.
///
/// Determines which nodes were affected and sends `ContentChange` messages
/// to the AI reactor channel.
async fn apply_client_update(
    doc: &Doc,
    update_bytes: &[u8],
    client_id: u64,
    change_tx: &mpsc::Sender<ContentChange>,
) -> Result<(), String> {
    // Snapshot node text content before applying the update.
    let before = snapshot_all_text(doc);

    // Apply the update.
    let update = Update::decode_v1(update_bytes).map_err(|e| format!("decode error: {e}"))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| format!("apply error: {e}"))?;

    // Compare and detect which nodes/fields changed.
    let after = snapshot_all_text(doc);
    for (node_key, after_text) in &after {
        let before_text = before.get(node_key);
        let notes_changed = before_text.map_or(true, |b| b.0 != after_text.0);
        let content_changed = before_text.map_or(true, |b| b.1 != after_text.1);

        if let Ok(uuid) = uuid::Uuid::parse_str(node_key) {
            let node_id = NodeId(uuid);
            let author = format!("human:{client_id}");

            if notes_changed {
                let _ = change_tx
                    .try_send(ContentChange {
                        node_id,
                        field: ContentField::Notes,
                        changed_by: author.clone(),
                    });
            }
            if content_changed {
                let _ = change_tx
                    .try_send(ContentChange {
                        node_id,
                        field: ContentField::Content,
                        changed_by: author,
                    });
            }
        }
    }

    Ok(())
}

/// Snapshot all node text for change detection (notes, content).
fn snapshot_all_text(doc: &Doc) -> HashMap<String, (String, String)> {
    let txn = doc.transact();
    let mut result = HashMap::new();

    if let Some(nodes) = txn.get_map("nodes") {
        for (key, value) in nodes.iter(&txn) {
            if let yrs::Out::YMap(node_map) = value {
                let notes = read_text_field(&node_map, &txn, "notes");
                let content = read_text_field(&node_map, &txn, "content");
                result.insert(key.to_string(), (notes, content));
            }
        }
    }

    result
}

/// Load full doc state from a persistence blob. Replaces current doc content.
fn load_doc_state(doc: &Doc, state: &[u8]) -> Result<(), String> {
    if state.is_empty() {
        return Ok(());
    }
    let update = Update::decode_v1(state).map_err(|e| format!("decode error: {e}"))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| format!("apply error: {e}"))?;
    Ok(())
}

// ──────────────────────────────────────────────
// Convenience helpers for callers
// ──────────────────────────────────────────────

/// Helper: send a WriteNodeContent command (fire-and-forget).
pub async fn write_content(
    doc_tx: &mpsc::Sender<DocCommand>,
    node_id: NodeId,
    field: ContentField,
    text: String,
    author: String,
) {
    let _ = doc_tx
        .send(DocCommand::WriteNodeContent {
            node_id,
            field,
            text,
            author,
        })
        .await;
}

/// Helper: read a node's text snapshot via the doc manager.
pub async fn read_content(
    doc_tx: &mpsc::Sender<DocCommand>,
    node_id: NodeId,
) -> Option<NodeTextSnapshot> {
    let (reply_tx, reply_rx) = oneshot::channel();
    doc_tx
        .send(DocCommand::ReadNodeContent {
            node_id,
            reply: reply_tx,
        })
        .await
        .ok()?;
    reply_rx.await.ok()
}

/// Helper: serialize full doc state.
pub async fn serialize_doc(doc_tx: &mpsc::Sender<DocCommand>) -> Option<Vec<u8>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    doc_tx
        .send(DocCommand::Serialize { reply: reply_tx })
        .await
        .ok()?;
    reply_rx.await.ok()
}

/// Helper: get state vector for sync handshake.
pub async fn get_state_vector(doc_tx: &mpsc::Sender<DocCommand>) -> Option<Vec<u8>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    doc_tx
        .send(DocCommand::GetStateVector { reply: reply_tx })
        .await
        .ok()?;
    reply_rx.await.ok()
}

/// Helper: get diff from a remote state vector.
pub async fn get_diff(doc_tx: &mpsc::Sender<DocCommand>, sv: Vec<u8>) -> Option<Vec<u8>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    doc_tx
        .send(DocCommand::GetDiff {
            sv,
            reply: reply_tx,
        })
        .await
        .ok()?;
    reply_rx.await.ok()
}

/// Helper: load persisted doc state into the manager.
pub async fn load_doc(
    doc_tx: &mpsc::Sender<DocCommand>,
    state: Vec<u8>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    doc_tx
        .send(DocCommand::Load {
            state,
            reply: reply_tx,
        })
        .await
        .map_err(|_| "doc manager channel closed".to_string())?;
    reply_rx
        .await
        .map_err(|_| "doc manager reply dropped".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// Create a Doc and run basic write/read operations synchronously.
    #[test]
    fn write_and_read_node_content() {
        let doc = Doc::with_options(Options {
            client_id: 0,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            let _ = txn.get_or_insert_map("nodes");
        }

        let node_id = NodeId(Uuid::new_v4());
        ensure_node_exists(&doc, &node_id);

        // Write notes.
        write_node_field(
            &doc,
            &node_id,
            ContentField::Notes,
            "Test notes",
            "human:1",
        );

        // Write content.
        write_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "Test content",
            "ai:gen-1",
        );

        // Read back.
        let snapshot = read_node_snapshot(&doc, &node_id);
        assert_eq!(snapshot.notes, "Test notes");
        assert_eq!(snapshot.content, "Test content");

        // Check attribution.
        assert_eq!(snapshot.attributed_spans.len(), 1);
        assert_eq!(snapshot.attributed_spans[0].author, "ai:gen-1");
        assert_eq!(snapshot.attributed_spans[0].text, "Test content");
    }

    #[test]
    fn append_tokens_to_content() {
        let doc = Doc::with_options(Options {
            client_id: 0,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            let _ = txn.get_or_insert_map("nodes");
        }

        let node_id = NodeId(Uuid::new_v4());
        ensure_node_exists(&doc, &node_id);

        // Simulate token streaming.
        append_to_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "Hello ",
            "ai:gen-1",
        );
        append_to_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "world!",
            "ai:gen-1",
        );

        let snapshot = read_node_snapshot(&doc, &node_id);
        assert_eq!(snapshot.content, "Hello world!");
    }

    #[test]
    fn serialize_and_restore() {
        let doc = Doc::with_options(Options {
            client_id: 0,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            let _ = txn.get_or_insert_map("nodes");
        }

        let node_id = NodeId(Uuid::new_v4());
        ensure_node_exists(&doc, &node_id);
        write_node_field(&doc, &node_id, ContentField::Content, "Persist me", "human:1");

        // Serialize.
        let txn = doc.transact();
        let state = txn.encode_state_as_update_v1(&yrs::StateVector::default());
        drop(txn);

        // Restore into a fresh doc.
        let doc2 = Doc::with_options(Options {
            client_id: 1,
            ..Options::default()
        });
        load_doc_state(&doc2, &state).unwrap();

        let snapshot = read_node_snapshot(&doc2, &node_id);
        assert_eq!(snapshot.content, "Persist me");
    }

    #[test]
    fn remove_node_cleans_up() {
        let doc = Doc::with_options(Options {
            client_id: 0,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            let _ = txn.get_or_insert_map("nodes");
        }

        let node_id = NodeId(Uuid::new_v4());
        ensure_node_exists(&doc, &node_id);
        write_node_field(&doc, &node_id, ContentField::Content, "Some text", "human:1");

        // Remove it.
        remove_node(&doc, &node_id);

        // Reading should return empty.
        let snapshot = read_node_snapshot(&doc, &node_id);
        assert_eq!(snapshot.content, "");
        assert_eq!(snapshot.notes, "");
    }

    #[test]
    fn mixed_authorship_spans() {
        let doc = Doc::with_options(Options {
            client_id: 0,
            ..Options::default()
        });
        {
            let mut txn = doc.transact_mut();
            let _ = txn.get_or_insert_map("nodes");
        }

        let node_id = NodeId(Uuid::new_v4());
        ensure_node_exists(&doc, &node_id);

        // Human writes, then AI appends.
        write_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "Human text. ",
            "human:1",
        );
        append_to_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "AI continuation.",
            "ai:gen-1",
        );

        let snapshot = read_node_snapshot(&doc, &node_id);
        assert_eq!(snapshot.content, "Human text. AI continuation.");
        assert_eq!(snapshot.attributed_spans.len(), 2);
        assert_eq!(snapshot.attributed_spans[0].author, "human:1");
        assert_eq!(snapshot.attributed_spans[0].text, "Human text. ");
        assert_eq!(snapshot.attributed_spans[1].author, "ai:gen-1");
        assert_eq!(snapshot.attributed_spans[1].text, "AI continuation.");
    }

    #[tokio::test]
    async fn spawn_and_communicate() {
        let (doc_tx, _update_tx, _change_rx) = spawn_doc_manager();

        let node_id = NodeId(Uuid::new_v4());

        // Ensure node exists.
        doc_tx
            .send(DocCommand::EnsureNode { node_id })
            .await
            .unwrap();

        // Write content.
        doc_tx
            .send(DocCommand::WriteNodeContent {
                node_id,
                field: ContentField::Content,
                text: "Hello from task".into(),
                author: "test:1".into(),
            })
            .await
            .unwrap();

        // Read it back.
        let snapshot = read_content(&doc_tx, node_id).await.unwrap();
        assert_eq!(snapshot.content, "Hello from task");

        // Shutdown.
        doc_tx.send(DocCommand::Shutdown).await.unwrap();
    }
}
