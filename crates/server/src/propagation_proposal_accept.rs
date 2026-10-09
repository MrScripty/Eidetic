use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphField, BibleGraphPart, BibleGraphSnapshotField,
    BibleGraphSnapshotProjection, ChangeEvent, ChangeEventKind, CommandEnvelope, FieldDelta,
    FieldValue, ObjectKind, ObjectRevision, PropagationProposal, PropagationProposalAction,
    PropagationProposalTarget, RevisionOperation, ScriptBlockId, ScriptDocument,
    ScriptDocumentProjection, ScriptSegment, ScriptSpan, SemanticProposalStatus,
    SetBibleGraphFieldCommand, SetBibleGraphSnapshotFieldCommand, SetScriptBlockCommand,
};
use rusqlite::{OptionalExtension, Transaction};

use crate::bible_graph_command;
use crate::bible_graph_store;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_review::{
    load_pending_proposal, proposal_status_revision, update_proposal_status_in_transaction,
};
use crate::propagation_proposal_script_accept::script_block_target_for_proposal;
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::script_document_command;
use crate::script_segment_replace;
use crate::script_store;

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
    if proposal.script_review_binding.is_some() {
        return crate::script_impact_review::accept_bound_proposal(
            conn,
            command,
            &proposal,
            created_at_ms,
        );
    }
    crate::script_fact_proposal::validate_proposal(conn, &proposal)?;
    let accepted_target = accepted_target(conn, &proposal)?;

    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalAccepted,
        format!("accept propagation {}", proposal.summary),
    )
    .with_created_at_ms(created_at_ms);
    let proposal_revision =
        proposal_status_revision(&proposal, event.id, SemanticProposalStatus::Accepted, None)?;
    let mut revisions = vec![proposal_revision];
    revisions.extend(accepted_target.revisions(event.id)?);

    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.propagation_accept",
        &event,
        &revisions,
        |tx| {
            if proposal.script_fact_binding.is_some() {
                let current = load_pending_proposal(tx, &proposal.id)
                    .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?;
                if current != proposal {
                    return Err(crate::script_fact_evidence::stale());
                }
                crate::script_fact_proposal::validate_proposal(tx, &proposal)?;
            }
            update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Accepted,
            )?;
            if proposal.script_fact_binding.is_some() {
                accepted_target_for_fact(tx, &proposal, event.id)
            } else {
                accepted_target.apply(tx, event.id)
            }
        },
    )?)
}

pub(crate) enum AcceptedPropagationTarget {
    BibleField {
        command: SetBibleGraphFieldCommand,
        old_value: Option<FieldValue>,
    },
    BibleSnapshotField {
        command: SetBibleGraphSnapshotFieldCommand,
        old_value: Option<FieldValue>,
    },
    ScriptBlock {
        document_exists: bool,
        command: SetScriptBlockCommand,
        old_text: Option<FieldValue>,
        span: ScriptSpan,
        before: ScriptDocumentProjection,
    },
    ScriptSegment {
        document: ScriptDocument,
        segment: ScriptSegment,
        commands: Vec<SetScriptBlockCommand>,
        spans: Vec<Vec<ScriptSpan>>,
        retained_block_ids: Vec<ScriptBlockId>,
        retained_span_ids_by_block:
            Vec<(ScriptBlockId, Vec<eidetic_core::contracts::ScriptSpanId>)>,
        omitted_object_revisions: Vec<ObjectRevision>,
        before: ScriptDocumentProjection,
    },
}

impl AcceptedPropagationTarget {
    fn revisions(
        &self,
        event_id: eidetic_core::contracts::ChangeEventId,
    ) -> Result<Vec<ObjectRevision>, PropagationProposalStoreError> {
        match self {
            Self::BibleField { command, old_value } => Ok(vec![
                ObjectRevision::new(
                    ObjectKind::BiblePartField,
                    command.field_id.as_str(),
                    event_id,
                    RevisionOperation::Update,
                )
                .with_field(FieldDelta::new(
                    "value",
                    old_value.clone(),
                    command.value.clone(),
                )),
            ]),
            Self::BibleSnapshotField { command, old_value } => {
                Ok(vec![bible_graph_command::snapshot_revision(
                    command,
                    old_value.clone(),
                    true,
                    event_id,
                )])
            }
            Self::ScriptBlock {
                document_exists,
                command,
                old_text,
                span,
                before,
            } => {
                let document = script_document_command::command_document(command);
                let segment = script_document_command::command_segment(command);
                let block = script_document_command::command_block(command);
                Ok(vec![
                    script_document_command::document_revision(
                        &document,
                        *document_exists,
                        event_id,
                    ),
                    script_document_command::segment_revision(&segment, Some(before), event_id),
                    script_document_command::block_revision(&block, old_text.clone(), event_id),
                    script_document_command::span_revision(span, event_id),
                ])
            }
            Self::ScriptSegment {
                document,
                segment,
                commands,
                spans,
                omitted_object_revisions,
                before,
                ..
            } => {
                let mut revisions = vec![
                    script_document_command::document_revision(document, true, event_id),
                    script_document_command::segment_revision(segment, Some(before), event_id),
                ];
                for (command, block_spans) in commands.iter().zip(spans.iter()) {
                    let block = script_document_command::command_block(command);
                    let old_text = script_document_command::find_block_text(before, &block.id);
                    revisions.push(script_document_command::block_revision(
                        &block, old_text, event_id,
                    ));
                    revisions.extend(
                        block_spans
                            .iter()
                            .map(|span| script_document_command::span_revision(span, event_id)),
                    );
                }
                revisions.extend(
                    omitted_object_revisions
                        .iter()
                        .cloned()
                        .map(|mut revision| {
                            revision.change_event_id = event_id;
                            revision
                        }),
                );
                Ok(revisions)
            }
        }
    }

