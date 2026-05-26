use eidetic_core::contracts::{
    AffectConfidence, AffectProjection, AffectTarget, AffectValue, AffectValueId, Arousal,
    ChangeEvent, ChangeEventKind, CommandEnvelope, DeleteAffectValueCommand, EmotionalIntensity,
    FieldDelta, FieldValue, MoodLabel, ObjectKind, ObjectRevision, ProjectionEnvelope,
    ProjectionVersion, RevisionOperation, SetAffectValueCommand, Valence,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const AFFECT_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS affect_values (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    target_kind TEXT NOT NULL CHECK (target_kind <> ''),
    target_id TEXT,
    valence_bp INTEGER NOT NULL,
    arousal_bp INTEGER NOT NULL,
    intensity_bp INTEGER NOT NULL,
    confidence_bp INTEGER NOT NULL,
    provenance TEXT NOT NULL CHECK (provenance <> ''),
    rationale TEXT,
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    updated_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_affect_values_target
    ON affect_values(target_kind, target_id, id);
CREATE INDEX IF NOT EXISTS idx_affect_values_deleted
    ON affect_values(deleted_event_id, id);

CREATE TABLE IF NOT EXISTS affect_mood_labels (
    affect_id TEXT NOT NULL REFERENCES affect_values(id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK (label <> ''),
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (affect_id, sort_order)
);
CREATE INDEX IF NOT EXISTS idx_affect_mood_labels_label
    ON affect_mood_labels(label, affect_id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(AFFECT_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_set_affect_value(
    conn: &mut Connection,
    command: &CommandEnvelope<SetAffectValueCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    command
        .payload
        .validate()
        .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "affect.set")? {
        return Ok(outcome);
    }

    let existing = load_affect_value(conn, command.payload.affect_id)?;
    let value = command_to_affect_value(&command.payload);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("set affect {}", value.id.0),
    )
    .with_created_at_ms(created_at_ms);
    let operation = if existing.is_some() {
        RevisionOperation::Update
    } else {
        RevisionOperation::Create
    };
    let revision = affect_revision(&value, event.id, operation)?;

    history_store::record_change_with(conn, command, "affect.set", &event, &[revision], |tx| {
        upsert_affect_value_in_transaction(tx, &value, event.id)
    })
}

pub(crate) fn record_delete_affect_value(
    conn: &mut Connection,
    command: &CommandEnvelope<DeleteAffectValueCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, "affect.delete")? {
        return Ok(outcome);
    }
    if load_affect_value(conn, command.payload.affect_id)?.is_none() {
        return Err(HistoryStoreError::InvalidValue(format!(
            "affect value not found: {}",
            command.payload.affect_id.0
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!("delete affect {}", command.payload.affect_id.0),
    )
    .with_created_at_ms(created_at_ms);
    let revision = ObjectRevision::new(
        ObjectKind::AffectValue,
        command.payload.affect_id.0.to_string(),
        event.id,
        RevisionOperation::Delete,
    );

    history_store::record_change_with(conn, command, "affect.delete", &event, &[revision], |tx| {
        tx.execute(
            "UPDATE affect_values SET deleted_event_id = ?2 WHERE id = ?1",
            params![
                command.payload.affect_id.0.to_string(),
                event.id.0.to_string(),
            ],
        )?;
        Ok(())
    })
}

pub(crate) fn load_affect_projection(
    conn: &Connection,
    target: AffectTarget,
) -> Result<ProjectionEnvelope<AffectProjection>, HistoryStoreError> {
    create_schema(conn)?;
    let (target_kind, target_id) = encode_target(&target)?;
    let mut statement = conn.prepare(
        "SELECT id, target_kind, target_id, valence_bp, arousal_bp, intensity_bp,
                confidence_bp, provenance, rationale
         FROM affect_values
         WHERE target_kind = ?1
           AND COALESCE(target_id, '') = COALESCE(?2, '')
           AND deleted_event_id IS NULL
         ORDER BY id ASC",
    )?;
    let rows = statement.query_map(params![target_kind, target_id], row_to_affect_value)?;
    let mut values = Vec::new();
    for row in rows {
        let mut value = row?;
        value.mood_labels = load_mood_labels(conn, value.id)?;
        values.push(value);
    }

    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::AffectValue)?;
    let projection = AffectProjection { target, values };
    Ok(match summary.latest_change_event_id {
        Some(change_event_id) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    })
}

pub(crate) fn load_affect_value(
    conn: &Connection,
    affect_id: AffectValueId,
) -> Result<Option<AffectValue>, HistoryStoreError> {
    create_schema(conn)?;
    let Some(mut value) = conn
        .query_row(
            "SELECT id, target_kind, target_id, valence_bp, arousal_bp, intensity_bp,
                    confidence_bp, provenance, rationale
             FROM affect_values
             WHERE id = ?1 AND deleted_event_id IS NULL",
            [affect_id.0.to_string()],
            row_to_affect_value,
        )
        .optional()?
    else {
        return Ok(None);
    };
    value.mood_labels = load_mood_labels(conn, value.id)?;
    Ok(Some(value))
}

fn command_to_affect_value(command: &SetAffectValueCommand) -> AffectValue {
    AffectValue {
        id: command.affect_id,
        target: command.target.clone(),
        valence: command.valence,
        arousal: command.arousal,
        intensity: command.intensity,
        confidence: command.confidence,
        mood_labels: command.mood_labels.clone(),
        provenance: command.provenance.clone(),
        rationale: command.rationale.clone(),
    }
}

fn upsert_affect_value_in_transaction(
    tx: &Transaction<'_>,
    value: &AffectValue,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let (target_kind, target_id) = encode_target(&value.target)?;
    tx.execute(
        "INSERT INTO affect_values (
            id, target_kind, target_id, valence_bp, arousal_bp, intensity_bp,
            confidence_bp, provenance, rationale, created_event_id, updated_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
         ON CONFLICT(id) DO UPDATE SET
            target_kind = excluded.target_kind,
            target_id = excluded.target_id,
            valence_bp = excluded.valence_bp,
            arousal_bp = excluded.arousal_bp,
            intensity_bp = excluded.intensity_bp,
            confidence_bp = excluded.confidence_bp,
            provenance = excluded.provenance,
            rationale = excluded.rationale,
            updated_event_id = excluded.updated_event_id,
            deleted_event_id = NULL",
        params![
            value.id.0.to_string(),
            target_kind,
            target_id,
            i64::from(value.valence.basis_points()),
            i64::from(value.arousal.basis_points()),
            i64::from(value.intensity.basis_points()),
            i64::from(value.confidence.basis_points()),
            encode_string_enum(&value.provenance)?,
            value.rationale,
            event_id.0.to_string(),
        ],
    )?;
    tx.execute(
        "DELETE FROM affect_mood_labels WHERE affect_id = ?1",
        [value.id.0.to_string()],
    )?;
    for (sort_order, label) in value.mood_labels.iter().enumerate() {
        tx.execute(
            "INSERT INTO affect_mood_labels (affect_id, label, sort_order)
             VALUES (?1, ?2, ?3)",
            params![value.id.0.to_string(), label.as_str(), sort_order as i64],
        )?;
    }
    Ok(())
}

fn affect_revision(
    value: &AffectValue,
    event_id: eidetic_core::contracts::ChangeEventId,
    operation: RevisionOperation,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::AffectValue,
        value.id.0.to_string(),
        event_id,
        operation,
    )
    .with_field(FieldDelta::new(
        "target",
        None,
        Some(FieldValue::Text(target_label(&value.target)?)),
    ))
    .with_field(FieldDelta::new(
        "valence_bp",
        None,
        Some(FieldValue::Integer(i64::from(value.valence.basis_points()))),
    ))
    .with_field(FieldDelta::new(
        "arousal_bp",
        None,
        Some(FieldValue::Integer(i64::from(value.arousal.basis_points()))),
    ))
    .with_field(FieldDelta::new(
        "intensity_bp",
        None,
        Some(FieldValue::Integer(i64::from(
            value.intensity.basis_points(),
        ))),
    ))
    .with_field(FieldDelta::new(
        "confidence_bp",
        None,
        Some(FieldValue::Integer(i64::from(
            value.confidence.basis_points(),
        ))),
    ))
    .with_field(FieldDelta::new(
        "provenance",
        None,
        Some(FieldValue::Text(encode_string_enum(&value.provenance)?)),
    )))
}

fn row_to_affect_value(row: &Row<'_>) -> Result<AffectValue, rusqlite::Error> {
    let id: String = row.get(0)?;
    let target_kind: String = row.get(1)?;
    let target_id: Option<String> = row.get(2)?;
    let provenance: String = row.get(7)?;
    Ok(AffectValue {
        id: AffectValueId(parse_uuid(&id, 0)?),
        target: decode_target(&target_kind, target_id.as_deref(), 1)?,
        valence: Valence::new(row.get::<_, i16>(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        arousal: Arousal::new(row.get::<_, u16>(4)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        intensity: EmotionalIntensity::new(row.get::<_, u16>(5)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        confidence: AffectConfidence::new(row.get::<_, u16>(6)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        mood_labels: Vec::new(),
        provenance: decode_string_enum(&provenance, 7)?,
        rationale: row.get(8)?,
    })
}

fn load_mood_labels(
    conn: &Connection,
    affect_id: AffectValueId,
) -> Result<Vec<MoodLabel>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT label
         FROM affect_mood_labels
         WHERE affect_id = ?1
         ORDER BY sort_order ASC",
    )?;
    let rows = statement.query_map([affect_id.0.to_string()], |row| {
        let label: String = row.get(0)?;
        MoodLabel::new(label).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    })?;
    let mut labels = Vec::new();
    for row in rows {
        labels.push(row?);
    }
    Ok(labels)
}

fn encode_target(
    target: &AffectTarget,
) -> Result<(&'static str, Option<String>), HistoryStoreError> {
    Ok(match target {
        AffectTarget::Project => ("project", None),
        AffectTarget::TimelineNode { node_id } => ("timeline_node", Some(node_id.0.to_string())),
        AffectTarget::ScriptSegment { segment_id } => {
            ("script_segment", Some(segment_id.as_str().to_string()))
        }
        AffectTarget::BibleNode { node_id } => ("bible_node", Some(node_id.as_str().to_string())),
        AffectTarget::BibleSnapshot { snapshot_id } => {
            ("bible_snapshot", Some(snapshot_id.as_str().to_string()))
        }
    })
}

fn decode_target(
    target_kind: &str,
    target_id: Option<&str>,
    column: usize,
) -> Result<AffectTarget, rusqlite::Error> {
    Ok(match target_kind {
        "project" => AffectTarget::Project,
        "timeline_node" => AffectTarget::TimelineNode {
            node_id: NodeId(parse_uuid(required_target_id(target_id, column)?, column)?),
        },
        "script_segment" => AffectTarget::ScriptSegment {
            segment_id: eidetic_core::contracts::ScriptSegmentId::new(required_target_id(
                target_id, column,
            )?)
            .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))?,
        },
        "bible_node" => AffectTarget::BibleNode {
            node_id: eidetic_core::contracts::BibleGraphNodeId::new(required_target_id(
                target_id, column,
            )?)
            .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))?,
        },
        "bible_snapshot" => AffectTarget::BibleSnapshot {
            snapshot_id: eidetic_core::contracts::BibleGraphSnapshotId::new(required_target_id(
                target_id, column,
            )?)
            .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))?,
        },
        _ => {
            return Err(conversion_error(
                column,
                rusqlite::types::Type::Text,
                format!("unknown affect target kind: {target_kind}"),
            ));
        }
    })
}

