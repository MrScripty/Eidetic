use eidetic_core::contracts::{
    ChangeEvent, ChangeEventId, ChangeEventKind, CommandEnvelope, FieldDelta, FieldValue,
    ObjectKind, ObjectRevision, PropagationProposal, RevisionOperation, SemanticDependencyId,
    SemanticProposalStatus, UpdatePropagationProposalCommand,
};
use rusqlite::{Transaction, params};
use serde::Serialize;

use crate::bible_graph_value_store::SqlGraphFieldValue;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_review::load_pending_proposal;
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::propagation_proposal_target::{SqlPropagationTarget, target_label};

pub(crate) fn record_update_propagation_proposal(
    conn: &mut rusqlite::Connection,
    command: &CommandEnvelope<UpdatePropagationProposalCommand>,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_store::create_schema(conn)?;
    if let Some(outcome) =
        history_store::check_recorded_command(conn, command, "semantic.propagation_update")?
    {
        return Ok(outcome);
    }

    let existing = load_pending_proposal(conn, &command.payload.proposal_id)?;
    let updated = updated_proposal(&existing, &command.payload);
    propagation_proposal_store::validate_proposal_shape(
        &updated.action,
        &updated.target,
        &updated.summary,
        updated.proposed_value.as_ref(),
        updated.proposed_text.as_deref(),
        updated.proposed_script_patch.as_ref(),
    )?;
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalUpdated,
        format!("update propagation {}", updated.summary),
    )
    .with_created_at_ms(created_at_ms);
    let revision = update_revision(&existing, &updated, event.id)?;
    if revision.fields.is_empty() {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal update has no changes: {}",
            existing.id.as_str()
        )));
    }

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.propagation_update",
        &event,
        &[revision],
        |tx| update_proposal_in_transaction(tx, &updated),
    )?)
}

fn updated_proposal(
    existing: &PropagationProposal,
    command: &UpdatePropagationProposalCommand,
) -> PropagationProposal {
    PropagationProposal {
        id: command.proposal_id.clone(),
        action: command.action.clone(),
        target: command.target.clone(),
        status: existing.status.clone(),
        summary: command.summary.clone(),
        proposed_value: command.proposed_value.clone(),
        proposed_text: command.proposed_text.clone(),
        proposed_script_patch: command.proposed_script_patch.clone(),
        source_dependency_id: command.source_dependency_id.clone(),
        source_event_id: command.source_event_id,
        rationale: command.rationale.clone(),
        created_at_ms: existing.created_at_ms,
    }
}

fn update_revision(
    existing: &PropagationProposal,
    updated: &PropagationProposal,
    event_id: ChangeEventId,
) -> Result<ObjectRevision, PropagationProposalStoreError> {
    let mut revision = ObjectRevision::new(
        ObjectKind::SemanticProposal,
        existing.id.as_str(),
        event_id,
        RevisionOperation::Update,
    );

    if existing.action != updated.action {
        revision = revision.with_field(FieldDelta::new(
            "action",
            Some(FieldValue::Text(encode_string_enum(&existing.action)?)),
            Some(FieldValue::Text(encode_string_enum(&updated.action)?)),
        ));
    }
    if existing.target != updated.target {
        revision = revision.with_field(FieldDelta::new(
            "target",
            Some(FieldValue::Text(target_label(&existing.target))),
            Some(FieldValue::Text(target_label(&updated.target))),
        ));
    }
    if existing.summary != updated.summary {
        revision = revision.with_field(FieldDelta::new(
            "summary",
            Some(FieldValue::Text(existing.summary.clone())),
            Some(FieldValue::Text(updated.summary.clone())),
        ));
    }
    if existing.proposed_value != updated.proposed_value {
        revision = revision.with_field(FieldDelta::new(
            "proposed_value",
            existing.proposed_value.clone(),
            updated.proposed_value.clone(),
        ));
    }
    if existing.proposed_text != updated.proposed_text {
        revision = revision.with_field(FieldDelta::new(
            "proposed_text",
            optional_text(existing.proposed_text.as_ref()),
            optional_text(updated.proposed_text.as_ref()),
        ));
    }
    if existing.proposed_script_patch != updated.proposed_script_patch {
        revision = revision.with_field(FieldDelta::new(
            "proposed_script_patch",
            optional_json(existing.proposed_script_patch.as_ref())?,
            optional_json(updated.proposed_script_patch.as_ref())?,
        ));
    }
    if existing.source_dependency_id != updated.source_dependency_id {
        revision = revision.with_field(FieldDelta::new(
            "source_dependency_id",
            optional_dependency(existing.source_dependency_id.as_ref()),
            optional_dependency(updated.source_dependency_id.as_ref()),
        ));
    }
    if existing.source_event_id != updated.source_event_id {
        revision = revision.with_field(FieldDelta::new(
            "source_event_id",
            optional_event_id(existing.source_event_id),
            optional_event_id(updated.source_event_id),
        ));
    }
    if existing.rationale != updated.rationale {
        revision = revision.with_field(FieldDelta::new(
            "rationale",
            optional_text(existing.rationale.as_ref()),
            optional_text(updated.rationale.as_ref()),
        ));
    }

    Ok(revision)
}

