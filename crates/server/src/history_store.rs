use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, CommandEnvelope, CommandId, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, ObjectRevisionId,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

const HISTORY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS commands (
    id            TEXT PRIMARY KEY,
    payload_type  TEXT NOT NULL,
    payload_json  TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS change_events (
    id            TEXT PRIMARY KEY,
    command_id    TEXT NOT NULL REFERENCES commands(id),
    kind          TEXT NOT NULL,
    summary       TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_change_events_command ON change_events(command_id);

CREATE TABLE IF NOT EXISTS object_revisions (
    id               TEXT PRIMARY KEY,
    object_kind      TEXT NOT NULL,
    object_id        TEXT NOT NULL CHECK (object_id <> ''),
    change_event_id  TEXT NOT NULL REFERENCES change_events(id),
    base_revision_id TEXT REFERENCES object_revisions(id),
    operation        TEXT NOT NULL,
    sort_order       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_object_revisions_object
    ON object_revisions(object_kind, object_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_object_revisions_event
    ON object_revisions(change_event_id, sort_order);

CREATE TABLE IF NOT EXISTS object_revision_fields (
    revision_id        TEXT NOT NULL REFERENCES object_revisions(id) ON DELETE CASCADE,
    field_index        INTEGER NOT NULL,
    field_key          TEXT NOT NULL CHECK (field_key <> ''),
    sort_order         INTEGER NOT NULL DEFAULT 0,
    old_type           TEXT,
    old_text           TEXT,
    old_integer        INTEGER,
    old_number         REAL,
    old_bool           INTEGER CHECK (old_bool IS NULL OR old_bool IN (0, 1)),
    old_ref_kind       TEXT,
    old_ref_id         TEXT,
    old_asset_ref      TEXT,
    new_type           TEXT,
    new_text           TEXT,
    new_integer        INTEGER,
    new_number         REAL,
    new_bool           INTEGER CHECK (new_bool IS NULL OR new_bool IN (0, 1)),
    new_ref_kind       TEXT,
    new_ref_id         TEXT,
    new_asset_ref      TEXT,
    PRIMARY KEY (revision_id, field_index)
);
CREATE INDEX IF NOT EXISTS idx_object_revision_fields_key
    ON object_revision_fields(field_key);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    conn.execute_batch(HISTORY_SCHEMA_SQL)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecordChangeOutcome {
    Recorded,
    AlreadyRecorded,
}

pub(crate) fn record_change<T>(
    conn: &mut Connection,
    command: &CommandEnvelope<T>,
    payload_type: &str,
    event: &ChangeEvent,
    revisions: &[ObjectRevision],
) -> Result<RecordChangeOutcome, HistoryStoreError>
where
    T: Serialize,
{
    record_change_with(conn, command, payload_type, event, revisions, |_| Ok(()))
}

pub(crate) fn record_change_with<T, F>(
    conn: &mut Connection,
    command: &CommandEnvelope<T>,
    payload_type: &str,
    event: &ChangeEvent,
    revisions: &[ObjectRevision],
    apply_current_state: F,
) -> Result<RecordChangeOutcome, HistoryStoreError>
where
    T: Serialize,
    F: FnOnce(&Transaction<'_>) -> Result<(), HistoryStoreError>,
{
    let payload_json = serde_json::to_string(&command.payload)?;
    let tx = conn.transaction()?;
    if let Some(existing) = existing_command_signature(&tx, command.id)? {
        if existing.payload_type == payload_type && existing.payload_json == payload_json {
            return Ok(RecordChangeOutcome::AlreadyRecorded);
        }
        return Err(HistoryStoreError::InvalidValue(
            "command id already exists with a different payload".to_string(),
        ));
    }

    tx.execute(
        "INSERT INTO commands (id, payload_type, payload_json, created_at_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            command.id.0.to_string(),
            payload_type,
            payload_json,
            event.created_at_ms
        ],
    )?;

    tx.execute(
        "INSERT INTO change_events (id, command_id, kind, summary, created_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            event.id.0.to_string(),
            event.command_id.0.to_string(),
            encode_string_enum(&event.kind)?,
            event.summary,
            event.created_at_ms
        ],
    )?;

    for (revision_index, revision) in revisions.iter().enumerate() {
        tx.execute(
            "INSERT INTO object_revisions (
                id, object_kind, object_id, change_event_id, base_revision_id, operation, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                revision.id.0.to_string(),
                encode_string_enum(&revision.object_kind)?,
                revision.object_id,
                revision.change_event_id.0.to_string(),
                revision.base_revision_id.map(|id| id.0.to_string()),
                encode_string_enum(&revision.operation)?,
                revision_index as i64
            ],
        )?;

        for (field_index, field) in revision.fields.iter().enumerate() {
            insert_field_delta(&tx, revision.id, field_index, field)?;
        }
    }

    apply_current_state(&tx)?;
    tx.commit()?;
    Ok(RecordChangeOutcome::Recorded)
}

#[cfg(test)]
pub(crate) fn load_command<T>(
    conn: &Connection,
    command_id: CommandId,
) -> Result<Option<CommandEnvelope<T>>, HistoryStoreError>
where
    T: DeserializeOwned,
{
    conn.query_row(
        "SELECT payload_json FROM commands WHERE id = ?1",
        [command_id.0.to_string()],
        |row| row.get::<_, String>(0),
    )
    .optional()?
    .map(|payload_json| {
        let payload = serde_json::from_str(&payload_json)?;
        Ok(CommandEnvelope {
            id: command_id,
            payload,
        })
    })
    .transpose()
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RevisionSummary {
    pub revision_count: u64,
    pub latest_change_event_id: Option<ChangeEventId>,
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

fn existing_command_signature(
    conn: &Connection,
    command_id: CommandId,
) -> Result<Option<CommandSignature>, rusqlite::Error> {
    conn.query_row(
        "SELECT payload_type, payload_json FROM commands WHERE id = ?1",
        [command_id.0.to_string()],
        |row| {
            Ok(CommandSignature {
                payload_type: row.get(0)?,
                payload_json: row.get(1)?,
            })
        },
    )
    .optional()
}

fn insert_field_delta(
    conn: &Connection,
    revision_id: ObjectRevisionId,
    field_index: usize,
    field: &FieldDelta,
) -> Result<(), HistoryStoreError> {
    let old_value = SqlFieldValue::from_field_value(field.old_value.as_ref())?;
    let new_value = SqlFieldValue::from_field_value(field.new_value.as_ref())?;
    conn.execute(
        "INSERT INTO object_revision_fields (
            revision_id, field_index, field_key, sort_order,
            old_type, old_text, old_integer, old_number, old_bool, old_ref_kind, old_ref_id, old_asset_ref,
            new_type, new_text, new_integer, new_number, new_bool, new_ref_kind, new_ref_id, new_asset_ref
         ) VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
         )",
        params![
            revision_id.0.to_string(),
            field_index as i64,
            field.field_key,
            field.sort_order,
            old_value.value_type,
            old_value.text,
            old_value.integer,
            old_value.number,
            old_value.bool_value,
            old_value.ref_kind,
            old_value.ref_id,
            old_value.asset_ref,
            new_value.value_type,
            new_value.text,
            new_value.integer,
            new_value.number,
            new_value.bool_value,
            new_value.ref_kind,
            new_value.ref_id,
            new_value.asset_ref
        ],
    )?;
    Ok(())
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

#[derive(Debug)]
struct RevisionRow {
    id: String,
    change_event_id: String,
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
struct CommandSignature {
    payload_type: String,
    payload_json: String,
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
                stored.bool_value = Some(i64::from(*value));
            }
            FieldValue::ObjectRef { kind, id } => {
                stored.value_type = Some("object_ref".to_string());
                stored.ref_kind = Some(encode_string_enum(kind)?);
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

#[derive(Debug, thiserror::Error)]
pub(crate) enum HistoryStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid id: {0}")]
    InvalidId(String),
    #[error("invalid value: {0}")]
    InvalidValue(String),
    #[error("missing required column for {0}")]
    MissingColumn(&'static str),
}

#[cfg(test)]
#[path = "history_store_tests.rs"]
mod tests;
