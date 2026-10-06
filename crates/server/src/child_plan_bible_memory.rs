//! Scoped Bible custody in the existing child-plan creation command receipt.
use eidetic_core::ai::backend::ChildPlanBibleContext;
use eidetic_core::contracts::{
    BibleRenderGraphProjectionRequest, ChangeEventId, FieldValue, ObjectKind, RevisionOperation,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::history_store::{self, HistoryStoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChildPlanBibleMemory {
    pub(crate) receipt: ChildPlanBibleContext,
    sources: Vec<(ObjectKind, String, Option<ChangeEventId>)>,
    node_selection_event: Option<ChangeEventId>,
    context_selection_event: Option<ChangeEventId>,
}

impl ChildPlanBibleMemory {
    pub(crate) fn unchanged(&self, expected: &Self) -> bool {
        // The public envelope retains its original display provenance, but its
        // global clock is not source admission: unrelated fact edits are allowed.
        self.receipt.context.payload == expected.receipt.context.payload
            && self.receipt.inputs == expected.receipt.inputs
            && self.sources == expected.sources
            && self.node_selection_event == expected.node_selection_event
            && self.context_selection_event == expected.context_selection_event
    }
}

/// Caller owns the same SQLite snapshot as screenplay and parent custody.
pub(crate) fn capture(
    conn: &Connection,
    parent: NodeId,
    story_time_ms: Option<u64>,
) -> Result<ChildPlanBibleMemory, HistoryStoreError> {
    let context = crate::ai_context_projection::load_ai_bible_context_projection(
        conn,
        parent,
        story_time_ms,
    )?;
    let inputs = crate::bible_field_lineage::capture(conn, &context.payload)?;
    let graph = crate::bible_render_graph_query::load_bounded_render_graph(
        conn,
        &BibleRenderGraphProjectionRequest {
            selected_timeline_node_id: Some(parent),
            ..Default::default()
        },
    )?;
    // Roots can be endpoints of consumed edges despite having no prompt fields.
    let graph_nodes = graph
        .nodes
        .iter()
        .map(|node| node.id.as_str().to_owned())
        .collect();
    let mut sources = Vec::new();
    for node in &context.payload.nodes {
        bind(
            conn,
            ObjectKind::BibleNode,
            node.node_id.as_str(),
            &mut sources,
        )?;
        for id in owned_ids(
            conn,
            "SELECT f.id FROM bible_graph_fields f JOIN bible_graph_parts p ON p.id=f.part_id WHERE p.node_id=?1 ORDER BY f.id",
            node.node_id.as_str(),
        )? {
            // Null and tombstoned rows retain collection/identity custody; never
            // claim their values were consumed by the temporal resolver.
            bind(conn, ObjectKind::BiblePartField, &id, &mut sources)?;
        }
        for id in owned_ids(
            conn,
            "SELECT id FROM bible_graph_snapshots WHERE node_id=?1 ORDER BY id",
            node.node_id.as_str(),
        )? {
            // Snapshot revisions own all sparse field assertions, including
            // withheld/future assertions which influence unknown membership.
            bind(conn, ObjectKind::BibleSnapshot, &id, &mut sources)?;
        }
    }
    let mut edge_sources = historical_edges(conn, &graph_nodes)?;
    for id in graph.edges.into_iter().map(|edge| edge.id).chain(
        graph
            .influences
            .into_iter()
            .filter_map(|record| record.bible_edge_id),
    ) {
        edge_sources.insert(
            id.as_str().to_owned(),
            latest(conn, ObjectKind::BibleEdge, id.as_str())?,
        );
    }
    for (id, event) in edge_sources {
        sources.push((ObjectKind::BibleEdge, id, event));
    }
    // Scope context history to this target, including older evaluations that
    // could be hidden by caller timestamps. No unrelated target advances it.
    let context_selection_event = event_query(conn,
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         WHERE (r.object_kind='context_evaluation' AND r.object_id IN (SELECT id FROM context_evaluations WHERE target_node_id=?1))
            OR (r.object_kind='context_influence' AND r.object_id IN (SELECT i.id FROM context_influence_records i JOIN context_evaluations v ON v.id=i.evaluation_id WHERE v.target_node_id=?1))
         ORDER BY e.rowid DESC,r.sort_order DESC,r.rowid DESC LIMIT 1", Some(&parent.0.to_string()))?;
    // The bounded selector orders every node by name/sort and includes ancestor
    // and explicit membership. Metadata-only collection custody detects node
    // entry/removal/rename ABA; unselected field/snapshot/edge edits do not advance it.
    let node_selection_event = event_query(conn,
        "SELECT r.change_event_id FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id
         WHERE r.object_kind='bible_node' ORDER BY e.rowid DESC,r.sort_order DESC,r.rowid DESC LIMIT 1", None)?;
    Ok(ChildPlanBibleMemory {
        receipt: ChildPlanBibleContext { context, inputs },
        sources,
        node_selection_event,
        context_selection_event,
    })
}

fn bind(
    conn: &Connection,
    kind: ObjectKind,
    id: &str,
    sources: &mut Vec<(ObjectKind, String, Option<ChangeEventId>)>,
) -> Result<(), HistoryStoreError> {
    let event = latest(conn, kind.clone(), id)?;
    sources.push((kind, id.to_owned(), event));
    Ok(())
}

fn latest(
    conn: &Connection,
    kind: ObjectKind,
    id: &str,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    Ok(history_store::load_revisions_for_object(conn, kind, id)?
        .last()
        .map(|revision| revision.change_event_id))
}

fn owned_ids(conn: &Connection, sql: &str, owner: &str) -> Result<Vec<String>, HistoryStoreError> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map([owner], |row| row.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn event_query(
    conn: &Connection,
    sql: &str,
    target: Option<&str>,
) -> Result<Option<ChangeEventId>, HistoryStoreError> {
    let raw: Option<String> = match target {
        Some(target) => conn.query_row(sql, [target], |row| row.get(0)).optional()?,
        None => conn.query_row(sql, [], |row| row.get(0)).optional()?,
    };
    raw.map(|raw| {
        uuid::Uuid::parse_str(&raw)
            .map(ChangeEventId)
            .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
    })
    .transpose()
}

fn historical_edges(
    conn: &Connection,
    nodes: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<ChangeEventId>>, HistoryStoreError> {
    // Reconstruct sparse endpoint deltas in committed event order. A rewiring
    // can move an initially unselected edge into this graph and back without
    // leaving either a current edge or a paired delta in a single revision.
    let mut statement = conn.prepare(
        "SELECT DISTINCT object_id FROM object_revisions WHERE object_kind='bible_edge' ORDER BY object_id",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut selected = BTreeMap::new();
    for row in rows {
        let id = row?;
        let mut from = None;
        let mut to = None;
        for revision in history_store::load_revisions_for_object(conn, ObjectKind::BibleEdge, &id)?
        {
            for field in &revision.fields {
                match field.field_key.as_str() {
                    "from_node_id" if field.old_value.is_some() => {
                        from = endpoint(&field.old_value)
                    }
                    "to_node_id" if field.old_value.is_some() => to = endpoint(&field.old_value),
                    _ => {}
                }
            }
            if selected_pair(nodes, &from, &to) {
                selected.insert(id.clone(), Some(revision.change_event_id));
            }
            for field in &revision.fields {
                match field.field_key.as_str() {
                    "from_node_id" => from = endpoint(&field.new_value),
                    "to_node_id" => to = endpoint(&field.new_value),
                    _ => {}
                }
            }
            if revision.operation == RevisionOperation::Delete {
                from = None;
                to = None;
            }
            if selected_pair(nodes, &from, &to) {
                selected.insert(id.clone(), Some(revision.change_event_id));
            }
        }
    }
    // Only the last membership-relevant event binds a historical-only edge;
    // later label edits outside this graph do not fabricate source consumption.
    Ok(selected)
}

fn endpoint(value: &Option<FieldValue>) -> Option<String> {
    match value {
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::BibleNode,
            id,
        }) => Some(id.clone()),
        _ => None,
    }
}

fn selected_pair(nodes: &BTreeSet<String>, from: &Option<String>, to: &Option<String>) -> bool {
    from.as_ref()
        .zip(to.as_ref())
        .is_some_and(|(from, to)| nodes.contains(from) && nodes.contains(to))
}
