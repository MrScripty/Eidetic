use eidetic_core::contracts::{
    AffectConfidence, AffectDependency, AffectDependencyEndpoint, AffectDependencyId,
    AffectProjection, AffectProposal, AffectProposalId, AffectProposalListProjection, AffectTarget,
    AffectValue, AffectValueId, Arousal, ChangeEvent, ChangeEventKind, CommandEnvelope,
    CreateAffectProposalCommand, DeleteAffectValueCommand, EmotionalIntensity, FieldDelta,
    FieldValue, MoodLabel, ObjectKind, ObjectRevision, ProjectionEnvelope, ProjectionVersion,
    RecordAffectDependencyCommand, RevisionOperation, SetAffectValueCommand, Valence,
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

CREATE TABLE IF NOT EXISTS affect_dependencies (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    affect_id TEXT NOT NULL REFERENCES affect_values(id),
    trait_kind TEXT NOT NULL CHECK (trait_kind <> ''),
    source_kind TEXT NOT NULL CHECK (source_kind <> ''),
    source_id TEXT NOT NULL CHECK (source_id <> ''),
    source_part_key TEXT,
    source_field_key TEXT,
    target_kind TEXT NOT NULL CHECK (target_kind <> ''),
    target_id TEXT NOT NULL CHECK (target_id <> ''),
    target_part_key TEXT,
    target_field_key TEXT,
    reason TEXT NOT NULL CHECK (reason <> ''),
    created_event_id TEXT NOT NULL REFERENCES change_events(id),
    deleted_event_id TEXT REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_affect_dependencies_affect
    ON affect_dependencies(affect_id, trait_kind, id);
CREATE INDEX IF NOT EXISTS idx_affect_dependencies_source
    ON affect_dependencies(source_kind, source_id, id);
CREATE INDEX IF NOT EXISTS idx_affect_dependencies_target
    ON affect_dependencies(target_kind, target_id, id);

CREATE TABLE IF NOT EXISTS affect_proposals (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    status TEXT NOT NULL CHECK (status <> ''),
    source TEXT NOT NULL CHECK (source <> ''),
    affect_id TEXT NOT NULL CHECK (affect_id <> ''),
    target_kind TEXT NOT NULL CHECK (target_kind <> ''),
    target_id TEXT,
    valence_bp INTEGER NOT NULL,
    arousal_bp INTEGER NOT NULL,
    intensity_bp INTEGER NOT NULL,
    confidence_bp INTEGER NOT NULL,
    provenance TEXT NOT NULL CHECK (provenance <> ''),
    affect_rationale TEXT,
    summary TEXT NOT NULL CHECK (summary <> ''),
    proposal_rationale TEXT,
    source_event_id TEXT REFERENCES change_events(id),
    created_at_ms INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_affect_proposals_status
    ON affect_proposals(status, created_at_ms, id);
CREATE INDEX IF NOT EXISTS idx_affect_proposals_target
    ON affect_proposals(target_kind, target_id, id);
CREATE INDEX IF NOT EXISTS idx_affect_proposals_source_event
    ON affect_proposals(source_event_id, id);

CREATE TABLE IF NOT EXISTS affect_proposal_mood_labels (
    proposal_id TEXT NOT NULL REFERENCES affect_proposals(id) ON DELETE CASCADE,
    label TEXT NOT NULL CHECK (label <> ''),
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (proposal_id, sort_order)
);
CREATE INDEX IF NOT EXISTS idx_affect_proposal_mood_labels_label
    ON affect_proposal_mood_labels(label, proposal_id);
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

pub(crate) fn record_affect_dependency(
    conn: &mut Connection,
    command: &CommandEnvelope<RecordAffectDependencyCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    command
        .payload
        .validate()
        .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "affect.dependency_record")?
    {
        return Ok(outcome);
    }
    if load_affect_value(conn, command.payload.dependency.affect_id)?.is_none() {
        return Err(HistoryStoreError::InvalidValue(format!(
            "affect value not found: {}",
            command.payload.dependency.affect_id.0
        )));
    }
    if dependency_exists(conn, command.payload.dependency.id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "affect dependency already exists: {}",
            command.payload.dependency.id.0
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        format!(
            "record affect dependency {}",
            command.payload.dependency.id.0
        ),
    )
    .with_created_at_ms(created_at_ms);
    let revision = affect_dependency_revision(&command.payload.dependency, event.id)?;

    history_store::record_change_with(
        conn,
        command,
        "affect.dependency_record",
        &event,
        &[revision],
        |tx| insert_affect_dependency_in_transaction(tx, &command.payload.dependency, event.id),
    )
}

pub(crate) fn record_create_affect_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateAffectProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, HistoryStoreError> {
    create_schema(conn)?;
    command
        .payload
        .validate()
        .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "affect.proposal_create")?
    {
        return Ok(outcome);
    }
    if affect_proposal_exists(conn, &command.payload.proposal_id)? {
        return Err(HistoryStoreError::InvalidValue(format!(
            "affect proposal already exists: {}",
            command.payload.proposal_id.as_str()
        )));
    }

    let proposal = command.payload.clone().into_proposal(created_at_ms);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!("propose affect {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let revision = affect_proposal_revision(&proposal, event.id)?;

    history_store::record_change_with(
        conn,
        command,
        "affect.proposal_create",
        &event,
        &[revision],
        |tx| insert_affect_proposal_in_transaction(tx, &proposal, event.id),
    )
}

pub(crate) fn load_affect_dependencies_for_affect(
    conn: &Connection,
    affect_id: AffectValueId,
) -> Result<Vec<AffectDependency>, HistoryStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, affect_id, trait_kind,
                source_kind, source_id, source_part_key, source_field_key,
                target_kind, target_id, target_part_key, target_field_key,
                reason
         FROM affect_dependencies
         WHERE affect_id = ?1 AND deleted_event_id IS NULL
         ORDER BY id ASC",
    )?;
    let rows = statement.query_map([affect_id.0.to_string()], row_to_affect_dependency)?;
    let mut dependencies = Vec::new();
    for row in rows {
        dependencies.push(row?);
    }
    Ok(dependencies)
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

pub(crate) fn load_affect_proposals(
    conn: &Connection,
) -> Result<Vec<AffectProposal>, HistoryStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, status, source, affect_id, target_kind, target_id,
                valence_bp, arousal_bp, intensity_bp, confidence_bp, provenance,
                affect_rationale, summary, proposal_rationale, source_event_id, created_at_ms
         FROM affect_proposals
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_affect_proposal)?;
    let mut proposals = Vec::new();
    for row in rows {
        let mut proposal = row?;
        proposal.proposed_value.mood_labels = load_proposal_mood_labels(conn, &proposal.id)?;
        proposals.push(proposal);
    }
    Ok(proposals)
}

