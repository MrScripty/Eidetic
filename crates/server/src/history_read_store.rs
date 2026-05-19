use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeReviewChange, CommandId, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, ObjectRevisionId,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::history_store::HistoryStoreError;

pub(crate) fn load_revisions_for_object(
    conn: &Connection,
    object_kind: ObjectKind,
    object_id: &str,
) -> Result<Vec<ObjectRevision>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, change_event_id, base_revision_id, operation
         FROM object_revisions
         WHERE object_kind = ?1 AND object_id = ?2
         ORDER BY sort_order ASC",
    )?;
    let rows = statement.query_map(
        params![encode_string_enum(&object_kind)?, object_id],
        |row| {
            Ok(RevisionRow {
                id: row.get(0)?,
                change_event_id: row.get(1)?,
                base_revision_id: row.get(2)?,
                operation: row.get(3)?,
            })
        },
    )?;

    let mut revisions = Vec::new();
    for row in rows {
        let row = row?;
        let revision_id = ObjectRevisionId(parse_uuid(&row.id)?);
        revisions.push(ObjectRevision {
            id: revision_id,
            object_kind: object_kind.clone(),
            object_id: object_id.to_string(),
            change_event_id: ChangeEventId(parse_uuid(&row.change_event_id)?),
            base_revision_id: row
                .base_revision_id
                .map(|id| parse_uuid(&id).map(ObjectRevisionId))
                .transpose()?,
            operation: decode_string_enum(&row.operation)?,
            fields: load_fields_for_revision(conn, revision_id)?,
        });
    }

    Ok(revisions)
}

pub(crate) fn load_change_review_changes(
    conn: &Connection,
    limit: u32,
) -> Result<Vec<ChangeReviewChange>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, command_id, kind, summary, created_at_ms
         FROM change_events
         ORDER BY created_at_ms DESC, rowid DESC
         LIMIT ?1",
    )?;
    let rows = statement.query_map([i64::from(limit)], |row| {
        Ok(ChangeEventRow {
            id: row.get(0)?,
            command_id: row.get(1)?,
            kind: row.get(2)?,
            summary: row.get(3)?,
            created_at_ms: row.get(4)?,
        })
    })?;

    let mut changes = Vec::new();
    for row in rows {
        let row = row?;
        let event = ChangeEvent {
            id: ChangeEventId(parse_uuid(&row.id)?),
            command_id: CommandId(parse_uuid(&row.command_id)?),
            kind: decode_string_enum(&row.kind)?,
            summary: row.summary,
            created_at_ms: u64::try_from(row.created_at_ms).unwrap_or_default(),
        };
        let revisions = load_revisions_for_event(conn, event.id)?;
        changes.push(ChangeReviewChange { event, revisions });
    }
    Ok(changes)
}

pub(crate) fn load_revision_summary_for_kind(
    conn: &Connection,
    object_kind: ObjectKind,
) -> Result<RevisionSummary, HistoryStoreError> {
    let encoded_kind = encode_string_enum(&object_kind)?;
    let revision_count = conn.query_row(
        "SELECT COUNT(*) FROM object_revisions WHERE object_kind = ?1",
        [encoded_kind.as_str()],
        |row| row.get::<_, i64>(0),
    )?;
    let latest_change_event_id = conn
        .query_row(
            "SELECT change_event_id
             FROM object_revisions
             WHERE object_kind = ?1
             ORDER BY rowid DESC
             LIMIT 1",
            [encoded_kind],
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

fn load_revisions_for_event(
    conn: &Connection,
    change_event_id: ChangeEventId,
) -> Result<Vec<ObjectRevision>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, object_kind, object_id, base_revision_id, operation
         FROM object_revisions
         WHERE change_event_id = ?1
         ORDER BY sort_order ASC",
    )?;
    let rows = statement.query_map([change_event_id.0.to_string()], |row| {
        Ok(EventRevisionRow {
            id: row.get(0)?,
            object_kind: row.get(1)?,
            object_id: row.get(2)?,
            base_revision_id: row.get(3)?,
            operation: row.get(4)?,
        })
    })?;

    let mut revisions = Vec::new();
    for row in rows {
        let row = row?;
        let revision_id = ObjectRevisionId(parse_uuid(&row.id)?);
        revisions.push(ObjectRevision {
            id: revision_id,
            object_kind: decode_string_enum(&row.object_kind)?,
            object_id: row.object_id,
            change_event_id,
            base_revision_id: row
                .base_revision_id
                .map(|id| parse_uuid(&id).map(ObjectRevisionId))
                .transpose()?,
            operation: decode_string_enum(&row.operation)?,
            fields: load_fields_for_revision(conn, revision_id)?,
        });
    }
    Ok(revisions)
}

