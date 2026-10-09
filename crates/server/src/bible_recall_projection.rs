//! Explicit one-hop read, independent of renderer selection and generation context.
use crate::history_store::{self, HistoryStoreError};
use eidetic_core::contracts::*;
use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn load(
    conn: &Connection,
    request: &BibleRecallRequest,
) -> Result<ProjectionEnvelope<BibleRecallProjection>, HistoryStoreError> {
    request
        .validate()
        .map_err(|message| HistoryStoreError::InvalidValue(message.into()))?;
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let result = load_in_snapshot(&tx, request)?;
        tx.commit()?;
        Ok(result)
    } else {
        load_in_snapshot(conn, request)
    }
}

fn load_in_snapshot(
    conn: &Connection,
    request: &BibleRecallRequest,
) -> Result<ProjectionEnvelope<BibleRecallProjection>, HistoryStoreError> {
    let anchor = crate::bible_graph_store::load_node(conn, &request.anchor_node_id)?
        .ok_or_else(|| HistoryStoreError::InvalidValue("Recall anchor no longer exists".into()))?;
    let mut edges = BTreeMap::new();
    for edge in crate::bible_graph_edge_store::load_outgoing_edges(conn, &anchor.id)?
        .into_iter()
        .chain(crate::bible_graph_edge_store::load_incoming_edges(
            conn, &anchor.id,
        )?)
    {
        if !request.edge_kinds.is_empty() && !request.edge_kinds.contains(&edge.edge_kind) {
            continue;
        }
        let outgoing = edge.from_node_id == anchor.id;
        if edge.directed
            && matches!(
                (request.direction, outgoing),
                (BibleRecallDirection::Incoming, true) | (BibleRecallDirection::Outgoing, false)
            )
        {
            continue;
        }
        let peer = if outgoing {
            &edge.to_node_id
        } else {
            &edge.from_node_id
        };
        if peer == &anchor.id || crate::bible_graph_store::load_node(conn, peer)?.is_none() {
            continue;
        }
        edges.insert(edge.id.as_str().to_owned(), (peer.clone(), edge));
    }
    let peers: BTreeSet<_> = edges.values().map(|(peer, _)| peer.clone()).collect();
    let selected: BTreeSet<_> = peers
        .iter()
        .take(request.neighbor_limit as usize)
        .cloned()
        .collect();
    // Reserve one explanation for each neighbor before spending the parallel-edge budget.
    let mut included = BTreeSet::new();
    for peer in &selected {
        if let Some((id, _)) = edges.iter().find(|(_, (candidate, _))| candidate == peer) {
            included.insert(id.clone());
        }
    }
    for (id, (peer, _)) in &edges {
        if included.len() >= BIBLE_RECALL_MAX_EDGES {
            break;
        }
        if selected.contains(peer) {
            included.insert(id.clone());
        }
    }
    let paths = included
        .iter()
        .map(|id| {
            let (peer, edge) = &edges[id];
            crate::bible_recall_evidence::path(conn, peer.clone(), edge.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut nodes = vec![crate::bible_recall_evidence::node(
        conn,
        anchor,
        request.story_time_ms,
    )?];
    for peer in &selected {
        let stored = crate::bible_graph_store::load_node(conn, peer)?.ok_or_else(|| {
            HistoryStoreError::InvalidValue("Recall neighbor no longer exists".into())
        })?;
        nodes.push(crate::bible_recall_evidence::node(
            conn,
            stored,
            request.story_time_ms,
        )?);
    }
    let projection = BibleRecallProjection {
        request: request.clone(),
        nodes,
        paths,
        omitted_neighbors: (peers.len() - selected.len()) as u64,
        omitted_edges: (edges.len() - included.len()) as u64,
        relationships_untimed: true,
    };
    let summary = history_store::load_revision_summary_for_kinds(
        conn,
        &[
            ObjectKind::BibleNode,
            ObjectKind::BiblePartField,
            ObjectKind::BibleEdge,
            ObjectKind::BibleSnapshot,
        ],
    )?;
    let mut envelope = match summary.latest_change_event_id {
        Some(event) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            event,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    };
    while serde_json::to_vec(&envelope)?.len() > BIBLE_RECALL_MAX_BYTES {
        let Some(node) = envelope
            .payload
            .nodes
            .iter_mut()
            .rev()
            .find(|node| !node.fields.is_empty())
        else {
            return Err(HistoryStoreError::InvalidValue(
                "Recall identities and path evidence exceed the 32 KiB budget".into(),
            ));
        };
        node.fields.pop();
        node.omitted_fields += 1;
    }
    Ok(envelope)
}

#[cfg(test)]
#[path = "bible_recall_projection_tests.rs"]
mod tests;
