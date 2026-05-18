use eidetic_core::contracts::{
    BibleGraphField, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPart,
    BibleGraphPartId, BibleGraphPartKey, BibleGraphPartProjection, ChangeEventId, FieldValue,
    ObjectKind, SetBibleGraphFieldCommand,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::history_store::HistoryStoreError;

pub(crate) fn set_field_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    if !node_exists_in_transaction(tx, &command.node_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "bible graph node does not exist: {}",
            command.node_id.as_str()
        )));
    }

    upsert_part_in_transaction(tx, command, event_id)?;
    upsert_field_in_transaction(tx, command, event_id)
}

pub(crate) fn load_part_projections(
    conn: &Connection,
    node_id: &BibleGraphNodeId,
) -> Result<Vec<BibleGraphPartProjection>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, node_id, part_key, name, system_owned, sort_order
         FROM bible_graph_parts
         WHERE node_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, name ASC, id ASC",
    )?;
    let rows = statement.query_map([node_id.as_str()], row_to_part)?;

    let mut parts = Vec::new();
    for row in rows {
        let part = row?;
        let fields = load_fields_for_part(conn, &part.id)?;
        parts.push(BibleGraphPartProjection { part, fields });
    }
    Ok(parts)
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

fn upsert_part_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let existing_node_id = tx
        .query_row(
            "SELECT node_id FROM bible_graph_parts WHERE id = ?1 AND deleted_event_id IS NULL",
            [command.part_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    match existing_node_id {
        Some(node_id) if node_id != command.node_id.as_str() => {
            Err(HistoryStoreError::InvalidValue(format!(
                "bible graph part {} does not belong to node {}",
                command.part_id.as_str(),
                command.node_id.as_str()
            )))
        }
        Some(_) => {
            tx.execute(
                "UPDATE bible_graph_parts
                 SET part_key = ?2, name = ?3, sort_order = ?4
                 WHERE id = ?1",
                params![
                    command.part_id.as_str(),
                    command.part_key.as_str(),
                    command.part_name,
                    command.part_sort_order as i64,
                ],
            )?;
            Ok(())
        }
        None => {
            tx.execute(
                "INSERT INTO bible_graph_parts (
                    id, node_id, part_key, name, system_owned, sort_order, created_event_id
                 ) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
                params![
                    command.part_id.as_str(),
                    command.node_id.as_str(),
                    command.part_key.as_str(),
                    command.part_name,
                    command.part_sort_order as i64,
                    event_id.0.to_string()
                ],
            )?;
            Ok(())
        }
    }
}

fn upsert_field_in_transaction(
    tx: &Transaction<'_>,
    command: &SetBibleGraphFieldCommand,
    event_id: ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let existing_part_id = tx
        .query_row(
            "SELECT part_id FROM bible_graph_fields WHERE id = ?1 AND deleted_event_id IS NULL",
            [command.field_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    if let Some(part_id) = existing_part_id {
        if part_id != command.part_id.as_str() {
            return Err(HistoryStoreError::InvalidValue(format!(
                "bible graph field {} does not belong to part {}",
                command.field_id.as_str(),
                command.part_id.as_str()
            )));
        }
    }

    let value = SqlGraphFieldValue::from_field_value(command.value.as_ref())?;
    tx.execute(
        "INSERT INTO bible_graph_fields (
            id, part_id, field_key,
            value_type, text_value, integer_value, number_value, bool_value, ref_kind, ref_id, asset_ref,
            sort_order, updated_event_id
         ) VALUES (
            ?1, ?2, ?3,
            ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13
         )
         ON CONFLICT(id) DO UPDATE SET
            part_id = excluded.part_id,
            field_key = excluded.field_key,
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
            command.part_id.as_str(),
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

fn load_fields_for_part(
    conn: &Connection,
    part_id: &BibleGraphPartId,
) -> Result<Vec<BibleGraphField>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT
            id, part_id, field_key,
            value_type, text_value, integer_value, number_value, bool_value, ref_kind, ref_id, asset_ref,
            sort_order
         FROM bible_graph_fields
         WHERE part_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, field_key ASC, id ASC",
    )?;
    let rows = statement.query_map([part_id.as_str()], row_to_field)?;

    let mut fields = Vec::new();
    for row in rows {
        fields.push(row?);
    }
    Ok(fields)
}

fn row_to_part(row: &Row<'_>) -> Result<BibleGraphPart, rusqlite::Error> {
    let id: String = row.get(0)?;
    let node_id: String = row.get(1)?;
    let part_key: String = row.get(2)?;
    let sort_order: i64 = row.get(5)?;

    Ok(BibleGraphPart {
        id: BibleGraphPartId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        node_id: BibleGraphNodeId::new(node_id).map_err(|e| conversion_failure(row, 1, e))?,
        part_key: BibleGraphPartKey::new(part_key).map_err(|e| conversion_failure(row, 2, e))?,
        name: row.get(3)?,
        system_owned: row.get::<_, i64>(4)? != 0,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 5, e))?,
    })
}