fn target_label(target: &AffectTarget) -> Result<String, HistoryStoreError> {
    let (kind, id) = encode_target(target)?;
    Ok(match id {
        Some(id) => format!("{kind}:{id}"),
        None => kind.to_string(),
    })
}

fn required_target_id<'a>(
    target_id: Option<&'a str>,
    column: usize,
) -> Result<&'a str, rusqlite::Error> {
    target_id.filter(|value| !value.is_empty()).ok_or_else(|| {
        conversion_error(
            column,
            rusqlite::types::Type::Text,
            "affect target id is required",
        )
    })
}

fn parse_uuid(value: &str, column: usize) -> Result<uuid::Uuid, rusqlite::Error> {
    uuid::Uuid::parse_str(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn encode_string_enum<T: serde::Serialize>(value: &T) -> Result<String, HistoryStoreError> {
    serde_json::to_string(value)
        .map(|value| value.trim_matches('"').to_string())
        .map_err(Into::into)
}

fn decode_string_enum<T>(value: &str, column: usize) -> Result<T, rusqlite::Error>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(&format!("\"{value}\""))
        .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))
}

fn conversion_error(
    column: usize,
    value_type: rusqlite::types::Type,
    error: impl ToString,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        value_type,
        Box::new(HistoryStoreError::InvalidValue(error.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{AffectProvenance, CommandId};

    #[test]
    fn affect_store_records_and_projects_timeline_node_values() {
        let mut conn = Connection::open_in_memory().unwrap();
        let target = AffectTarget::TimelineNode {
            node_id: NodeId::new(),
        };
        let command = set_command(target.clone(), "uneasy");

        let outcome =
            record_set_affect_value(&mut conn, &CommandEnvelope::new(command.clone()), 100)
                .unwrap();

        assert_eq!(outcome, RecordChangeOutcome::Recorded);
        let projection = load_affect_projection(&conn, target).unwrap();
        assert_eq!(projection.payload.values.len(), 1);
        assert_eq!(
            projection.payload.values[0].mood_labels[0].as_str(),
            "uneasy"
        );
        assert_eq!(projection.payload.values[0].valence.basis_points(), -250);
    }

    #[test]
    fn affect_store_is_idempotent_for_duplicate_commands() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(set_command(AffectTarget::Project, "hopeful"));

        assert_eq!(
            record_set_affect_value(&mut conn, &command, 100).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_set_affect_value(&mut conn, &command, 100).unwrap(),
            RecordChangeOutcome::AlreadyRecorded
        );
        let projection = load_affect_projection(&conn, AffectTarget::Project).unwrap();
        assert_eq!(projection.payload.values.len(), 1);
    }

    #[test]
    fn affect_store_soft_deletes_values_from_projection() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = set_command(AffectTarget::Project, "calm");
        let affect_id = command.affect_id;
        record_set_affect_value(&mut conn, &CommandEnvelope::new(command), 100).unwrap();

        record_delete_affect_value(
            &mut conn,
            &CommandEnvelope::new(DeleteAffectValueCommand {
                command_id: CommandId::new(),
                affect_id,
            }),
            101,
        )
        .unwrap();

        let projection = load_affect_projection(&conn, AffectTarget::Project).unwrap();
        assert!(projection.payload.values.is_empty());
    }

    fn set_command(target: AffectTarget, mood: &str) -> SetAffectValueCommand {
        SetAffectValueCommand {
            command_id: CommandId::new(),
            affect_id: AffectValueId::new(),
            target,
            valence: Valence::new(-250).unwrap(),
            arousal: Arousal::new(650).unwrap(),
            intensity: EmotionalIntensity::new(700).unwrap(),
            confidence: AffectConfidence::new(900).unwrap(),
            mood_labels: vec![MoodLabel::new(mood).unwrap()],
            provenance: AffectProvenance::UserAuthored,
            rationale: Some("Opening mood".to_string()),
        }
    }
}
