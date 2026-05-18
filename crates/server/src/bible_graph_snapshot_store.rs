use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSnapshot,
    BibleGraphSnapshotField, BibleGraphSnapshotFieldId, BibleGraphSnapshotId,
    BibleGraphSnapshotProjection, ChangeEventId, SetBibleGraphSnapshotFieldCommand,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::bible_graph_value_store::SqlGraphFieldValue;
use crate::history_store::HistoryStoreError;

pub(crate) fn set_snapshot_field_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphSnapshotFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    if !node_exists_in_transaction(tx, &command.node_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "bible graph node does not exist: {}",
            command.node_id.as_str()
        )));
    }

    upsert_snapshot_in_transaction(tx, command, event_id)?;
    upsert_snapshot_field_in_transaction(tx, command, event_id)
}

pub(crate) fn load_snapshot_projections(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Vec<BibleGraphSnapshotProjection>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, node_id, at_ms, label, sort_order
         FROM bible_graph_snapshots
         WHERE node_id = ?1 AND deleted_event_id IS NULL
         ORDER BY at_ms ASC, sort_order ASC, label ASC, id ASC",
    )?;
    let rows = statement.query_map([node_id.as_str()], row_to_snapshot)?;

    let mut snapshots = Vec::new();
    for row in rows {
        let snapshot = row?;
        let fields = load_fields_for_snapshot(conn, &snapshot.id)?;
        snapshots.push(BibleGraphSnapshotProjection { snapshot, fields });
    }
    Ok(snapshots)
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

