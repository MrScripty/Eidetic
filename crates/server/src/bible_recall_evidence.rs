//! Source identities and owned revision custody for the shared recall read.
use crate::history_store::HistoryStoreError;
use eidetic_core::contracts::*;
use rusqlite::{Connection, params};

pub(crate) fn path(
    conn: &Connection,
    peer: BibleGraphNodeId,
    edge: BibleGraphEdge,
) -> Result<BibleRecallPath, HistoryStoreError> {
    let revision_event_id = crate::bible_relationship_lineage::current_revision(
        conn,
        &SemanticDependencyEndpoint::BibleEdge {
            edge_id: edge.id.clone(),
        },
    )?
    .ok_or_else(|| {
        HistoryStoreError::InvalidValue("Recall relationship lineage is unknown".into())
    })?;
    let relationship = BibleRelationshipInput {
        revision_event_id,
        edge: AiBibleContextEdge {
            edge_id: edge.id,
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            edge_kind: edge.edge_kind,
            label: edge.label,
            directed: edge.directed,
        },
    };
    crate::bible_relationship_lineage::validate_history(conn, &relationship)?;
    Ok(BibleRecallPath {
        neighbor_node_id: peer,
        relationship,
    })
}

pub(crate) fn node(
    conn: &Connection,
    stored: BibleGraphNode,
    story_time_ms: Option<u64>,
) -> Result<BibleRecallNode, HistoryStoreError> {
    let detail = crate::bible_graph_store::load_node_detail_projection(conn, &stored.id)?
        .ok_or_else(|| HistoryStoreError::InvalidValue("Recall node no longer exists".into()))?;
    let resolved = crate::ai_temporal_context::resolve_fields(
        crate::ai_context_projection::context_fields(detail.parts),
        detail.snapshots,
        story_time_ms,
    )?;
    let name_revision_event_id = crate::bible_node_name_lineage::known_revision(
        conn,
        &SemanticDependencyEndpoint::BibleNode {
            node_id: stored.id.clone(),
        },
    )?;
    if let Some(revision_event_id) = name_revision_event_id {
        crate::bible_node_name_lineage::validate_history(
            conn,
            &BibleNodeNameInput {
                node_id: stored.id.clone(),
                name: stored.name.clone(),
                revision_event_id,
            },
        )?;
    }
    let baseline = AiBibleContextNode {
        node_id: stored.id.clone(),
        parent_id: stored.parent_id,
        schema_key: stored.schema_key.clone(),
        name: stored.name.clone(),
        fields: resolved.fields,
        snapshots: vec![],
        unresolved_timed_fields: vec![],
        incoming_edges: vec![],
        outgoing_edges: vec![],
    };
    let mut fields = Vec::new();
    for input in crate::bible_field_lineage::capture_nodes(conn, &[baseline])? {
        crate::bible_field_lineage::validate_history(conn, &input)?;
        fields.push(BibleRecallField {
            part_key: input.part_key,
            field_key: input.field_key,
            value: input.value,
            source: BibleRecallFieldSource::Baseline {
                field_id: input.field_id,
                revision_event_id: input.revision_event_id,
            },
        });
    }
    for source in resolved.snapshot_sources.into_values() {
        let (snapshot_event, field_event): (String, String) = conn.query_row(
            "SELECT s.updated_event_id, f.updated_event_id FROM bible_graph_snapshots s
             JOIN bible_graph_snapshot_fields f ON f.snapshot_id=s.id
             WHERE s.id=?1 AND f.id=?2 AND s.node_id=?3
               AND s.deleted_event_id IS NULL AND f.deleted_event_id IS NULL",
            params![
                source.field.snapshot_id.as_str(),
                source.field.id.as_str(),
                stored.id.as_str()
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let snapshot_revision_event_id = parse_event(&snapshot_event)?;
        let field_revision_event_id = parse_event(&field_event)?;
        let value = source
            .field
            .value
            .expect("resolver returns only established assertions");
        validate_snapshot(
            conn,
            &stored.id,
            &source.field.snapshot_id,
            snapshot_revision_event_id,
            field_revision_event_id,
            source.at_ms,
            &source.label,
            &source.field.part_key,
            &source.field.field_key,
            &value,
        )?;
        fields.push(BibleRecallField {
            part_key: source.field.part_key,
            field_key: source.field.field_key,
            value,
            source: BibleRecallFieldSource::Snapshot {
                snapshot_id: source.field.snapshot_id,
                snapshot_field_id: source.field.id,
                snapshot_revision_event_id,
                field_revision_event_id,
                at_ms: source.at_ms,
                label: source.label,
            },
        });
    }
    fields.sort_by(|a, b| {
        a.part_key
            .as_str()
            .cmp(b.part_key.as_str())
            .then_with(|| a.field_key.as_str().cmp(b.field_key.as_str()))
    });
    Ok(BibleRecallNode {
        node_id: stored.id,
        name: stored.name,
        schema_key: stored.schema_key,
        name_revision_event_id,
        fields,
        unresolved_timed_fields: resolved.unresolved_fields,
        omitted_fields: 0,
    })
}

// Snapshot metadata and individual assertions can have different latest writes.
// Validate both using existing sparse object history, without a generation receipt.
#[allow(clippy::too_many_arguments)]
fn validate_snapshot(
    conn: &Connection,
    node: &BibleGraphNodeId,
    snapshot: &BibleGraphSnapshotId,
    snapshot_event: ChangeEventId,
    field_event: ChangeEventId,
    at_ms: u64,
    label: &str,
    part: &BibleGraphPartKey,
    field: &BibleGraphFieldKey,
    value: &FieldValue,
) -> Result<(), HistoryStoreError> {
    let metadata = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleSnapshot,
        snapshot.as_str(),
        snapshot_event,
    )?;
    let assertion = crate::revision_projection::load_object_field_projection_at_event(
        conn,
        ObjectKind::BibleSnapshot,
        snapshot.as_str(),
        field_event,
    )?;
    let expected_node = FieldValue::ObjectRef {
        kind: ObjectKind::BibleNode,
        id: node.as_str().into(),
    };
    if !metadata.is_some_and(|history| {
        !history.deleted
            && history.fields.get("node_id") == Some(&expected_node)
            && history.fields.get("at_ms") == Some(&FieldValue::Integer(at_ms as i64))
            && history.fields.get("label") == Some(&FieldValue::Text(label.into()))
    }) || !assertion.is_some_and(|history| {
        !history.deleted
            && history
                .fields
                .get(&format!("field.{}.{}", part.as_str(), field.as_str()))
                == Some(value)
    }) {
        return Err(HistoryStoreError::InvalidValue(
            "Recall snapshot does not match canonical revision history".into(),
        ));
    }
    Ok(())
}

fn parse_event(value: &str) -> Result<ChangeEventId, HistoryStoreError> {
    uuid::Uuid::parse_str(value)
        .map(ChangeEventId)
        .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))
}
