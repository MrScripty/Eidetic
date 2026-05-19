use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, CreatePropagationProposalCommand, FieldDelta,
    FieldValue, ObjectKind, ObjectRevision, ProjectionEnvelope, ProjectionVersion,
    PropagationProposal, PropagationProposalAction, PropagationProposalId,
    PropagationProposalListProjection, PropagationProposalTarget, RevisionOperation,
};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::bible_graph_value_store::SqlGraphFieldValue;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_target::{
    PropagationProposalTargetError, SqlPropagationTarget, target_label,
};

const PROPAGATION_PROPOSAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS propagation_proposals (
    id TEXT PRIMARY KEY CHECK (id <> ''),
    action TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL CHECK (target_id <> ''),
    target_part_key TEXT,
    target_field_key TEXT,
    target_field_id TEXT,
    target_snapshot_id TEXT,
    status TEXT NOT NULL,
    summary TEXT NOT NULL CHECK (summary <> ''),
    proposed_value_type TEXT,
    proposed_value_text TEXT,
    proposed_value_integer INTEGER,
    proposed_value_number REAL,
    proposed_value_bool INTEGER CHECK (proposed_value_bool IS NULL OR proposed_value_bool IN (0, 1)),
    proposed_value_ref_kind TEXT,
    proposed_value_ref_id TEXT,
    proposed_value_asset_ref TEXT,
    proposed_text TEXT,
    source_dependency_id TEXT,
    source_event_id TEXT,
    rationale TEXT,
    created_at_ms INTEGER NOT NULL,
    created_event_id TEXT NOT NULL REFERENCES change_events(id)
);
CREATE INDEX IF NOT EXISTS idx_propagation_proposals_status
    ON propagation_proposals(status, created_at_ms);
CREATE INDEX IF NOT EXISTS idx_propagation_proposals_target
    ON propagation_proposals(target_kind, target_id, target_part_key, target_field_key);
CREATE INDEX IF NOT EXISTS idx_propagation_proposals_source_dependency
    ON propagation_proposals(source_dependency_id);
"#;

pub(crate) fn create_schema(conn: &Connection) -> Result<(), PropagationProposalStoreError> {
    history_store::create_schema(conn)?;
    conn.execute_batch(PROPAGATION_PROPOSAL_SCHEMA_SQL)?;
    Ok(())
}

pub(crate) fn record_create_propagation_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<CreatePropagationProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    create_schema(conn)?;
    validate_create_command(&command.payload)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.propagation_proposal")?
    {
        return Ok(outcome);
    }
    if proposal_exists(conn, &command.payload.proposal_id)? {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal already exists: {}",
            command.payload.proposal_id.as_str()
        )));
    }

    let proposal = command.payload.clone().into_proposal(created_at_ms);
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        format!("propose propagation {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let revision = propagation_proposal_revision(&proposal, event.id)?;

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.propagation_proposal",
        &event,
        &[revision],
        |tx| insert_proposal_in_transaction(tx, &proposal, event.id),
    )?)
}

pub(crate) fn load_propagation_proposals(
    conn: &Connection,
) -> Result<Vec<PropagationProposal>, PropagationProposalStoreError> {
    create_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT
            id, action, target_kind, target_id, target_part_key, target_field_key,
            target_field_id, target_snapshot_id, status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, proposed_text,
            source_dependency_id, source_event_id, rationale, created_at_ms
         FROM propagation_proposals
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let rows = statement.query_map([], row_to_proposal)?;
    let mut proposals = Vec::new();
    for row in rows {
        proposals.push(row?);
    }
    Ok(proposals)
}

pub(crate) fn load_propagation_proposal(
    conn: &Connection,
    proposal_id: &PropagationProposalId,
) -> Result<Option<PropagationProposal>, PropagationProposalStoreError> {
    create_schema(conn)?;
    conn.query_row(
        "SELECT
            id, action, target_kind, target_id, target_part_key, target_field_key,
            target_field_id, target_snapshot_id, status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, proposed_text,
            source_dependency_id, source_event_id, rationale, created_at_ms
         FROM propagation_proposals
         WHERE id = ?1",
        [proposal_id.as_str()],
        row_to_proposal,
    )
    .optional()
    .map_err(PropagationProposalStoreError::from)
}