fn row_to_field(row: &Row<'_>) -> Result<BibleGraphField, rusqlite::Error> {
    let id: String = row.get(0)?;
    let part_id: String = row.get(1)?;
    let field_key: String = row.get(2)?;
    let sort_order: i64 = row.get(11)?;
    let value = SqlGraphFieldValue {
        value_type: row.get(3)?,
        text: row.get(4)?,
        integer: row.get(5)?,
        number: row.get(6)?,
        bool_value: row.get(7)?,
        ref_kind: row.get(8)?,
        ref_id: row.get(9)?,
        asset_ref: row.get(10)?,
    }
    .into_field_value()
    .map_err(|e| conversion_failure(row, 3, e))?;

    Ok(BibleGraphField {
        id: BibleGraphFieldId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        part_id: BibleGraphPartId::new(part_id).map_err(|e| conversion_failure(row, 1, e))?,
        field_key: BibleGraphFieldKey::new(field_key).map_err(|e| conversion_failure(row, 2, e))?,
        value,
        sort_order: u32::try_from(sort_order).map_err(|e| conversion_failure(row, 11, e))?,
    })
}

#[derive(Debug)]
struct SqlGraphFieldValue {
    value_type: Option<String>,
    text: Option<String>,
    integer: Option<i64>,
    number: Option<f64>,
    bool_value: Option<i64>,
    ref_kind: Option<String>,
    ref_id: Option<String>,
    asset_ref: Option<String>,
}

impl SqlGraphFieldValue {
    fn none() -> Self {
        Self {
            value_type: None,
            text: None,
            integer: None,
            number: None,
            bool_value: None,
            ref_kind: None,
            ref_id: None,
            asset_ref: None,
        }
    }

    fn from_field_value(value: Option<&FieldValue>) -> Result<Self, HistoryStoreError> {
        let Some(value) = value else {
            return Ok(Self::none());
        };

        let mut stored = Self::none();
        match value {
            FieldValue::Text(value) => {
                stored.value_type = Some("text".to_string());
                stored.text = Some(value.clone());
            }
            FieldValue::Integer(value) => {
                stored.value_type = Some("integer".to_string());
                stored.integer = Some(*value);
            }
            FieldValue::Number(value) => {
                stored.value_type = Some("number".to_string());
                stored.number = Some(*value);
            }
            FieldValue::Bool(value) => {
                stored.value_type = Some("bool".to_string());
                stored.bool_value = Some(if *value { 1 } else { 0 });
            }
            FieldValue::ObjectRef { kind, id } => {
                stored.value_type = Some("object_ref".to_string());
                stored.ref_kind = Some(encode_object_kind(kind)?);
                stored.ref_id = Some(id.clone());
            }
            FieldValue::AssetRef(value) => {
                stored.value_type = Some("asset_ref".to_string());
                stored.asset_ref = Some(value.clone());
            }
        }
        Ok(stored)
    }

    fn into_field_value(self) -> Result<Option<FieldValue>, HistoryStoreError> {
        let Some(value_type) = self.value_type else {
            return Ok(None);
        };

        match value_type.as_str() {
            "text" => Ok(Some(FieldValue::Text(required(self.text, "text")?))),
            "integer" => Ok(Some(FieldValue::Integer(required(
                self.integer,
                "integer",
            )?))),
            "number" => Ok(Some(FieldValue::Number(required(self.number, "number")?))),
            "bool" => Ok(Some(FieldValue::Bool(
                required(self.bool_value, "bool")? != 0,
            ))),
            "object_ref" => Ok(Some(FieldValue::ObjectRef {
                kind: decode_object_kind(&required(self.ref_kind, "ref_kind")?)?,
                id: required(self.ref_id, "ref_id")?,
            })),
            "asset_ref" => Ok(Some(FieldValue::AssetRef(required(
                self.asset_ref,
                "asset_ref",
            )?))),
            other => Err(HistoryStoreError::InvalidValue(format!(
                "unknown graph field value type: {other}"
            ))),
        }
    }
}

fn required<T>(value: Option<T>, field_name: &'static str) -> Result<T, HistoryStoreError> {
    value.ok_or(HistoryStoreError::MissingColumn(field_name))
}

fn encode_object_kind(value: &ObjectKind) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected object kind to serialize as string".to_string(),
        )),
    }
}

fn decode_object_kind(value: &str) -> Result<ObjectKind, HistoryStoreError> {
    Ok(serde_json::from_value(serde_json::Value::String(
        value.to_string(),
    ))?)
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
