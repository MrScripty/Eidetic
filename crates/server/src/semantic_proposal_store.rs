use eidetic_core::contracts::{
    BibleGraphSchemaKey, BibleReferenceProposal, BibleReferenceProposalListProjection, ChangeEvent,
    ChangeEventKind, CommandEnvelope, CreateBibleReferenceProposalCommand, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, ProjectionEnvelope, ProjectionVersion,
    RejectBibleReferenceProposalCommand, RevisionOperation, SemanticProposalId,
    SemanticProposalStatus,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};

const SEMANTIC_PROPOSAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS bible_reference_proposals (
    id                  TEXT PRIMARY KEY CHECK (id <> ''),
    source_node_id      TEXT NOT NULL CHECK (source_node_id <> ''),
    child_name          TEXT NOT NULL,
    reference_kind      TEXT NOT NULL,
    reference_text      TEXT NOT NULL CHECK (reference_text <> ''),
    proposed_schema_key TEXT NOT NULL CHECK (proposed_schema_key <> ''),
    status              TEXT NOT NULL,
    rationale           TEXT,
    created_at_ms       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_bible_reference_proposals_status
    ON bible_reference_proposals(status, created_at_ms);
CREATE INDEX IF NOT EXISTS idx_bible_reference_proposals_source
    ON bible_reference_proposals(source_node_id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), HistoryStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(SEMANTIC_PROPOSAL_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_create_bible_reference_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<CreateBibleReferenceProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, SemanticProposalStoreError> {
    create_schema(conn)?;
    validate_create_command(&command.payload)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.bible_reference_proposal")?
    {
        return Ok(outcome);
    }
    if proposal_exists(conn, &command.payload.proposal_id)? {
        return Err(SemanticProposalStoreError::InvalidCommand(format!(
            "semantic proposal already exists: {}",
            command.payload.proposal_id.as_str()
        )));
    }

    let proposal = command.payload.clone().into_proposal(created_at_ms);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!("propose bible reference {}", proposal.reference_text),
    )
    .with_created_at_ms(created_at_ms);
    let revision = bible_reference_proposal_revision(&proposal, event.id)?;

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.bible_reference_proposal",
        &event,
        &[revision],
        |tx| insert_proposal_in_transaction(tx, &proposal),
    )?)
}

pub(crate) fn record_reject_bible_reference_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<RejectBibleReferenceProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, SemanticProposalStoreError> {
    create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.bible_reference_reject")?
    {
        return Ok(outcome);
    }
    let proposal =
        load_bible_reference_proposal(conn, &command.payload.proposal_id)?.ok_or_else(|| {
            SemanticProposalStoreError::NotFound(format!(
                "semantic proposal not found: {}",
                command.payload.proposal_id.as_str()
            ))
        })?;
    if proposal.status != SemanticProposalStatus::Pending {
        return Err(SemanticProposalStoreError::InvalidCommand(format!(
            "semantic proposal is not pending: {}",
            proposal.id.as_str()
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalRejected,
        format!("reject bible reference {}", proposal.reference_text),
    )
    .with_created_at_ms(created_at_ms);
    let mut revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str().to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "status",
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
        Some(FieldValue::Text(encode_string_enum(
            &SemanticProposalStatus::Rejected,
        )?)),
    ));
    if let Some(reason) = command.payload.reason.as_ref() {
        revision = revision.with_field(FieldDelta::new(
            "rejection_reason",
            None,
            Some(FieldValue::Text(reason.clone())),
        ));
    }

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.bible_reference_reject",
        &event,
        &[revision],
        |tx| {
            update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Pending,
                SemanticProposalStatus::Rejected,
            )
        },
    )?)
}

pub(crate) fn load_bible_reference_proposals(
    conn: &Connection,
) -> Result<Vec<BibleReferenceProposal>, SemanticProposalStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT id, source_node_id, child_name, reference_kind, reference_text,
                proposed_schema_key, status, rationale, created_at_ms
         FROM bible_reference_proposals
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_proposal)?;
    let mut proposals = Vec::new();
    for row in rows {
        proposals.push(row?);
    }
    Ok(proposals)
}

fn load_bible_reference_proposal(
    conn: &Connection,
    proposal_id: &SemanticProposalId,
) -> Result<Option<BibleReferenceProposal>, SemanticProposalStoreError> {
    create_schema(conn)?;
    conn.query_row(
        "SELECT id, source_node_id, child_name, reference_kind, reference_text,
                proposed_schema_key, status, rationale, created_at_ms
         FROM bible_reference_proposals
         WHERE id = ?1",
        [proposal_id.as_str()],
        row_to_proposal,
    )
    .optional()
    .map_err(SemanticProposalStoreError::from)
}

pub(crate) fn load_bible_reference_proposal_list_projection(
    conn: &Connection,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, SemanticProposalStoreError> {
    let proposals = load_bible_reference_proposals(conn)?;
    let summary =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::SemanticProposal)?;
    let projection = BibleReferenceProposalListProjection { proposals };
    Ok(match summary.latest_change_event_id {
        Some(change_event_id) => ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        ),
        None => ProjectionEnvelope::initial(projection),
    })
}

