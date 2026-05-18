use eidetic_core::contracts::{
    BibleGraphNode, BibleGraphNodeId, BibleGraphNodeListProjection, BibleGraphSchemaKey,
    BibleNodeDetailProjection, ChangeEventId, ObjectKind, ProjectionEnvelope, ProjectionVersion,
    canonical_bible_root_nodes,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::history_store::{self, HistoryStoreError};

const BIBLE_GRAPH_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS bible_graph_nodes (
    id               TEXT PRIMARY KEY CHECK (id <> ''),
    parent_id        TEXT,
    schema_key       TEXT NOT NULL CHECK (schema_key <> ''),
    name             TEXT NOT NULL CHECK (name <> ''),
    system_owned     INTEGER NOT NULL CHECK (system_owned IN (0, 1)),
    sort_order       INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_bible_graph_nodes_parent
    ON bible_graph_nodes(parent_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_bible_graph_nodes_schema
    ON bible_graph_nodes(schema_key);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(BIBLE_GRAPH_SCHEMA_SQL)?;
    Ok(())
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

    Ok(node.map(|node| BibleNodeDetailProjection {
        node,
        parts: Vec::new(),
        incoming_edges: Vec::new(),
        outgoing_edges: Vec::new(),
    }))
}

pub(crate) fn load_node_detail_projection_envelope(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Option<ProjectionEnvelope<BibleNodeDetailProjection>>, HistoryStoreError> {
    let Some(projection) = load_node_detail_projection(conn, node_id)? else {
        return Ok(None);
    };
    let revisions =
        history_store::load_revisions_for_object(conn, ObjectKind::BibleNode, node_id.as_str())?;

    match revisions.last() {
        Some(revision) => Ok(Some(ProjectionEnvelope::from_event(
            ProjectionVersion(revisions.len() as u64 + 1),
            revision.change_event_id,
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

fn node_exists(conn: &Connection, node_id: &BibleGraphNodeId) -> Result<bool, HistoryStoreError> {
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