fn update_proposal_in_transaction(
    tx: &Transaction<'_>,
    proposal: &PropagationProposal,
) -> Result<(), HistoryStoreError> {
    let target = SqlPropagationTarget::from_target(&proposal.target);
    let proposed_value = SqlGraphFieldValue::from_field_value(proposal.proposed_value.as_ref())?;
    let proposed_script_patch = proposal
        .proposed_script_patch
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let updated = tx.execute(
        "UPDATE propagation_proposals
         SET action = ?1,
             target_kind = ?2,
             target_id = ?3,
             target_part_key = ?4,
             target_field_key = ?5,
             target_field_id = ?6,
             target_snapshot_id = ?7,
             summary = ?8,
             proposed_value_type = ?9,
             proposed_value_text = ?10,
             proposed_value_integer = ?11,
             proposed_value_number = ?12,
             proposed_value_bool = ?13,
             proposed_value_ref_kind = ?14,
             proposed_value_ref_id = ?15,
             proposed_value_asset_ref = ?16,
             proposed_text = ?17,
             proposed_script_patch_json = ?18,
             source_dependency_id = ?19,
             source_event_id = ?20,
             rationale = ?21
         WHERE id = ?22 AND status = ?23",
        params![
            encode_string_enum(&proposal.action)?,
            target.kind,
            target.id,
            target.part_key,
            target.field_key,
            target.field_id,
            target.snapshot_id,
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
            proposed_script_patch,
            proposal.source_dependency_id.as_ref().map(|id| id.as_str()),
            proposal.source_event_id.map(|id| id.0.to_string()),
            proposal.rationale,
            proposal.id.as_str(),
            encode_string_enum(&SemanticProposalStatus::Pending)?,
        ],
    )?;
    if updated != 1 {
        return Err(HistoryStoreError::InvalidValue(format!(
            "propagation proposal changed before update: {}",
            proposal.id.as_str()
        )));
    }
    Ok(())
}

fn optional_text(value: Option<&String>) -> Option<FieldValue> {
    value.cloned().map(FieldValue::Text)
}

fn optional_json<T>(value: Option<&T>) -> Result<Option<FieldValue>, HistoryStoreError>
where
    T: serde::Serialize,
{
    Ok(value
        .map(serde_json::to_string)
        .transpose()
        .map(|value| value.map(FieldValue::Text))?)
}

fn optional_dependency(value: Option<&SemanticDependencyId>) -> Option<FieldValue> {
    value.map(|id| FieldValue::Text(id.as_str().to_string()))
}

fn optional_event_id(value: Option<ChangeEventId>) -> Option<FieldValue> {
    value.map(|id| FieldValue::Text(id.0.to_string()))
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