fn validate_create_command(
    command: &CreateBibleReferenceProposalCommand,
) -> Result<(), SemanticProposalStoreError> {
    if command.reference_text.trim().is_empty() {
        return Err(SemanticProposalStoreError::InvalidCommand(
            "reference_text is required".to_string(),
        ));
    }
    if command.child_name.trim().is_empty() {
        return Err(SemanticProposalStoreError::InvalidCommand(
            "child_name is required".to_string(),
        ));
    }
    Ok(())
}

fn proposal_exists(
    conn: &Connection,
    proposal_id: &SemanticProposalId,
) -> Result<bool, SemanticProposalStoreError> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM bible_reference_proposals WHERE id = ?1",
            [proposal_id.as_str()],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

pub(crate) fn bible_reference_proposal_revision(
    proposal: &BibleReferenceProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    let revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str().to_string(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "source_node_id",
        None,
        Some(FieldValue::ObjectRef {
            kind: ObjectKind::TimelineNode,
            id: proposal.source_node_id.0.to_string(),
        }),
    ))
    .with_field(FieldDelta::new(
        "child_name",
        None,
        Some(FieldValue::Text(proposal.child_name.clone())),
    ))
    .with_field(FieldDelta::new(
        "reference_kind",
        None,
        Some(FieldValue::Text(encode_string_enum(
            &proposal.reference_kind,
        )?)),
    ))
    .with_field(FieldDelta::new(
        "reference_text",
        None,
        Some(FieldValue::Text(proposal.reference_text.clone())),
    ))
    .with_field(FieldDelta::new(
        "proposed_schema_key",
        None,
        Some(FieldValue::Text(
            proposal.proposed_schema_key.as_str().to_string(),
        )),
    ))
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
    ));
    Ok(match proposal.rationale.as_ref() {
        Some(rationale) => revision.with_field(FieldDelta::new(
            "rationale",
            None,
            Some(FieldValue::Text(rationale.clone())),
        )),
        None => revision,
    })
}

pub(crate) fn insert_proposal_in_transaction(
    tx: &Transaction<'_>,
    proposal: &BibleReferenceProposal,
) -> Result<(), HistoryStoreError> {
    tx.execute(
        "INSERT INTO bible_reference_proposals (
            id, source_node_id, child_name, reference_kind, reference_text,
            proposed_schema_key, status, rationale, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            proposal.id.as_str(),
            proposal.source_node_id.0.to_string(),
            proposal.child_name.as_str(),
            encode_string_enum(&proposal.reference_kind)?,
            proposal.reference_text.as_str(),
            proposal.proposed_schema_key.as_str(),
            encode_string_enum(&proposal.status)?,
            proposal.rationale.as_deref(),
            proposal.created_at_ms as i64
        ],
    )?;
    Ok(())
}

fn update_proposal_status_in_transaction(
    tx: &Transaction<'_>,
    proposal_id: &SemanticProposalId,
    old_status: SemanticProposalStatus,
    new_status: SemanticProposalStatus,
) -> Result<(), HistoryStoreError> {
    let updated = tx.execute(
        "UPDATE bible_reference_proposals
         SET status = ?1
         WHERE id = ?2 AND status = ?3",
        params![
            encode_string_enum(&new_status)?,
            proposal_id.as_str(),
            encode_string_enum(&old_status)?
        ],
    )?;
    if updated != 1 {
        return Err(HistoryStoreError::InvalidValue(format!(
            "semantic proposal status changed before update: {}",
            proposal_id.as_str()
        )));
    }
    Ok(())
}

fn row_to_proposal(row: &rusqlite::Row<'_>) -> Result<BibleReferenceProposal, rusqlite::Error> {
    let id: String = row.get(0)?;
    let source_node_id: String = row.get(1)?;
    let reference_kind: String = row.get(3)?;
    let proposed_schema_key: String = row.get(5)?;
    let status: String = row.get(6)?;
    let created_at_ms: i64 = row.get(8)?;
    Ok(BibleReferenceProposal {
        id: SemanticProposalId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        source_node_id: NodeId(
            uuid::Uuid::parse_str(&source_node_id).map_err(|e| conversion_failure(row, 1, e))?,
        ),
        child_name: row.get(2)?,
        reference_kind: decode_string_enum(&reference_kind)
            .map_err(|e| conversion_failure(row, 3, e))?,
        reference_text: row.get(4)?,
        proposed_schema_key: BibleGraphSchemaKey::new(proposed_schema_key)
            .map_err(|e| conversion_failure(row, 5, e))?,
        status: decode_string_enum(&status).map_err(|e| conversion_failure(row, 6, e))?,
        rationale: row.get(7)?,
        created_at_ms: u64::try_from(created_at_ms).map_err(|e| conversion_failure(row, 8, e))?,
    })
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

fn decode_string_enum<T>(value: &str) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_string()))
}

fn conversion_failure<E>(row: &rusqlite::Row<'_>, index: usize, error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::FromSqlConversionFailure(
        index,
        row.get_ref_unwrap(index).data_type(),
        Box::new(error),
    )
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SemanticProposalStoreError {
    #[error("{0}")]
    InvalidCommand(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[cfg(test)]
#[path = "semantic_proposal_store_tests.rs"]
mod tests;