pub(crate) fn load_propagation_proposal_list_projection(
    conn: &Connection,
) -> Result<ProjectionEnvelope<PropagationProposalListProjection>, PropagationProposalStoreError> {
    let proposals = load_propagation_proposals(conn)?;
    let summary =
        history_store::load_revision_summary_for_kind(conn, ObjectKind::SemanticProposal)?;
    let projection = PropagationProposalListProjection { proposals };
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
    command: &CreatePropagationProposalCommand,
) -> Result<(), PropagationProposalStoreError> {
    validate_proposal_shape(
        &command.action,
        &command.target,
        &command.summary,
        command.proposed_value.as_ref(),
        command.proposed_text.as_deref(),
    )
}

pub(crate) fn validate_proposal_shape(
    action: &PropagationProposalAction,
    target: &PropagationProposalTarget,
    summary: &str,
    proposed_value: Option<&FieldValue>,
    proposed_text: Option<&str>,
) -> Result<(), PropagationProposalStoreError> {
    if summary.trim().is_empty() {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "summary is required".to_string(),
        ));
    }
    match (action, target) {
        (
            PropagationProposalAction::SetBibleField,
            PropagationProposalTarget::BibleField { .. },
        )
        | (
            PropagationProposalAction::SetBibleSnapshotField,
            PropagationProposalTarget::BibleSnapshotField { .. },
        ) => {
            if proposed_value.is_none() {
                return Err(PropagationProposalStoreError::InvalidCommand(
                    "proposed_value is required for bible propagation proposals".to_string(),
                ));
            }
        }
        (
            PropagationProposalAction::PatchScriptBlock,
            PropagationProposalTarget::ScriptBlock { .. },
        ) => {
            if proposed_text.is_none_or(|value| value.trim().is_empty()) {
                return Err(PropagationProposalStoreError::InvalidCommand(
                    "proposed_text is required for script block patch proposals".to_string(),
                ));
            }
        }
        (
            PropagationProposalAction::RegenerateScriptSegment,
            PropagationProposalTarget::ScriptSegment { .. },
        ) => {}
        _ => {
            return Err(PropagationProposalStoreError::InvalidCommand(
                "propagation proposal action does not match target kind".to_string(),
            ));
        }
    }
    Ok(())
}

fn proposal_exists(
    conn: &Connection,
    proposal_id: &PropagationProposalId,
) -> Result<bool, PropagationProposalStoreError> {
    conn.query_row(
        "SELECT 1 FROM propagation_proposals WHERE id = ?1",
        [proposal_id.as_str()],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(PropagationProposalStoreError::from)
}

fn insert_proposal_in_transaction(
    tx: &Transaction<'_>,
    proposal: &PropagationProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let target = SqlPropagationTarget::from_target(&proposal.target);
    let proposed_value = SqlGraphFieldValue::from_field_value(proposal.proposed_value.as_ref())?;
    tx.execute(
        "INSERT INTO propagation_proposals (
            id, action,
            target_kind, target_id, target_part_key, target_field_key, target_field_id, target_snapshot_id,
            status, summary,
            proposed_value_type, proposed_value_text, proposed_value_integer,
            proposed_value_number, proposed_value_bool, proposed_value_ref_kind,
            proposed_value_ref_id, proposed_value_asset_ref, proposed_text,
            source_dependency_id, source_event_id, rationale, created_at_ms, created_event_id
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
         )",
        params![
            proposal.id.as_str(),
            encode_string_enum(&proposal.action)?,
            target.kind,
            target.id,
            target.part_key,
            target.field_key,
            target.field_id,
            target.snapshot_id,
            encode_string_enum(&proposal.status)?,
            proposal.summary,
            proposed_value.value_type,
            proposed_value.text,
            proposed_value.integer,
            proposed_value.number,
            proposed_value.bool_value,
            proposed_value.ref_kind,
            proposed_value.ref_id,
            proposed_value.asset_ref,
            proposal.proposed_text,
            proposal.source_dependency_id.as_ref().map(|id| id.as_str()),
            proposal.source_event_id.map(|id| id.0.to_string()),
            proposal.rationale,
            proposal.created_at_ms as i64,
            event_id.0.to_string(),
        ],
    )?;
    Ok(())
}

