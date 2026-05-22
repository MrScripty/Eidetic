use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, ContextEvaluation, ContextEvaluationId,
    ContextInfluenceId, ContextInfluenceProjection, ContextInfluenceRecord, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, ProjectionEnvelope, ProjectionVersion,
    RecordContextEvaluationCommand, RevisionOperation,
};
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const CONTEXT_INFLUENCE_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS context_evaluations (
    id                TEXT PRIMARY KEY CHECK (id <> ''),
    target_node_id    TEXT NOT NULL CHECK (target_node_id <> ''),
    task_kind         TEXT NOT NULL CHECK (task_kind <> ''),
    summary           TEXT NOT NULL,
    distilled_context TEXT,
    created_at_ms     INTEGER NOT NULL,
    created_event_id  TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id  TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_context_evaluations_target
    ON context_evaluations(target_node_id, created_at_ms, id);

CREATE TABLE IF NOT EXISTS context_influence_records (
    id                    TEXT PRIMARY KEY CHECK (id <> ''),
    evaluation_id         TEXT NOT NULL REFERENCES context_evaluations(id),
    timeline_node_id      TEXT NOT NULL CHECK (timeline_node_id <> ''),
    source_layer          TEXT NOT NULL CHECK (source_layer <> ''),
    influence_kind        TEXT NOT NULL CHECK (influence_kind <> ''),
    confidence            REAL NOT NULL,
    reason                TEXT NOT NULL,
    provenance            TEXT NOT NULL CHECK (provenance <> ''),
    bible_node_id         TEXT,
    bible_edge_id         TEXT,
    introduced_by_node_id TEXT,
    sort_order            INTEGER NOT NULL,
    created_event_id      TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id      TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_context_influence_records_evaluation
    ON context_influence_records(evaluation_id, sort_order, id);
CREATE INDEX IF NOT EXISTS idx_context_influence_records_bible_node
    ON context_influence_records(bible_node_id, evaluation_id);
CREATE INDEX IF NOT EXISTS idx_context_influence_records_bible_edge
    ON context_influence_records(bible_edge_id, evaluation_id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(CONTEXT_INFLUENCE_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_context_evaluation(
    conn: &mut Connection,
    command: &CommandEnvelope<RecordContextEvaluationCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "context.evaluation_record")?
    {
        return Ok(outcome);
    }
    validate_command(&command.payload)?;
    if evaluation_exists(conn, command.payload.evaluation.id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "context evaluation already exists: {}",
            command.payload.evaluation.id.0
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!(
            "record context evaluation {}",
            command.payload.evaluation.id.0
        ),
    )
    .with_created_at_ms(created_at_ms);
    let revisions = revisions_for_command(&command.payload, event.id)?;

    history_store::record_change_with(
        conn,
        command,
        "context.evaluation_record",
        &event,
        &revisions,
        |tx| insert_command_in_transaction(tx, &command.payload, event.id),
    )
}

pub(crate) fn load_context_influence_projection(
    conn: &Connection,
    target_node_id: NodeId,
) -> Result<Option<ProjectionEnvelope<ContextInfluenceProjection>>, HistoryStoreError> {
    create_schema(conn)?;
    let Some(evaluation) = load_latest_evaluation(conn, target_node_id)? else {
        return Ok(None);
    };
    let records = load_influence_records(conn, evaluation.id)?;
    let summary = history_store::load_revision_summary_for_kinds(
        conn,
        &[ObjectKind::ContextEvaluation, ObjectKind::ContextInfluence],
    )?;
    let projection = ContextInfluenceProjection {
        target_node_id: evaluation.target_node_id,
        evaluation_id: evaluation.id,
        task_kind: evaluation.task_kind,
        distilled_context: evaluation.distilled_context,
        records,
    };

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(Some(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ))),
        None => Ok(Some(ProjectionEnvelope::initial(projection))),
    }
}

pub(crate) fn load_latest_context_influence_records(
    conn: &Connection,
    target_node_id: NodeId,
) -> Result<Vec<ContextInfluenceRecord>, HistoryStoreError> {
    create_schema(conn)?;
    let Some(evaluation) = load_latest_evaluation(conn, target_node_id)? else {
        return Ok(Vec::new());
    };
    load_influence_records(conn, evaluation.id)
}

fn validate_command(command: &RecordContextEvaluationCommand) -> Result<(), HistoryStoreError> {
    for record in &command.influences {
        if record.evaluation_id != command.evaluation.id {
            return Err(HistoryStoreError::InvalidValue(format!(
                "context influence {} references a different evaluation",
                record.id.0
            )));
        }
        if !(0.0..=1.0).contains(&record.confidence) {
            return Err(HistoryStoreError::InvalidValue(
                "context influence confidence must be between 0 and 1".to_string(),
            ));
        }
    }
    Ok(())
}

fn evaluation_exists(
    conn: &Connection,
    evaluation_id: ContextEvaluationId,
) -> Result<bool, HistoryStoreError> {
    conn.query_row(
        "SELECT 1 FROM context_evaluations WHERE id = ?1 AND deleted_event_id IS NULL",
        [evaluation_id.0.to_string()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn insert_command_in_transaction(
    tx: &Transaction<'_>,
    command: &RecordContextEvaluationCommand,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    insert_evaluation_in_transaction(tx, &command.evaluation, event_id)?;
    for record in &command.influences {
        insert_influence_record_in_transaction(tx, record, event_id)?;
    }
    Ok(())
}

fn insert_evaluation_in_transaction(
    tx: &Transaction<'_>,
    evaluation: &ContextEvaluation,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO context_evaluations (
            id, target_node_id, task_kind, summary, distilled_context, created_at_ms,
            created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            evaluation.id.0.to_string(),
            evaluation.target_node_id.0.to_string(),
            encode_string_enum(&evaluation.task_kind)?,
            evaluation.summary,
            evaluation.distilled_context,
            evaluation.created_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn insert_influence_record_in_transaction(
    tx: &Transaction<'_>,
    record: &ContextInfluenceRecord,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO context_influence_records (
            id, evaluation_id, timeline_node_id, source_layer, influence_kind, confidence,
            reason, provenance, bible_node_id, bible_edge_id, introduced_by_node_id,
            sort_order, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            record.id.0.to_string(),
            record.evaluation_id.0.to_string(),
            record.timeline_node_id.0.to_string(),
            encode_story_level(record.source_layer),
            encode_string_enum(&record.influence_kind)?,
            f64::from(record.confidence),
            record.reason,
            encode_string_enum(&record.provenance)?,
            record.bible_node_id.as_ref().map(|id| id.as_str()),
            record.bible_edge_id.as_ref().map(|id| id.as_str()),
            record
                .introduced_by_node_id
                .as_ref()
                .map(|id| id.0.to_string()),
            record.sort_order as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn load_latest_evaluation(
    conn: &Connection,
    target_node_id: NodeId,
) -> Result<Option<ContextEvaluation>, HistoryStoreError> {
    conn.query_row(
        "SELECT id, target_node_id, task_kind, summary, distilled_context, created_at_ms
         FROM context_evaluations
         WHERE target_node_id = ?1 AND deleted_event_id IS NULL
         ORDER BY created_at_ms DESC, id DESC
         LIMIT 1",
        [target_node_id.0.to_string()],
        row_to_evaluation,
    )
    .optional()
    .map_err(HistoryStoreError::from)
}

fn load_influence_records(
    conn: &Connection,
    evaluation_id: ContextEvaluationId,
) -> Result<Vec<ContextInfluenceRecord>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT id, evaluation_id, timeline_node_id, source_layer, influence_kind, confidence,
            reason, provenance, bible_node_id, bible_edge_id, introduced_by_node_id, sort_order
         FROM context_influence_records
         WHERE evaluation_id = ?1 AND deleted_event_id IS NULL
         ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = statement.query_map([evaluation_id.0.to_string()], row_to_influence_record)?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

fn revisions_for_command(
    command: &RecordContextEvaluationCommand,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<Vec<ObjectRevision>, HistoryStoreError> {
    let mut revisions = vec![
        ObjectRevision::new(
            ObjectKind::ContextEvaluation,
            command.evaluation.id.0.to_string(),
            event_id,
            RevisionOperation::Create,
        )
        .with_field(FieldDelta::new(
            "task_kind",
            None,
            Some(FieldValue::Text(encode_string_enum(
                &command.evaluation.task_kind,
            )?)),
        ))
        .with_field(FieldDelta::new(
            "summary",
            None,
            Some(FieldValue::Text(command.evaluation.summary.clone())),
        )),
    ];

    for record in &command.influences {
        revisions.push(
            ObjectRevision::new(
                ObjectKind::ContextInfluence,
                record.id.0.to_string(),
                event_id,
                RevisionOperation::Create,
            )
            .with_field(FieldDelta::new(
                "influence_kind",
                None,
                Some(FieldValue::Text(encode_string_enum(
                    &record.influence_kind,
                )?)),
            ))
            .with_field(FieldDelta::new(
                "reason",
                None,
                Some(FieldValue::Text(record.reason.clone())),
            ))
            .with_field(FieldDelta::new(
                "confidence",
                None,
                Some(FieldValue::Number(f64::from(record.confidence))),
            )),
        );
    }
    Ok(revisions)
}

fn row_to_evaluation(row: &Row<'_>) -> Result<ContextEvaluation, rusqlite::Error> {
    let id: String = row.get(0)?;
    let target_node_id: String = row.get(1)?;
    let task_kind: String = row.get(2)?;
    let created_at_ms: i64 = row.get(5)?;

    Ok(ContextEvaluation {
        id: ContextEvaluationId(parse_uuid(row, 0, &id)?),
        target_node_id: NodeId(parse_uuid(row, 1, &target_node_id)?),
        task_kind: decode_string_enum(row, 2, &task_kind)?,
        summary: row.get(3)?,
        distilled_context: row.get(4)?,
        created_at_ms: u64::try_from(created_at_ms)
            .map_err(|error| conversion_failure(row, 5, error))?,
    })
}

fn row_to_influence_record(row: &Row<'_>) -> Result<ContextInfluenceRecord, rusqlite::Error> {
    let id: String = row.get(0)?;
    let evaluation_id: String = row.get(1)?;
    let timeline_node_id: String = row.get(2)?;
    let source_layer: String = row.get(3)?;
    let influence_kind: String = row.get(4)?;
    let provenance: String = row.get(7)?;
    let bible_node_id: Option<String> = row.get(8)?;
    let bible_edge_id: Option<String> = row.get(9)?;
    let introduced_by_node_id: Option<String> = row.get(10)?;
    let sort_order: i64 = row.get(11)?;

    Ok(ContextInfluenceRecord {
        id: ContextInfluenceId(parse_uuid(row, 0, &id)?),
        evaluation_id: ContextEvaluationId(parse_uuid(row, 1, &evaluation_id)?),
        timeline_node_id: NodeId(parse_uuid(row, 2, &timeline_node_id)?),
        source_layer: parse_story_level(row, 3, &source_layer)?,
        influence_kind: decode_string_enum(row, 4, &influence_kind)?,
        confidence: row.get::<_, f32>(5)?,
        reason: row.get(6)?,
        provenance: decode_string_enum(row, 7, &provenance)?,
        bible_node_id: bible_node_id
            .map(eidetic_core::contracts::BibleGraphNodeId::new)
            .transpose()
            .map_err(|error| conversion_failure(row, 8, error))?,
        bible_edge_id: bible_edge_id
            .map(eidetic_core::contracts::BibleGraphEdgeId::new)
            .transpose()
            .map_err(|error| conversion_failure(row, 9, error))?,
        introduced_by_node_id: introduced_by_node_id
            .map(|value| parse_uuid(row, 10, &value).map(NodeId))
            .transpose()?,
        sort_order: u32::try_from(sort_order)
            .map_err(|error| conversion_failure(row, 11, error))?,
    })
}

fn encode_story_level(level: StoryLevel) -> &'static str {
    level.label()
}

fn parse_story_level(
    row: &Row<'_>,
    index: usize,
    value: &str,
) -> Result<StoryLevel, rusqlite::Error> {
    match value {
        "Premise" => Ok(StoryLevel::Premise),
        "Act" => Ok(StoryLevel::Act),
        "Sequence" => Ok(StoryLevel::Sequence),
        "Scene" => Ok(StoryLevel::Scene),
        "Beat" => Ok(StoryLevel::Beat),
        other => Err(conversion_failure(
            row,
            index,
            HistoryStoreError::InvalidValue(format!("unknown story level: {other}")),
        )),
    }
}

fn encode_string_enum<T: serde::Serialize>(value: &T) -> Result<String, HistoryStoreError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(value) => Ok(value),
        _ => Err(HistoryStoreError::InvalidValue(
            "expected enum to serialize as string".to_string(),
        )),
    }
}

fn decode_string_enum<T: serde::de::DeserializeOwned>(
    row: &Row<'_>,
    index: usize,
    value: &str,
) -> Result<T, rusqlite::Error> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .map_err(|error| conversion_failure(row, index, error))
}

fn parse_uuid(row: &Row<'_>, index: usize, value: &str) -> Result<uuid::Uuid, rusqlite::Error> {
    uuid::Uuid::parse_str(value).map_err(|error| conversion_failure(row, index, error))
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
#[path = "context_influence_store_tests.rs"]
mod tests;