fn upsert_snapshot_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphSnapshotFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let existing_node_id = tx
        .query_row(
            "SELECT node_id FROM bible_graph_snapshots WHERE id = ?1 AND deleted_event_id IS NULL",
            [command.snapshot_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    if let Some(node_id) = existing_node_id {
        if node_id != command.node_id.as_str() {
            return Err(HistoryStoreError::InvalidValue(format!(
                "bible graph snapshot {} does not belong to node {}",
                command.snapshot_id.as_str(),
                command.node_id.as_str()
            )));
        }

        tx.execute(
            "UPDATE bible_graph_snapshots
             SET at_ms = ?2, label = ?3, sort_order = ?4, updated_event_id = ?5
             WHERE id = ?1",
            params![
                command.snapshot_id.as_str(),
                to_i64(command.at_ms, "at_ms")?,
                command.label,
                command.snapshot_sort_order as i64,
                event_id.0.to_string(),
            ],
        )?;
        return Ok(());
    }

    tx.execute(
        "INSERT INTO bible_graph_snapshots (
            id, node_id, at_ms, label, sort_order, created_event_id, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            command.snapshot_id.as_str(),
            command.node_id.as_str(),
            to_i64(command.at_ms, "at_ms")?,
            command.label,
            command.snapshot_sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn upsert_snapshot_field_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphSnapshotFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let existing_field = tx
        .query_row(
            "SELECT snapshot_id, part_key, field_key
             FROM bible_graph_snapshot_fields
             WHERE id = ?1 AND deleted_event_id IS NULL",
            [command.field_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;

    if let Some((snapshot_id, part_key, field_key)) = existing_field {
        if snapshot_id != command.snapshot_id.as_str() {
            return Err(HistoryStoreError::InvalidValue(format!(
                "bible graph snapshot field {} does not belong to snapshot {}",
                command.field_id.as_str(),
                command.snapshot_id.as_str()
            )));
        }
        if part_key != command.part_key.as_str() {
            return Err(HistoryStoreError::InvalidValue(format!(
                "bible graph snapshot field {} already has part key {}",
                command.field_id.as_str(),
                part_key
            )));
        }
        if field_key != command.field_key.as_str() {
            return Err(HistoryStoreError::InvalidValue(format!(
                "bible graph snapshot field {} already has field key {}",
                command.field_id.as_str(),
                field_key
            )));
        }
    }

    let value = SqlGraphFieldValue::from_field_value(command.value.as_ref())?;
    tx.execute(
        "INSERT INTO bible_graph_snapshot_fields (
            id, snapshot_id, part_key, part_name, field_key,
            value_type, text_value, integer_value, number_value, bool_value, ref_kind, ref_id, asset_ref,
            sort_order, updated_event_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
            ?14, ?15
         )
         ON CONFLICT(id) DO UPDATE SET
            snapshot_id = excluded.snapshot_id,
            part_name = excluded.part_name,
            value_type = excluded.value_type,
            text_value = excluded.text_value,
            integer_value = excluded.integer_value,
            number_value = excluded.number_value,
            bool_value = excluded.bool_value,
            ref_kind = excluded.ref_kind,
            ref_id = excluded.ref_id,
            asset_ref = excluded.asset_ref,
            sort_order = excluded.sort_order,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            command.field_id.as_str(),
            command.snapshot_id.as_str(),
            command.part_key.as_str(),
            command.part_name,
            command.field_key.as_str(),
            value.value_type,
            value.text,
            value.integer,
            value.number,
            value.bool_value,
            value.ref_kind,
            value.ref_id,
            value.asset_ref,
            command.field_sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn load_fields_for_snapshot(
    conn: &Connection,
    snapshot_id: &BibleGraphSnapshotId,
) -> Result<Vec<BibleGraphSnapshotField>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT
            id, snapshot_id, part_key, part_name, field_key,
            value_type, text_value, integer_value, number_value, bool_value, ref_kind, ref_id, asset_ref,
            sort_order
         FROM bible_graph_snapshot_fields
         WHERE snapshot_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, part_key ASC, field_key ASC, id ASC",
    )?;
    let rows = statement.query_map([snapshot_id.as_str()], row_to_snapshot_field)?;

    let mut fields = Vec::new();
    for row in rows {
        fields.push(row?);
    }
    Ok(fields)
}

fn row_to_snapshot(row: &Row<'_>) -> Result<BibleGraphSnapshot, rusqlite::Error> {
    let id: String = row.get(0)?;
    let node_id: String = row.get(1)?;
    let at_ms: i64 = row.get(2)?;
    let sort_order: i64 = row.get(4)?;

    Ok(BibleGraphSnapshot {
        id: BibleGraphSnapshotId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        node_id: BibleGraphNodeId::new(node_id).map_err(|e| conversion_failure(row, 1, e))?,
        at_ms: u64::try_from(at_ms).map_err(|e| conversion_failure(row, 2, e))?,
        label: row.get(3)?,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 4, e))?,
    })
}

fn row_to_snapshot_field(row: &Row<'_>) -> Result<BibleGraphSnapshotField, rusqlite::Error> {
    let id: String = row.get(0)?;
    let snapshot_id: String = row.get(1)?;
    let part_key: String = row.get(2)?;
    let field_key: String = row.get(4)?;
    let sort_order: i64 = row.get(13)?;
    let value = SqlGraphFieldValue {
        value_type: row.get(5)?,
        text: row.get(6)?,
        integer: row.get(7)?,
        number: row.get(8)?,
        bool_value: row.get(9)?,
        ref_kind: row.get(10)?,
        ref_id: row.get(11)?,
        asset_ref: row.get(12)?,
    }
    .into_field_value()
    .map_err(|e| conversion_failure(row, 5, e))?;

    Ok(BibleGraphSnapshotField {
        id: BibleGraphSnapshotFieldId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        snapshot_id: BibleGraphSnapshotId::new(snapshot_id)
            .map_err(|e| conversion_failure(row, 1, e))?,
        part_key: BibleGraphPartKey::new(part_key).map_err(|e| conversion_failure(row, 2, e))?,
        part_name: row.get(3)?,
        field_key: BibleGraphFieldKey::new(field_key).map_err(|e| conversion_failure(row, 4, e))?,
        value,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 13, e))?,
    })
}

fn to_i64(value: u64, field_name: &'static str) -> Result<i64, HistoryStoreError> {
    i64::try_from(value).map_err(|_| {
        HistoryStoreError::InvalidValue(format!("{field_name} is too large for sqlite integer"))
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
