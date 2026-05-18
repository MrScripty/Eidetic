use eidetic_core::contracts::{
    BibleGraphNode, BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphPartProjection,
    BibleGraphSchemaKey, BibleNodeDetailProjection, ChangeEventId, ObjectKind, ProjectionEnvelope,
    ProjectionVersion, SetBibleGraphFieldCommand, canonical_bible_root_nodes,
    default_part_projections_for_node,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::bible_graph_edge_store;
use crate::bible_graph_field_store;
use crate::bible_graph_schema;
use crate::history_store::{self, HistoryStoreError};

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    bible_graph_schema::create_schema(conn)
}

pub(crate) fn insert_node_in_transaction(
    tx: &Transaction<'_>,
    node: &BibleGraphNode,
    created_event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let existing = tx
        .query_row(
            "SELECT 1 FROM bible_graph_nodes WHERE id = ?1",
            [node.id.as_str()],
            |_| Ok(()),
        )
        .optional()?;
    if existing.is_some() {
        return Err(HistoryStoreError::InvalidValue(format!(
            "bible graph node already exists: {}",
            node.id.as_str()
        )));
    }

    tx.execute(
        "INSERT INTO bible_graph_nodes (
            id, parent_id, schema_key, name, system_owned, sort_order, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            node.id.as_str(),
            node.parent_id.as_ref().map(BibleGraphNodeId::as_str),
            node.schema_key.as_str(),
            node.name,
            if node.system_owned { 1_i64 } else { 0_i64 },
            node.sort_order as i64,
            created_event_id.0.to_string()
        ],
    )?;
    Ok(())
}

pub(crate) fn insert_missing_canonical_roots_in_transaction(
    tx: &Transaction<'_>,
    created_event_id: ChangeEventId,
) -> Result<Vec<BibleGraphNode>, HistoryStoreError> {
    let mut inserted = Vec::new();
    for node in canonical_bible_root_nodes() {
        if node_exists_in_transaction(tx, &node.id)? {
            continue;
        }
        insert_node_in_transaction(tx, &node, created_event_id)?;
        inserted.push(node);
    }
    Ok(inserted)
}

pub(crate) fn set_field_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    bible_graph_field_store::set_field_in_transaction(tx, command, event_id)
}

pub(crate) fn missing_canonical_root_nodes(
    conn: &Connection,
) -> Result<Vec<BibleGraphNode>, HistoryStoreError> {
    let mut missing = Vec::new();
    for node in canonical_bible_root_nodes() {
        if !node_exists(conn, &node.id)? {
            missing.push(node);
        }
    }
    Ok(missing)
}

pub(crate) fn load_node_detail_projection(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Option<BibleNodeDetailProjection>, HistoryStoreError> {
    let node = conn
        .query_row(
            "SELECT id, parent_id, schema_key, name, system_owned, sort_order
             FROM bible_graph_nodes
             WHERE id = ?1 AND deleted_event_id IS NULL",
            [node_id.as_str()],
            row_to_node,
        )
        .optional()?;

    let Some(node) = node else {
        return Ok(None);
    };
    let parts = merge_default_part_projections(
        &node,
        bible_graph_field_store::load_part_projections(conn, node_id)?,
    );

    let incoming_edges = bible_graph_edge_store::load_incoming_edges(conn, node_id)?;
    let outgoing_edges = bible_graph_edge_store::load_outgoing_edges(conn, node_id)?;

    Ok(Some(BibleNodeDetailProjection {
        node,
        parts,
        incoming_edges,
        outgoing_edges,
        snapshots: Vec::new(),
    }))
}

fn merge_default_part_projections(
    node: &BibleGraphNode,
    persisted_parts: Vec<BibleGraphPartProjection>,
) -> Vec<BibleGraphPartProjection> {
    let mut parts = default_part_projections_for_node(node);

    for persisted_part in persisted_parts {
        if let Some(default_part) = parts
            .iter_mut()
            .find(|part| part.part.part_key.as_str() == persisted_part.part.part_key.as_str())
        {
            merge_persisted_fields(default_part, persisted_part);
        } else {
            parts.push(persisted_part);
        }
    }

    parts.sort_by(|a, b| {
        a.part
            .sort_order
            .cmp(&b.part.sort_order)
            .then_with(|| a.part.name.cmp(&b.part.name))
            .then_with(|| a.part.id.as_str().cmp(b.part.id.as_str()))
    });
    parts
}

fn merge_persisted_fields(
    default_part: &mut BibleGraphPartProjection,
    persisted_part: BibleGraphPartProjection,
) {
    for persisted_field in persisted_part.fields {
        if let Some(default_field) = default_part
            .fields
            .iter_mut()
            .find(|field| field.field_key.as_str() == persisted_field.field_key.as_str())
        {
            *default_field = persisted_field;
        } else {
            default_part.fields.push(persisted_field);
        }
    }

    default_part.fields.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.field_key.as_str().cmp(b.field_key.as_str()))
            .then_with(|| a.id.as_str().cmp(b.id.as_str()))
    });
}