    fn apply(
        &self,
        tx: &Transaction<'_>,
        event_id: eidetic_core::contracts::ChangeEventId,
    ) -> Result<(), HistoryStoreError> {
        match self {
            Self::BibleField { command, .. } => {
                bible_graph_store::set_field_in_transaction(tx, command, event_id)
            }
            Self::BibleSnapshotField { command, .. } => {
                bible_graph_store::set_snapshot_field_in_transaction(tx, command, event_id)
            }
            Self::ScriptBlock { command, span, .. } => {
                let document = script_document_command::command_document(command);
                let segment = script_document_command::command_segment(command);
                let block = script_document_command::command_block(command);
                script_store::upsert_document_in_transaction(tx, &document, event_id)?;
                script_store::upsert_segment_in_transaction(tx, &segment, event_id)?;
                script_store::upsert_block_in_transaction(tx, &block, event_id)?;
                script_store::upsert_span_in_transaction(tx, span, event_id)?;
                Ok(())
            }
            Self::ScriptSegment {
                document,
                segment,
                commands,
                spans,
                retained_block_ids,
                retained_span_ids_by_block,
                ..
            } => {
                script_store::upsert_document_in_transaction(tx, document, event_id)?;
                script_store::upsert_segment_in_transaction(tx, segment, event_id)?;
                for (command, block_spans) in commands.iter().zip(spans.iter()) {
                    let block = script_document_command::command_block(command);
                    script_store::upsert_block_in_transaction(tx, &block, event_id)?;
                    for span in block_spans {
                        script_store::upsert_span_in_transaction(tx, span, event_id)?;
                    }
                }
                for (block_id, retained_span_ids) in retained_span_ids_by_block {
                    script_segment_replace::delete_omitted_block_spans_in_transaction(
                        tx,
                        block_id,
                        retained_span_ids,
                        event_id,
                    )?;
                }
                script_segment_replace::delete_omitted_segment_blocks_in_transaction(
                    tx,
                    &segment.id,
                    retained_block_ids,
                    event_id,
                )
            }
        }
    }
}

fn accepted_target(
    conn: &rusqlite::Connection,
    proposal: &PropagationProposal,
) -> Result<AcceptedPropagationTarget, PropagationProposalStoreError> {
    match (&proposal.action, &proposal.target) {
        (
            PropagationProposalAction::SetBibleField,
            PropagationProposalTarget::BibleField { .. },
        ) => {
            let command = bible_field_command_for_proposal(conn, proposal)?;
            let old_value = field_old_value(conn, &command)?;
            Ok(AcceptedPropagationTarget::BibleField { command, old_value })
        }
        (
            PropagationProposalAction::SetBibleSnapshotField,
            PropagationProposalTarget::BibleSnapshotField { .. },
        ) => bible_snapshot_field_target_for_proposal(conn, proposal),
        (
            PropagationProposalAction::PatchScriptBlock,
            PropagationProposalTarget::ScriptBlock { .. },
        ) => script_block_target_for_proposal(conn, proposal),
        (
            PropagationProposalAction::RegenerateScriptSegment,
            PropagationProposalTarget::ScriptSegment { .. },
        ) => crate::propagation_proposal_script_accept::script_segment_target_for_proposal(
            conn, proposal,
        ),
        _ => Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal action cannot be accepted yet: {}",
            proposal.id.as_str()
        ))),
    }
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

    let mut command = set_bible_field_command(node_id.clone(), part, field, value);
    // Detail projections can merge stored fields into schema-default parts.
    // Preserve the stored owner and its author-edited metadata when changing value.
    let owner: Option<(String, String, String, u32, Option<String>)> = conn
        .query_row(
            "SELECT node_id, part_key, name, sort_order, deleted_event_id FROM bible_graph_parts WHERE id=?1",
            [field.part_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .optional()?;
    match owner {
        Some((owner_node, owner_key, name, sort_order, None))
            if owner_node == node_id.as_str() && owner_key == part_key.as_str() =>
        {
            command.part_name = name;
            command.part_sort_order = sort_order;
        }
        None if proposal.script_fact_binding.is_none() && field.part_id == part.id => {
            // Ordinary proposals may materialize an unpersisted schema field.
            let persisted: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM bible_graph_fields WHERE id=?1)",
                [field.id.as_str()],
                |row| row.get(0),
            )?;
            if persisted {
                return Err(crate::script_fact_evidence::stale().into());
            }
        }
        _ => return Err(crate::script_fact_evidence::stale().into()),
    }
    Ok(command)
}

