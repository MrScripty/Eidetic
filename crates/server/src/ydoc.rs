//! Y.Doc manager: single-owner async task for CRDT content.
//!
//! All text content (notes, scripts, outlines) lives in a Yrs `Doc`.
//! This module provides a channel-based interface so that concurrent
//! callers (Tauri commands, desktop event adapters, AI generation) never
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

use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot};
use tracing;
#[cfg(test)]
use yrs::GetString;
use yrs::types::Attrs;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Doc, Map, MapPrelim, MapRef, Options, ReadTxn, Text, TextPrelim, TextRef, Transact,
    Update, WriteTxn,
};

use crate::backend_task::BackendTaskSupervisor;
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
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct NodeTextSnapshot {
    pub notes: String,
    pub content: String,
    pub attributed_spans: Vec<AttributedSpan>,
}

/// A contiguous span of text with a single author.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct AttributedSpan {
    pub text: String,
    /// e.g. "human:{user_id}" or "ai:{generation_id}"
    pub author: String,
}

/// A binary update to broadcast to document update subscribers.
#[derive(Debug, Clone)]
pub struct DocUpdate {
    /// Client that originated this update (0 = server/AI).
    pub origin_client: u64,
    /// Encoded Yrs update bytes (v1 encoding).
    pub data: Vec<u8>,
}

/// Commands sent to the Y.Doc manager task via channel.
pub enum DocCommand {
    /// Apply a binary update from a collaborative document client.
    ApplyUpdate {
        client_id: u64,
        update: Vec<u8>,
        reply: oneshot::Sender<Result<(), String>>,
    },
    /// Get the full state vector for sync handshake.
    GetStateVector { reply: oneshot::Sender<Vec<u8>> },
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
    #[cfg(test)]
    ReadNodeContent {
        node_id: NodeId,
        reply: oneshot::Sender<NodeTextSnapshot>,
    },
    /// Ensure a node entry exists in Y.Doc when a timeline node is created.
    EnsureNode { node_id: NodeId },
    /// Remove a node entry from Y.Doc when a timeline node is deleted.
    RemoveNode { node_id: NodeId },
    /// Serialize full doc state for persistence.
    Serialize { reply: oneshot::Sender<Vec<u8>> },
    /// Load doc state from persistence (replaces current doc content).
    Load {
        state: Vec<u8>,
        reply: oneshot::Sender<Result<(), String>>,
    },
}

/// Channel capacity for the doc command queue.
pub const DOC_CHANNEL_CAPACITY: usize = 256;

/// Channel capacity for the doc update broadcast feed.
pub const UPDATE_BROADCAST_CAPACITY: usize = 256;

// ──────────────────────────────────────────────
// Doc manager task
// ──────────────────────────────────────────────

/// Spawn the doc manager and return the command sender.
///
/// Also returns the broadcast sender for doc updates.
pub fn spawn_doc_manager(
    supervisor: &BackendTaskSupervisor,
) -> (mpsc::Sender<DocCommand>, broadcast::Sender<DocUpdate>) {
    let (cmd_tx, cmd_rx) = mpsc::channel(DOC_CHANNEL_CAPACITY);
    let (update_tx, _) = broadcast::channel(UPDATE_BROADCAST_CAPACITY);

    let update_tx_clone = update_tx.clone();
    supervisor.spawn("y-doc-manager", run_doc_manager(cmd_rx, update_tx_clone));

    (cmd_tx, update_tx)
}

