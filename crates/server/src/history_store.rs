use eidetic_core::contracts::{
    ChangeEvent, CommandEnvelope, CommandId, FieldDelta, FieldValue, ObjectRevision,
    ObjectRevisionId,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
#[cfg(test)]
use serde::de::DeserializeOwned;

pub(crate) use crate::history_read_store::{
    RevisionSummary, load_change_review_changes, load_revision_summary_for_kind,
    load_revision_summary_for_kinds, load_revisions_for_object,
};

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

pub(crate) fn check_recorded_command<T>(
    conn: &Connection,
    command: &CommandEnvelope<T>,
    payload_type: &str,
) -> Result<Option<RecordChangeOutcome>, HistoryStoreError>
where
    T: Serialize,
{
    let Some(existing) = existing_command_signature(conn, command.id)? else {
        return Ok(None);
    };
    let payload_json = serde_json::to_string(&command.payload)?;
    if existing.payload_type == payload_type && existing.payload_json == payload_json {
        return Ok(Some(RecordChangeOutcome::AlreadyRecorded));
    }
    Err(HistoryStoreError::InvalidValue(
        "command id already exists with a different payload".to_string(),
    ))
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