fn bible_snapshot_field_target_for_proposal(
    conn: &rusqlite::Connection,
    proposal: &PropagationProposal,
) -> Result<AcceptedPropagationTarget, PropagationProposalStoreError> {
    let PropagationProposalTarget::BibleSnapshotField {
        node_id,
        snapshot_id,
        part_key,
        field_key,
        field_id,
    } = &proposal.target
    else {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal target is not a bible snapshot field: {}",
            proposal.id.as_str()
        )));
    };
    let value = proposal.proposed_value.clone().ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(
            "accepted bible snapshot field propagation proposal requires proposed_value"
                .to_string(),
        )
    })?;
    let projection =
        bible_graph_store::load_node_detail_projection(conn, node_id)?.ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "bible graph node does not exist: {}",
                node_id.as_str()
            ))
        })?;
    let (snapshot, field) = snapshot_field_context(
        &projection.snapshots,
        snapshot_id,
        part_key,
        field_key,
        field_id,
    )?;
    let old_value = field.value.clone();
    let command = set_snapshot_field_command(node_id.clone(), snapshot, field, value);
    bible_graph_command::validate_snapshot_field_command(&command)?;
    bible_graph_command::validate_snapshot_field_schema(&projection, &command)?;
    Ok(AcceptedPropagationTarget::BibleSnapshotField { command, old_value })
}

fn snapshot_field_context<'a>(
    snapshots: &'a [BibleGraphSnapshotProjection],
    snapshot_id: &eidetic_core::contracts::BibleGraphSnapshotId,
    part_key: &eidetic_core::contracts::BibleGraphPartKey,
    field_key: &eidetic_core::contracts::BibleGraphFieldKey,
    field_id: &eidetic_core::contracts::BibleGraphSnapshotFieldId,
) -> Result<
    (
        &'a BibleGraphSnapshotProjection,
        &'a BibleGraphSnapshotField,
    ),
    PropagationProposalStoreError,
> {
    let snapshot = snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot.id == *snapshot_id)
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "bible graph snapshot does not exist: {}",
                snapshot_id.as_str()
            ))
        })?;
    let field = snapshot
        .fields
        .iter()
        .find(|field| {
            field.id == *field_id && field.part_key == *part_key && field.field_key == *field_key
        })
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "bible graph snapshot field does not exist in projection: {}.{}",
                part_key.as_str(),
                field_key.as_str()
            ))
        })?;
    Ok((snapshot, field))
}

fn set_snapshot_field_command(
    node_id: eidetic_core::contracts::BibleGraphNodeId,
    snapshot: &BibleGraphSnapshotProjection,
    field: &BibleGraphSnapshotField,
    value: FieldValue,
) -> SetBibleGraphSnapshotFieldCommand {
    SetBibleGraphSnapshotFieldCommand {
        snapshot_id: snapshot.snapshot.id.clone(),
        node_id,
        at_ms: snapshot.snapshot.at_ms,
        label: snapshot.snapshot.label.clone(),
        snapshot_sort_order: snapshot.snapshot.sort_order,
        field_id: field.id.clone(),
        part_key: field.part_key.clone(),
        part_name: field.part_name.clone(),
        field_key: field.field_key.clone(),
        value: Some(value),
        field_sort_order: field.sort_order,
    }
}

fn set_bible_field_command(
    node_id: eidetic_core::contracts::BibleGraphNodeId,
    part: &BibleGraphPart,
    field: &BibleGraphField,
    value: FieldValue,
) -> SetBibleGraphFieldCommand {
    SetBibleGraphFieldCommand {
        node_id,
        part_id: field.part_id.clone(),
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

#[cfg(test)]
#[path = "propagation_proposal_accept_tests.rs"]
mod tests;

fn accepted_target_for_fact(
    tx: &Transaction<'_>,
    proposal: &PropagationProposal,
    event: eidetic_core::contracts::ChangeEventId,
) -> Result<(), HistoryStoreError> {
    let current = accepted_target(tx, proposal)
        .map_err(|e| HistoryStoreError::InvalidValue(e.to_string()))?;
    current.apply(tx, event)
}
