use eidetic_core::contracts::{
    ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind,
    ObjectRevision, PropagationProposalId, RejectPropagationProposalCommand, RevisionOperation,
    SemanticProposalStatus,
};
use rusqlite::{Transaction, params};
use serde::Serialize;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_store::{
    self, PropagationProposalStoreError, load_propagation_proposal,
};

pub(crate) fn record_reject_propagation_proposal(
    conn: &mut rusqlite::Connection,
    command: &CommandEnvelope<RejectPropagationProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_store::create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.propagation_reject")?
    {
        return Ok(outcome);
    }
    let proposal =
        load_propagation_proposal(conn, &command.payload.proposal_id)?.ok_or_else(|| {
            PropagationProposalStoreError::NotFound(format!(
                "propagation proposal not found: {}",
                command.payload.proposal_id.as_str()
            ))
        })?;
    if proposal.status != SemanticProposalStatus::Pending {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal is not pending: {}",
            proposal.id.as_str()
        )));
    }

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalRejected,
        format!("reject propagation {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let mut revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str(),
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
        "semantic.propagation_reject",
        &event,
        &[revision],
        |tx| {
            update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Rejected,
            )
        },
    )?)
}

fn update_proposal_status_in_transaction(
    tx: &Transaction<'_>,
    proposal_id: &PropagationProposalId,
    new_status: SemanticProposalStatus,
) -> Result<(), HistoryStoreError> {
    let updated = tx.execute(
        "UPDATE propagation_proposals
         SET status = ?1
         WHERE id = ?2 AND status = ?3",
        params![
            encode_string_enum(&new_status)?,
            proposal_id.as_str(),
            encode_string_enum(&SemanticProposalStatus::Pending)?
        ],
    )?;
    if updated != 1 {
        return Err(HistoryStoreError::InvalidValue(format!(
            "propagation proposal status changed before update: {}",
            proposal_id.as_str()
        )));
    }
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