/// The main loop of the Y.Doc manager task.
///
/// Owns the `Doc` exclusively — no external code touches it.
/// All interaction is via [`DocCommand`] messages.
async fn run_doc_manager(
    mut rx: mpsc::Receiver<DocCommand>,
    update_tx: broadcast::Sender<DocUpdate>,
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

    // Subscribe to doc updates for broadcasting to document update subscribers.
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
                let result = apply_client_update(&doc, &update).await;
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

            #[cfg(test)]
            DocCommand::ReadNodeContent { node_id, reply } => {
                let snapshot = read_node_snapshot(&doc, &node_id);
                let _ = reply.send(snapshot);
            }

            DocCommand::EnsureNode { node_id } => {
                ensure_node_exists(&doc, &node_id);
            }

            DocCommand::RemoveNode { node_id } => {
                remove_node(&doc, &node_id);
            }

            DocCommand::Serialize { reply } => {
                let txn = doc.transact();
                let state = txn.encode_state_as_update_v1(&yrs::StateVector::default());
                let _ = reply.send(state);
            }

            DocCommand::Load { state, reply } => {
                let result = load_doc_state(&doc, &state);
                let _ = reply.send(result);
            }
        }
    }
    tracing::info!("Y.Doc manager shutting down");
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
#[cfg(test)]
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

/// Read a snapshot of a node's text content from Y.Doc.
#[cfg(test)]
fn read_node_snapshot(doc: &Doc, node_id: &NodeId) -> NodeTextSnapshot {
    let node_key = node_id.0.to_string();
    let txn = doc.transact();
    let nodes_map = txn.get_map("nodes");

    let content = match nodes_map.as_ref() {
        Some(nodes) => match nodes.get(&txn, &node_key) {
            Some(yrs::Out::YMap(node_map)) => read_text_field(&node_map, &txn, "content"),
            _ => String::new(),
        },
        None => String::new(),
    };

    #[cfg(test)]
    let (notes, attributed_spans) = match nodes_map.as_ref() {
        Some(nodes) => match nodes.get(&txn, &node_key) {
            Some(yrs::Out::YMap(node_map)) => (
                read_text_field(&node_map, &txn, "notes"),
                read_attributed_spans(&node_map, &txn, "content"),
            ),
            _ => (String::new(), Vec::new()),
        },
        None => (String::new(), Vec::new()),
    };

    NodeTextSnapshot {
        #[cfg(test)]
        notes,
        content,
        #[cfg(test)]
        attributed_spans,
    }
}

/// Read plain text from a Y.Text field within a node map.
#[cfg(test)]
fn read_text_field(node_map: &MapRef, txn: &yrs::Transaction<'_>, field_name: &str) -> String {
    match node_map.get(txn, field_name) {
        Some(yrs::Out::YText(text)) => text.get_string(txn),
        _ => String::new(),
    }
}

/// Read attributed spans from a Y.Text field (content field for author tracking).
#[cfg(test)]
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

    for chunk in diff {
        let chunk_text = match &chunk.insert {
            yrs::Out::Any(Any::String(s)) => s.to_string(),
            _ => continue,
        };
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
        });
    }

    spans
}

/// Apply a binary update from a collaborative document client.
async fn apply_client_update(doc: &Doc, update_bytes: &[u8]) -> Result<(), String> {
    let update = Update::decode_v1(update_bytes).map_err(|e| format!("decode error: {e}"))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| format!("apply error: {e}"))?;
    Ok(())
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

/// Helper: read a node's text snapshot via the doc manager.
#[cfg(test)]
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
pub async fn load_doc(doc_tx: &mpsc::Sender<DocCommand>, state: Vec<u8>) -> Result<(), String> {
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
        write_node_field(&doc, &node_id, ContentField::Notes, "Test notes", "human:1");

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
        append_to_node_field(&doc, &node_id, ContentField::Content, "Hello ", "ai:gen-1");
        append_to_node_field(&doc, &node_id, ContentField::Content, "world!", "ai:gen-1");

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
        write_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "Persist me",
            "human:1",
        );

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
        write_node_field(
            &doc,
            &node_id,
            ContentField::Content,
            "Some text",
            "human:1",
        );

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
        let supervisor = BackendTaskSupervisor::default();
        let (doc_tx, _update_tx) = spawn_doc_manager(&supervisor);

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

        drop(doc_tx);
        supervisor.abort_all();
    }
}
