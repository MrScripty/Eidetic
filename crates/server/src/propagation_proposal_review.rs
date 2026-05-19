use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphField, BibleGraphPart, ChangeEvent,
    ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue, ObjectKind, ObjectRevision,
    PropagationProposal, PropagationProposalAction, PropagationProposalId,
    PropagationProposalTarget, RejectPropagationProposalCommand, RevisionOperation,
    SemanticProposalStatus, SetBibleGraphFieldCommand,
};
use rusqlite::{Transaction, params};
use serde::Serialize;

use crate::bible_graph_store;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_store::{
    self, PropagationProposalStoreError, load_propagation_proposal,
};

pub(crate) fn record_accept_propagation_proposal(
    conn: &mut rusqlite::Connection,
    command: &CommandEnvelope<AcceptPropagationProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_store::create_schema(conn)?;
    bible_graph_store::create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.propagation_accept")?
    {
        return Ok(outcome);
    }
    let proposal = load_pending_proposal(conn, &command.payload.proposal_id)?;
    let field_command = bible_field_command_for_proposal(conn, &proposal)?;
    let old_value = field_old_value(conn, &field_command)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalAccepted,
        format!("accept propagation {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let proposal_revision =
        proposal_status_revision(&proposal, event.id, SemanticProposalStatus::Accepted, None)?;
    let field_revision = ObjectRevision::new(
        ObjectKind::BiblePartField,
        field_command.field_id.as_str(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "value",
        old_value,
        field_command.value.clone(),
    ));

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.propagation_accept",
        &event,
        &[proposal_revision, field_revision],
        |tx| {
            update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Accepted,
            )?;
            bible_graph_store::set_field_in_transaction(tx, &field_command, event.id)
        },
    )?)
}

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
    let proposal = load_pending_proposal(conn, &command.payload.proposal_id)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalRejected,
        format!("reject propagation {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let revision = proposal_status_revision(
        &proposal,
        event.id,
        SemanticProposalStatus::Rejected,
        command.payload.reason.as_deref(),
    )?;

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

fn load_pending_proposal(
    conn: &rusqlite::Connection,
    proposal_id: &PropagationProposalId,
) -> Result<PropagationProposal, PropagationProposalStoreError> {
    let proposal = load_propagation_proposal(conn, proposal_id)?.ok_or_else(|| {
        PropagationProposalStoreError::NotFound(format!(
            "propagation proposal not found: {}",
            proposal_id.as_str()
        ))
    })?;
    if proposal.status != SemanticProposalStatus::Pending {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal is not pending: {}",
            proposal.id.as_str()
        )));
    }
    Ok(proposal)
}

fn bible_field_command_for_proposal(
    conn: &rusqlite::Connection,
    proposal: &PropagationProposal,
) -> Result<SetBibleGraphFieldCommand, PropagationProposalStoreError> {
    if proposal.action != PropagationProposalAction::SetBibleField {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal action cannot be accepted by bible field handler: {}",
            proposal.id.as_str()
        )));
    }
    let PropagationProposalTarget::BibleField {
        node_id,
        part_key,
        field_key,
        field_id,
    } = &proposal.target
    else {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal target is not a bible field: {}",
            proposal.id.as_str()
        )));
    };
    let value = proposal.proposed_value.clone().ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(
            "accepted bible field propagation proposal requires proposed_value".to_string(),
        )
    })?;
    let projection =
        bible_graph_store::load_node_detail_projection(conn, node_id)?.ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "bible graph node does not exist: {}",
                node_id.as_str()
            ))
        })?;
    let Some((part, field)) = projection
        .parts
        .iter()
        .find(|part| part.part.part_key == *part_key)
        .and_then(|part| {
            part.fields
                .iter()
                .find(|field| {
                    field.field_key == *field_key
                        && field_id
                            .as_ref()
                            .is_none_or(|expected_id| field.id == *expected_id)
                })
                .map(|field| (&part.part, field))
        })
    else {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "bible graph field does not exist in projection: {}.{}",
            part_key.as_str(),
            field_key.as_str()
        )));
    };

    Ok(set_bible_field_command(node_id.clone(), part, field, value))
}

fn set_bible_field_command(
    node_id: eidetic_core::contracts::BibleGraphNodeId,
    part: &BibleGraphPart,
    field: &BibleGraphField,
    value: FieldValue,
) -> SetBibleGraphFieldCommand {
    SetBibleGraphFieldCommand {
        node_id,
        part_id: part.id.clone(),
        part_key: part.part_key.clone(),
        part_name: part.name.clone(),
        part_sort_order: part.sort_order,
        field_id: field.id.clone(),
        field_key: field.field_key.clone(),
        value: Some(value),
        field_sort_order: field.sort_order,
    }
}

fn field_old_value(
    conn: &rusqlite::Connection,
    command: &SetBibleGraphFieldCommand,
) -> Result<Option<FieldValue>, PropagationProposalStoreError> {
    let projection = bible_graph_store::load_node_detail_projection(conn, &command.node_id)?
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "bible graph node does not exist: {}",
                command.node_id.as_str()
            ))
        })?;
    Ok(projection
        .parts
        .iter()
        .flat_map(|part| part.fields.iter())
        .find(|field| field.id == command.field_id)
        .and_then(|field| field.value.clone()))
}

fn proposal_status_revision(
    proposal: &PropagationProposal,
    event_id: eidetic_core::contracts::ChangeEventId,
    new_status: SemanticProposalStatus,
    rejection_reason: Option<&str>,
) -> Result<ObjectRevision, HistoryStoreError> {
    let revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        proposal.id.as_str(),
        event_id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "status",
        Some(FieldValue::Text(encode_string_enum(&proposal.status)?)),
        Some(FieldValue::Text(encode_string_enum(&new_status)?)),
    ));
    Ok(match rejection_reason {
        Some(reason) => revision.with_field(FieldDelta::new(
            "rejection_reason",
            None,
            Some(FieldValue::Text(reason.to_string())),
        )),
        None => revision,
    })
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