pub(crate) fn load_affect_proposal_list_projection(
    conn: &Connection,
) -> Result<ProjectionEnvelope<AffectProposalListProjection>, HistoryStoreError> {
    let proposals = load_affect_proposals(conn)?;
    let summary = history_store::load_revision_summary_for_kind(conn, ObjectKind::AffectProposal)?;
    let projection = AffectProposalListProjection { proposals };
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

pub(crate) fn load_timeline_node_affect_values(
    conn: &Connection,
) -> Result<Vec<AffectValue>, HistoryStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, target_kind, target_id, valence_bp, arousal_bp, intensity_bp,
                confidence_bp, provenance, rationale
         FROM affect_values
         WHERE target_kind = 'timeline_node'
           AND deleted_event_id IS NULL
         ORDER BY target_id ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_affect_value)?;
    let mut values = Vec::new();
    for row in rows {
        let mut value = row?;
        value.mood_labels = load_mood_labels(conn, value.id)?;
        values.push(value);
    }
    Ok(values)
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

fn insert_affect_dependency_in_transaction(
    tx: &Transaction<'_>,
    dependency: &AffectDependency,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let source = SqlAffectDependencyEndpoint::from_endpoint(&dependency.source);
    let target = SqlAffectDependencyEndpoint::from_endpoint(&dependency.target);
    tx.execute(
        "INSERT INTO affect_dependencies (
            id, affect_id, trait_kind,
            source_kind, source_id, source_part_key, source_field_key,
            target_kind, target_id, target_part_key, target_field_key,
            reason, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            dependency.id.0.to_string(),
            dependency.affect_id.0.to_string(),
            encode_string_enum(&dependency.trait_kind)?,
            source.kind,
            source.id,
            source.part_key,
            source.field_key,
            target.kind,
            target.id,
            target.part_key,
            target.field_key,
            dependency.reason,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn insert_affect_proposal_in_transaction(
    tx: &Transaction<'_>,
    proposal: &AffectProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let value = &proposal.proposed_value;
    let (target_kind, target_id) = encode_target(&value.target)?;
    tx.execute(
        "INSERT INTO affect_proposals (
            id, status, source, affect_id, target_kind, target_id,
            valence_bp, arousal_bp, intensity_bp, confidence_bp, provenance,
            affect_rationale, summary, proposal_rationale, source_event_id,
            created_at_ms, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            proposal.id.as_str(),
            encode_string_enum(&proposal.status)?,
            encode_string_enum(&proposal.source)?,
            value.id.0.to_string(),
            target_kind,
            target_id,
            i64::from(value.valence.basis_points()),
            i64::from(value.arousal.basis_points()),
            i64::from(value.intensity.basis_points()),
            i64::from(value.confidence.basis_points()),
            encode_string_enum(&value.provenance)?,
            value.rationale.as_deref(),
            proposal.summary.as_str(),
            proposal.rationale.as_deref(),
            proposal.source_event_id.map(|id| id.0.to_string()),
            proposal.created_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    for (sort_order, label) in value.mood_labels.iter().enumerate() {
        tx.execute(
            "INSERT INTO affect_proposal_mood_labels (proposal_id, label, sort_order)
             VALUES (?1, ?2, ?3)",
            params![proposal.id.as_str(), label.as_str(), sort_order as i64],
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

fn affect_dependency_revision(
    dependency: &AffectDependency,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::AffectDependency,
        dependency.id.0.to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "affect_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::AffectValue,
            id: dependency.affect_id.0.to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "trait_kind",
        None,
        Some(FieldValue::Text(encode_string_enum(
            &dependency.trait_kind,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "source",
        None,
        Some(FieldValue::Text(endpoint_label(&dependency.source))),
    ))
    .with_field(FieldDelta::new(
        "target",
        None,
        Some(FieldValue::Text(endpoint_label(&dependency.target))),
    )))
}

fn affect_proposal_revision(
    proposal: &AffectProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    Ok(ObjectRevision::new(
        ObjectKind::AffectProposal,
        proposal.id.as_str().to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
    ))
    .with_field(FieldDelta::new(
        "source",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.source)?)),
    ))
    .with_field(FieldDelta::new(
        "target",
        None,
        Some(FieldValue::Text(target_label(
            &proposal.proposed_value.target,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "summary",
        None,
        Some(FieldValue::Text(proposal.summary.clone())),
    ))
    .with_field(FieldDelta::new(
        "affect_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::AffectValue,
            id: proposal.proposed_value.id.0.to_string(),
        }),
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

fn row_to_affect_dependency(row: &Row<'_>) -> Result<AffectDependency, rusqlite::Error> {
    let id: String = row.get(0)?;
    let affect_id: String = row.get(1)?;
    let trait_kind: String = row.get(2)?;
    let source = SqlAffectDependencyEndpoint {
        kind: row.get(3)?,
        id: row.get(4)?,
        part_key: row.get(5)?,
        field_key: row.get(6)?,
    }
    .into_endpoint(3)?;
    let target = SqlAffectDependencyEndpoint {
        kind: row.get(7)?,
        id: row.get(8)?,
        part_key: row.get(9)?,
        field_key: row.get(10)?,
    }
    .into_endpoint(7)?;
    Ok(AffectDependency {
        id: AffectDependencyId(parse_uuid(&id, 0)?),
        affect_id: AffectValueId(parse_uuid(&affect_id, 1)?),
        trait_kind: decode_string_enum(&trait_kind, 2)?,
        source,
        target,
        reason: row.get(11)?,
    })
}

fn row_to_affect_proposal(row: &Row<'_>) -> Result<AffectProposal, rusqlite::Error> {
    let id: String = row.get(0)?;
    let status: String = row.get(1)?;
    let source: String = row.get(2)?;
    let affect_id: String = row.get(3)?;
    let target_kind: String = row.get(4)?;
    let target_id: Option<String> = row.get(5)?;
    let provenance: String = row.get(10)?;
    let source_event_id: Option<String> = row.get(14)?;
    let created_at_ms: i64 = row.get(15)?;
    Ok(AffectProposal {
        id: AffectProposalId::new(id)
            .map_err(|error| conversion_error(0, rusqlite::types::Type::Text, error))?,
        status: decode_string_enum(&status, 1)?,
        source: decode_string_enum(&source, 2)?,
        proposed_value: AffectValue {
            id: AffectValueId(parse_uuid(&affect_id, 3)?),
            target: decode_target(&target_kind, target_id.as_deref(), 4)?,
            valence: Valence::new(row.get::<_, i16>(6)?)
                .map_err(|error| conversion_error(6, rusqlite::types::Type::Integer, error))?,
            arousal: Arousal::new(row.get::<_, u16>(7)?)
                .map_err(|error| conversion_error(7, rusqlite::types::Type::Integer, error))?,
            intensity: EmotionalIntensity::new(row.get::<_, u16>(8)?)
                .map_err(|error| conversion_error(8, rusqlite::types::Type::Integer, error))?,
            confidence: AffectConfidence::new(row.get::<_, u16>(9)?)
                .map_err(|error| conversion_error(9, rusqlite::types::Type::Integer, error))?,
            mood_labels: Vec::new(),
            provenance: decode_string_enum(&provenance, 10)?,
            rationale: row.get(11)?,
        },
        summary: row.get(12)?,
        rationale: row.get(13)?,
        source_event_id: source_event_id
            .as_deref()
            .map(|value| parse_uuid(value, 14).map(eidetic_core::contracts::ChangeEventId))
            .transpose()?,
        created_at_ms: u64::try_from(created_at_ms)
            .map_err(|error| conversion_error(15, rusqlite::types::Type::Integer, error))?,
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

fn load_proposal_mood_labels(
    conn: &Connection,
    proposal_id: &AffectProposalId,
) -> Result<Vec<MoodLabel>, HistoryStoreError> {
    let mut statement = conn.prepare(
        "SELECT label
         FROM affect_proposal_mood_labels
         WHERE proposal_id = ?1
         ORDER BY sort_order ASC",
    )?;
    let rows = statement.query_map([proposal_id.as_str()], |row| {
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

fn dependency_exists(
    conn: &Connection,
    dependency_id: AffectDependencyId,
) -> Result<bool, HistoryStoreError> {
    conn.query_row(
        "SELECT 1 FROM affect_dependencies WHERE id = ?1 AND deleted_event_id IS NULL",
        [dependency_id.0.to_string()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

fn affect_proposal_exists(
    conn: &Connection,
    proposal_id: &AffectProposalId,
) -> Result<bool, HistoryStoreError> {
    conn.query_row(
        "SELECT 1 FROM affect_proposals WHERE id = ?1",
        [proposal_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(HistoryStoreError::from)
}

struct SqlAffectDependencyEndpoint {
    kind: String,
    id: String,
    part_key: Option<String>,
    field_key: Option<String>,
}

impl SqlAffectDependencyEndpoint {
    fn from_endpoint(endpoint: &AffectDependencyEndpoint) -> Self {
        match endpoint {
            AffectDependencyEndpoint::TimelineNode { node_id } => Self {
                kind: "timeline_node".to_string(),
                id: node_id.0.to_string(),
                part_key: None,
                field_key: None,
            },
            AffectDependencyEndpoint::ScriptSegment { segment_id } => Self {
                kind: "script_segment".to_string(),
                id: segment_id.as_str().to_string(),
                part_key: None,
                field_key: None,
            },
            AffectDependencyEndpoint::BibleNode { node_id } => Self {
                kind: "bible_node".to_string(),
                id: node_id.as_str().to_string(),
                part_key: None,
                field_key: None,
            },
            AffectDependencyEndpoint::BibleField {
                node_id,
                part_key,
                field_key,
            } => Self {
                kind: "bible_field".to_string(),
                id: node_id.as_str().to_string(),
                part_key: Some(part_key.as_str().to_string()),
                field_key: Some(field_key.as_str().to_string()),
            },
            AffectDependencyEndpoint::GenerationPrompt { workflow_id } => Self {
                kind: "generation_prompt".to_string(),
                id: workflow_id.as_str().to_string(),
                part_key: None,
                field_key: None,
            },
        }
    }

    fn into_endpoint(self, column: usize) -> Result<AffectDependencyEndpoint, rusqlite::Error> {
        Ok(match self.kind.as_str() {
            "timeline_node" => AffectDependencyEndpoint::TimelineNode {
                node_id: NodeId(parse_uuid(&self.id, column)?),
            },
            "script_segment" => AffectDependencyEndpoint::ScriptSegment {
                segment_id: eidetic_core::contracts::ScriptSegmentId::new(self.id).map_err(
                    |error| conversion_error(column, rusqlite::types::Type::Text, error),
                )?,
            },
            "bible_node" => AffectDependencyEndpoint::BibleNode {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id).map_err(
                    |error| conversion_error(column, rusqlite::types::Type::Text, error),
                )?,
            },
            "bible_field" => AffectDependencyEndpoint::BibleField {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id).map_err(
                    |error| conversion_error(column, rusqlite::types::Type::Text, error),
                )?,
                part_key: eidetic_core::contracts::BibleGraphPartKey::new(required_dependency_key(
                    self.part_key.as_deref(),
                    column,
                    "part_key",
                )?)
                .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))?,
                field_key: eidetic_core::contracts::BibleGraphFieldKey::new(
                    required_dependency_key(self.field_key.as_deref(), column, "field_key")?,
                )
                .map_err(|error| conversion_error(column, rusqlite::types::Type::Text, error))?,
            },
            "generation_prompt" => AffectDependencyEndpoint::GenerationPrompt {
                workflow_id: eidetic_core::contracts::AgentWorkflowId::new(self.id).map_err(
                    |error| conversion_error(column, rusqlite::types::Type::Text, error),
                )?,
            },
            _ => {
                return Err(conversion_error(
                    column,
                    rusqlite::types::Type::Text,
                    format!("unknown affect dependency endpoint kind: {}", self.kind),
                ));
            }
        })
    }
}

fn required_dependency_key<'a>(
    value: Option<&'a str>,
    column: usize,
    name: &'static str,
) -> Result<&'a str, rusqlite::Error> {
    value.filter(|value| !value.is_empty()).ok_or_else(|| {
        conversion_error(
            column,
            rusqlite::types::Type::Text,
            format!("affect dependency {name} is required"),
        )
    })
}

fn endpoint_label(endpoint: &AffectDependencyEndpoint) -> String {
    let sql = SqlAffectDependencyEndpoint::from_endpoint(endpoint);
    match (sql.part_key, sql.field_key) {
        (Some(part), Some(field)) => format!("{}:{}:{}:{}", sql.kind, sql.id, part, field),
        _ => format!("{}:{}", sql.kind, sql.id),
    }
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
    use eidetic_core::contracts::{
        AffectProposalSource, AffectProvenance, AffectTraitKind, AgentWorkflowId, CommandId,
        SemanticProposalStatus,
    };

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

    #[test]
    fn affect_store_records_and_loads_dependencies() {
        let mut conn = Connection::open_in_memory().unwrap();
        let affect_command = set_command(AffectTarget::Project, "uneasy");
        let affect_id = affect_command.affect_id;
        record_set_affect_value(&mut conn, &CommandEnvelope::new(affect_command), 100).unwrap();
        let dependency = affect_dependency(affect_id);

        let outcome = record_affect_dependency(
            &mut conn,
            &CommandEnvelope::new(RecordAffectDependencyCommand {
                command_id: CommandId::new(),
                dependency: dependency.clone(),
            }),
            101,
        )
        .unwrap();

        assert_eq!(outcome, RecordChangeOutcome::Recorded);
        let dependencies = load_affect_dependencies_for_affect(&conn, affect_id).unwrap();
        assert_eq!(dependencies, vec![dependency]);
    }

    #[test]
    fn affect_store_rejects_dependency_for_missing_affect_value() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(RecordAffectDependencyCommand {
            command_id: CommandId::new(),
            dependency: affect_dependency(AffectValueId::new()),
        });

        let error = record_affect_dependency(&mut conn, &command, 101).unwrap_err();

        assert!(error.to_string().contains("affect value not found"));
    }

    #[test]
    fn affect_store_is_idempotent_for_duplicate_dependency_commands() {
        let mut conn = Connection::open_in_memory().unwrap();
        let affect_command = set_command(AffectTarget::Project, "uneasy");
        let affect_id = affect_command.affect_id;
        record_set_affect_value(&mut conn, &CommandEnvelope::new(affect_command), 100).unwrap();
        let command = CommandEnvelope::new(RecordAffectDependencyCommand {
            command_id: CommandId::new(),
            dependency: affect_dependency(affect_id),
        });

        assert_eq!(
            record_affect_dependency(&mut conn, &command, 101).unwrap(),
            RecordChangeOutcome::Recorded
        );
        assert_eq!(
            record_affect_dependency(&mut conn, &command, 101).unwrap(),
            RecordChangeOutcome::AlreadyRecorded
        );
    }

    #[test]
    fn affect_store_records_and_projects_affect_proposals() {
        let mut conn = Connection::open_in_memory().unwrap();
        let command = CommandEnvelope::new(CreateAffectProposalCommand {
            proposal_id: AffectProposalId::new("proposal.affect.scene-weather").unwrap(),
            source: AffectProposalSource::ManualScriptEdit,
            proposed_value: AffectValue {
                id: AffectValueId::new(),
                target: AffectTarget::Project,
                valence: Valence::new(-100).unwrap(),
                arousal: Arousal::new(600).unwrap(),
                intensity: EmotionalIntensity::new(700).unwrap(),
                confidence: AffectConfidence::new(850).unwrap(),
                mood_labels: vec![MoodLabel::new("rainy").unwrap()],
                provenance: AffectProvenance::ScriptEditDetected,
                rationale: Some("User changed the scene weather.".to_string()),
            },
            summary: "Detected rainier project mood".to_string(),
            rationale: Some("Manual script edit introduced rain.".to_string()),
            source_event_id: None,
        });

        let outcome = record_create_affect_proposal(&mut conn, &command, 200).unwrap();

        assert_eq!(outcome, RecordChangeOutcome::Recorded);
        let projection = load_affect_proposal_list_projection(&conn).unwrap();
        assert_eq!(projection.payload.proposals.len(), 1);
        let proposal = &projection.payload.proposals[0];
        assert_eq!(proposal.id.as_str(), "proposal.affect.scene-weather");
        assert_eq!(proposal.status, SemanticProposalStatus::Pending);
        assert_eq!(proposal.source, AffectProposalSource::ManualScriptEdit);
        assert_eq!(proposal.proposed_value.mood_labels[0].as_str(), "rainy");
        assert_eq!(proposal.created_at_ms, 200);
        assert!(
            load_affect_projection(&conn, AffectTarget::Project)
                .unwrap()
                .payload
                .values
                .is_empty()
        );
    }

    #[test]
    fn affect_store_loads_timeline_node_affect_values() {
        let mut conn = Connection::open_in_memory().unwrap();
        let timeline_node_id = NodeId::new();
        record_set_affect_value(
            &mut conn,
            &CommandEnvelope::new(set_command(
                AffectTarget::TimelineNode {
                    node_id: timeline_node_id,
                },
                "uneasy",
            )),
            100,
        )
        .unwrap();
        record_set_affect_value(
            &mut conn,
            &CommandEnvelope::new(set_command(AffectTarget::Project, "hopeful")),
            101,
        )
        .unwrap();

        let values = load_timeline_node_affect_values(&conn).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0].target,
            AffectTarget::TimelineNode {
                node_id: timeline_node_id
            }
        );
        assert_eq!(values[0].mood_labels[0].as_str(), "uneasy");
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

    fn affect_dependency(affect_id: AffectValueId) -> AffectDependency {
        AffectDependency {
            id: AffectDependencyId::new(),
            affect_id,
            trait_kind: AffectTraitKind::Valence,
            source: AffectDependencyEndpoint::TimelineNode {
                node_id: NodeId::new(),
            },
            target: AffectDependencyEndpoint::GenerationPrompt {
                workflow_id: AgentWorkflowId::new("workflow.scene.graph_context").unwrap(),
            },
            reason: "Affect constrains prompt tone.".to_string(),
        }
    }
}