fn row_to_proposal(row: &Row<'_>) -> Result<PropagationProposal, rusqlite::Error> {
    let id: String = row.get(0)?;
    let action: String = row.get(1)?;
    let status: String = row.get(8)?;
    let value = SqlGraphFieldValue {
        value_type: row.get(10)?,
        text: row.get(11)?,
        integer: row.get(12)?,
        number: row.get(13)?,
        bool_value: row.get(14)?,
        ref_kind: row.get(15)?,
        ref_id: row.get(16)?,
        asset_ref: row.get(17)?,
    };
    let source_dependency_id: Option<String> = row.get(19)?;
    let source_event_id: Option<String> = row.get(20)?;
    let created_at_ms: i64 = row.get(22)?;
    Ok(PropagationProposal {
        id: PropagationProposalId::new(id).map_err(|e| conversion_failure(row, 0, e))?,
        action: decode_string_enum(&action).map_err(|e| conversion_failure(row, 1, e))?,
        target: SqlPropagationTarget {
            kind: row.get(2)?,
            id: row.get(3)?,
            part_key: row.get(4)?,
            field_key: row.get(5)?,
            field_id: row.get(6)?,
            snapshot_id: row.get(7)?,
        }
        .into_target()
        .map_err(|e| conversion_failure(row, 2, e))?,
        status: decode_string_enum(&status).map_err(|e| conversion_failure(row, 8, e))?,
        summary: row.get(9)?,
        proposed_value: value
            .into_field_value()
            .map_err(|e| conversion_failure(row, 10, e))?,
        proposed_text: row.get(18)?,
        source_dependency_id: source_dependency_id
            .map(eidetic_core::contracts::SemanticDependencyId::new)
            .transpose()
            .map_err(|e| conversion_failure(row, 19, e))?,
        source_event_id: source_event_id
            .map(|id| uuid::Uuid::parse_str(&id).map(eidetic_core::contracts::ChangeEventId))
            .transpose()
            .map_err(|e| conversion_failure(row, 20, e))?,
        rationale: row.get(21)?,
        created_at_ms: u64::try_from(created_at_ms).map_err(|e| conversion_failure(row, 22, e))?,
    })
}

fn propagation_proposal_revision(
    proposal: &PropagationProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
) -> Result<ObjectRevision, HistoryStoreError> {
    let revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str(),
        event_id,
        RevisionOperation::Create,
    )
    .with_field(FieldDelta::new(
        "proposal_kind",
        None,
        Some(FieldValue::Text("propagation".to_string())),
    ))
    .with_field(FieldDelta::new(
        "action",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.action)?)),
    ))
    .with_field(FieldDelta::new(
        "target",
        None,
        Some(FieldValue::Text(target_label(&proposal.target))),
    ))
    .with_field(FieldDelta::new(
        "summary",
        None,
        Some(FieldValue::Text(proposal.summary.clone())),
    ))
    .with_field(FieldDelta::new(
        "status",
        None,
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
    ));
    let revision = match proposal.proposed_value.as_ref() {
        Some(value) => {
            revision.with_field(FieldDelta::new("proposed_value", None, Some(value.clone())))
        }
        None => revision,
    };
    Ok(match proposal.proposed_text.as_ref() {
        Some(text) => revision.with_field(FieldDelta::new(
            "proposed_text",
            None,
            Some(FieldValue::Text(text.clone())),
        )),
        None => revision,
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

fn conversion_failure<E>(row: &Row<'_>, index: usize, error: E) -> rusqlite::Error
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
pub(crate) enum PropagationProposalStoreError {
    #[error("{0}")]
    InvalidCommand(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Contract(#[from] eidetic_core::contracts::PropagationProposalContractError),
    #[error(transparent)]
    BibleGraphContract(#[from] eidetic_core::contracts::BibleGraphContractError),
    #[error(transparent)]
    BibleGraphCommand(#[from] crate::bible_graph_command::BibleGraphCommandError),
    #[error(transparent)]
    ScriptContract(#[from] eidetic_core::contracts::ScriptContractError),
    #[error(transparent)]
    SemanticDependencyContract(#[from] eidetic_core::contracts::SemanticDependencyContractError),
    #[error(transparent)]
    Target(#[from] PropagationProposalTargetError),
    #[error(transparent)]
    ScriptDocumentCommand(#[from] crate::script_document_command::ScriptDocumentCommandError),
}

#[cfg(test)]
#[path = "propagation_proposal_store_tests.rs"]
mod tests;
