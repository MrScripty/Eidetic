use std::collections::BTreeSet;

use eidetic_core::contracts::{
    AiBibleContextEdge, AiBibleContextField, AiBibleContextNode, AiBibleContextProjection,
    AiBibleContextSnapshot, BibleGraphEdge, BibleGraphEdgeId, BibleGraphNode,
    BibleGraphPartProjection, BibleGraphSnapshotProjection, BibleRenderGraphProjectionRequest,
    ChangeEventId, ObjectKind, ProjectionEnvelope, ProjectionVersion,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::bible_graph_store;
use crate::history_store::{HistoryStoreError, RevisionSummary};

pub(crate) fn load_ai_bible_context_projection(
    conn: &Connection,
    target_node_id: NodeId,
) -> Result<ProjectionEnvelope<AiBibleContextProjection>, HistoryStoreError> {
    bible_graph_store::create_schema(conn)?;

    let request = BibleRenderGraphProjectionRequest {
        selected_timeline_node_id: Some(target_node_id),
        ..BibleRenderGraphProjectionRequest::default()
    };
    let bounded_graph = crate::bible_render_graph_query::load_bounded_render_graph(conn, &request)?;
    let visible_edge_ids: BTreeSet<_> = bounded_graph
        .edges
        .iter()
        .map(|edge| edge.id.clone())
        .collect();
    let nodes = bounded_graph
        .nodes
        .into_iter()
        .filter(|node| !node.system_owned)
        .map(|node| load_context_node(conn, node, &visible_edge_ids))
        .collect::<Result<Vec<_>, _>>()?;
    let projection = AiBibleContextProjection {
        target_node_id,
        nodes,
    };
    let summary = load_revision_summary(conn)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

fn load_context_node(
    conn: &Connection,
    node: BibleGraphNode,
    visible_edge_ids: &BTreeSet<BibleGraphEdgeId>,
) -> Result<AiBibleContextNode, HistoryStoreError> {
    let Some(detail) = bible_graph_store::load_node_detail_projection(conn, &node.id)? else {
        return Err(HistoryStoreError::InvalidValue(format!(
            "bible graph node missing during AI context projection: {}",
            node.id.as_str()
        )));
    };

    Ok(AiBibleContextNode {
        node_id: node.id,
        parent_id: node.parent_id,
        schema_key: node.schema_key,
        name: node.name,
        fields: context_fields(detail.parts),
        snapshots: context_snapshots(detail.snapshots),
        incoming_edges: context_edges(detail.incoming_edges, visible_edge_ids),
        outgoing_edges: context_edges(detail.outgoing_edges, visible_edge_ids),
    })
}

fn context_fields(parts: Vec<BibleGraphPartProjection>) -> Vec<AiBibleContextField> {
    let mut fields = Vec::new();
    for part in parts {
        for field in part.fields {
            let Some(value) = field.value else {
                continue;
            };
            fields.push(AiBibleContextField {
                part_key: part.part.part_key.clone(),
                part_name: part.part.name.clone(),
                field_key: field.field_key,
                value,
            });
        }
    }
    fields
}

fn context_snapshots(snapshots: Vec<BibleGraphSnapshotProjection>) -> Vec<AiBibleContextSnapshot> {
    snapshots
        .into_iter()
        .filter_map(|projection| {
            let fields = projection
                .fields
                .into_iter()
                .filter_map(|field| {
                    field.value.map(|value| AiBibleContextField {
                        part_key: field.part_key,
                        part_name: field.part_name,
                        field_key: field.field_key,
                        value,
                    })
                })
                .collect::<Vec<_>>();

            if fields.is_empty() {
                return None;
            }

            Some(AiBibleContextSnapshot {
                label: projection.snapshot.label,
                at_ms: projection.snapshot.at_ms,
                fields,
            })
        })
        .collect()
}

fn context_edges(
    edges: Vec<BibleGraphEdge>,
    visible_edge_ids: &BTreeSet<BibleGraphEdgeId>,
) -> Vec<AiBibleContextEdge> {
    edges
        .into_iter()
        .filter(|edge| visible_edge_ids.contains(&edge.id))
        .map(|edge| AiBibleContextEdge {
            edge_id: edge.id,
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            edge_kind: edge.edge_kind,
            label: edge.label,
            directed: edge.directed,
        })
        .collect()
}

fn load_revision_summary(conn: &Connection) -> Result<RevisionSummary, HistoryStoreError> {
    let bible_node = encode_object_kind(&ObjectKind::BibleNode)?;
    let bible_part_field = encode_object_kind(&ObjectKind::BiblePartField)?;
    let bible_edge = encode_object_kind(&ObjectKind::BibleEdge)?;
    let bible_snapshot = encode_object_kind(&ObjectKind::BibleSnapshot)?;

    let revision_count = conn.query_row(
        "SELECT COUNT(*)
         FROM object_revisions
         WHERE object_kind IN (?1, ?2, ?3, ?4)",
        params![bible_node, bible_part_field, bible_edge, bible_snapshot],
        |row| row.get::<_, i64>(0),
    )?;
    let latest_change_event_id = conn
        .query_row(
            "SELECT change_event_id
             FROM object_revisions
             WHERE object_kind IN (?1, ?2, ?3, ?4)
             ORDER BY rowid DESC
             LIMIT 1",
            params![
                encode_object_kind(&ObjectKind::BibleNode)?,
                encode_object_kind(&ObjectKind::BiblePartField)?,
                encode_object_kind(&ObjectKind::BibleEdge)?,
                encode_object_kind(&ObjectKind::BibleSnapshot)?,
            ],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|id| parse_uuid(&id).map(ChangeEventId))
        .transpose()?;

    Ok(RevisionSummary {
        revision_count: u64::try_from(revision_count).unwrap_or_default(),
        latest_change_event_id,
    })
}

fn encode_object_kind(value: &ObjectKind) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected object kind to serialize as string".to_string(),
        )),
    }
}

fn parse_uuid(value: &str) -> Result<uuid::Uuid, HistoryStoreError> {
    uuid::Uuid::parse_str(value).map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
}

#[cfg(test)]
#[path = "ai_context_projection_tests.rs"]
mod tests;