pub(crate) fn load_node_detail_projection_envelope(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Option<ProjectionEnvelope<BibleNodeDetailProjection>>, HistoryStoreError> {
    let Some(projection) = load_node_detail_projection(conn, node_id)? else {
        return Ok(None);
    };
    let summary = load_node_detail_revision_summary(conn, node_id)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(Some(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ))),
        None => Ok(Some(ProjectionEnvelope::initial(projection))),
    }
}

pub(crate) fn load_node_list_projection(
    conn: &Connection,
) -> Result<BibleGraphNodeListProjection, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, parent_id, schema_key, name, system_owned, sort_order
         FROM bible_graph_nodes
         WHERE deleted_event_id IS NULL
         ORDER BY sort_order ASC, name ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_node)?;
    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(BibleGraphNodeListProjection { nodes })
}

pub(crate) fn load_node_list_projection_envelope(
    conn: &Connection,
) -> Result<ProjectionEnvelope<BibleGraphNodeListProjection>, HistoryStoreError> {
    let projection = load_node_list_projection(conn)?;
    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::BibleNode)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}

pub(crate) fn node_exists(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<bool, HistoryStoreError> {
    conn.query_row(
        "SELECT 1 FROM bible_graph_nodes WHERE id = ?1 AND deleted_event_id IS NULL",
        [node_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn node_exists_in_transaction(
    tx: &Transaction<'_>,
    node_id: &BibleGraphNodeId,
) -> Result<bool, HistoryStoreError> {
    tx.query_row(
        "SELECT 1 FROM bible_graph_nodes WHERE id = ?1 AND deleted_event_id IS NULL",
        [node_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn load_node_detail_revision_summary(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<history_store::RevisionSummary, HistoryStoreError> {
    let bible_node = encode_object_kind(&ObjectKind::BibleNode)?;
    let bible_part_field = encode_object_kind(&ObjectKind::BiblePartField)?;
    let bible_edge = encode_object_kind(&ObjectKind::BibleEdge)?;
    let revision_count = conn.query_row(
        "SELECT COUNT(*)
         FROM object_revisions
         WHERE (object_kind = ?1 AND object_id = ?2)
            OR (
                object_kind = ?3
                AND object_id IN (
                    SELECT fields.id
                    FROM bible_graph_fields fields
                    INNER JOIN bible_graph_parts parts ON parts.id = fields.part_id
                    WHERE parts.node_id = ?2
                )
            )
            OR (
                object_kind = ?4
                AND object_id IN (
                    SELECT id
                    FROM bible_graph_edges
                    WHERE from_node_id = ?2 OR to_node_id = ?2
                )
            )",
        params![
            bible_node.as_str(),
            node_id.as_str(),
            bible_part_field.as_str(),
            bible_edge.as_str(),
        ],
        |row| row.get::<_, i64>(0),
    )?;
    let latest_change_event_id = conn
        .query_row(
            "SELECT change_event_id
             FROM object_revisions
             WHERE (object_kind = ?1 AND object_id = ?2)
                OR (
                    object_kind = ?3
                    AND object_id IN (
                        SELECT fields.id
                        FROM bible_graph_fields fields
                        INNER JOIN bible_graph_parts parts ON parts.id = fields.part_id
                        WHERE parts.node_id = ?2
                    )
                )
                OR (
                    object_kind = ?4
                    AND object_id IN (
                        SELECT id
                        FROM bible_graph_edges
                        WHERE from_node_id = ?2 OR to_node_id = ?2
                    )
                )
             ORDER BY rowid DESC
             LIMIT 1",
            params![bible_node, node_id.as_str(), bible_part_field, bible_edge],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|id| parse_uuid(&id).map(ChangeEventId))
        .transpose()?;

    Ok(history_store::RevisionSummary {
        revision_count: u64::try_from(revision_count).unwrap_or_default(),
        latest_change_event_id,
    })
}

fn row_to_node(row: &Row<'_>) -> Result<BibleGraphNode, rusqlite::Error> {
    let id: String = row.get(0)?;
    let parent_id: Option<String> = row.get(1)?;
    let schema_key: String = row.get(2)?;
    let sort_order: i64 = row.get(5)?;

    Ok(BibleGraphNode {
        id: BibleGraphNodeId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        parent_id: parent_id
            .map(BibleGraphNodeId::new)
            .transpose()
            .map_err(|e| conversion_failure(row, 1, e))?,
        schema_key: BibleGraphSchemaKey::new(schema_key)
            .map_err(|e| conversion_failure(row, 2, e))?,
        name: row.get(3)?,
        system_owned: row.get::<_, i64>(4)? != 0,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 5, e))?,
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

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    let value_type = row
        .get_ref(index)
        .map(|value| value.data_type())
        .unwrap_or(rusqlite::types::Type::Null);
    rusqlite::Error::FromSqlConversionFailure(index, value_type, Box::new(error))
}

#[cfg(test)]
#[path = "bible_graph_store_tests.rs"]
mod tests;