fn load_fields_for_revision(
    conn: &Connection,
    revision_id: ObjectRevisionId,
) -> Result<Vec<FieldDelta>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT
            field_key, sort_order,
            old_type, old_text, old_integer, old_number, old_bool, old_ref_kind, old_ref_id, old_asset_ref,
            new_type, new_text, new_integer, new_number, new_bool, new_ref_kind, new_ref_id, new_asset_ref
         FROM object_revision_fields
         WHERE revision_id = ?1
         ORDER BY field_index ASC",
    )?;
    let rows = statement.query_map([revision_id.0.to_string()], |row| {
        Ok(FieldRow {
            field_key: row.get(0)?,
            sort_order: row.get(1)?,
            old_value: SqlFieldValue {
                value_type: row.get(2)?,
                text: row.get(3)?,
                integer: row.get(4)?,
                number: row.get(5)?,
                bool_value: row.get(6)?,
                ref_kind: row.get(7)?,
                ref_id: row.get(8)?,
                asset_ref: row.get(9)?,
            },
            new_value: SqlFieldValue {
                value_type: row.get(10)?,
                text: row.get(11)?,
                integer: row.get(12)?,
                number: row.get(13)?,
                bool_value: row.get(14)?,
                ref_kind: row.get(15)?,
                ref_id: row.get(16)?,
                asset_ref: row.get(17)?,
            },
        })
    })?;

    let mut fields = Vec::new();
    for row in rows {
        let row = row?;
        fields.push(FieldDelta {
            field_key: row.field_key,
            old_value: row.old_value.into_field_value()?,
            new_value: row.new_value.into_field_value()?,
            sort_order: row.sort_order,
        });
    }
    Ok(fields)
}

fn encode_string_enum<T>(value: &T) -> Result<String, HistoryStoreError>
where
    T: Serialize,
{
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected enum to serialize as string".to_string(),
        )),
    }
}

fn decode_string_enum<T>(value: &str) -> Result<T, HistoryStoreError>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(serde_json::Value::String(
        value.to_string(),
    ))?)
}

fn parse_uuid(value: &str) -> Result<uuid::Uuid, HistoryStoreError> {
    uuid::Uuid::parse_str(value).map_err(|e| HistoryStoreError::InvalidId(e.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RevisionSummary {
    pub revision_count: u64,
    pub latest_change_event_id: Option<ChangeEventId>,
}

#[derive(Debug)]
struct RevisionRow {
    id: String,
    change_event_id: String,
    base_revision_id: Option<String>,
    operation: String,
}

#[derive(Debug)]
struct ChangeEventRow {
    id: String,
    command_id: String,
    kind: String,
    summary: String,
    created_at_ms: i64,
}

#[derive(Debug)]
struct EventRevisionRow {
    id: String,
    object_kind: String,
    object_id: String,
    base_revision_id: Option<String>,
    operation: String,
}

#[derive(Debug)]
struct FieldRow {
    field_key: String,
    sort_order: u32,
    old_value: SqlFieldValue,
    new_value: SqlFieldValue,
}

#[derive(Debug)]
struct SqlFieldValue {
    value_type: Option<String>,
    text: Option<String>,
    integer: Option<i64>,
    number: Option<f64>,
    bool_value: Option<i64>,
    ref_kind: Option<String>,
    ref_id: Option<String>,
    asset_ref: Option<String>,
}

impl SqlFieldValue {
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
                kind: decode_string_enum(&required(self.ref_kind, "ref_kind")?)?,
                id: required(self.ref_id, "ref_id")?,
            })),
            "asset_ref" => Ok(Some(FieldValue::AssetRef(required(
                self.asset_ref,
                "asset_ref",
            )?))),
            other => Err(HistoryStoreError::InvalidValue(format!(
                "unknown field value type: {other}"
            ))),
        }
    }
}

fn required<T>(value: Option<T>, field_name: &'static str) -> Result<T, HistoryStoreError> {
    value.ok_or(HistoryStoreError::MissingColumn(field_name))
}
