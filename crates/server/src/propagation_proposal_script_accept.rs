use eidetic_core::contracts::{
    FieldValue, ObjectKind, ObjectRevision, PropagationProposal, PropagationProposalTarget,
    RevisionOperation, ScriptBlockId, ScriptBlockProjection, ScriptDocumentProjection, ScriptPatch,
    ScriptSegmentId, ScriptSegmentProjection, ScriptSpanProvenance, SetScriptBlockCommand,
};

use crate::propagation_proposal_accept::AcceptedPropagationTarget;
use crate::propagation_proposal_store::PropagationProposalStoreError;
use crate::script_document_command;
use crate::script_store;

pub(crate) fn script_block_target_for_proposal(
    conn: &rusqlite::Connection,
    proposal: &PropagationProposal,
) -> Result<AcceptedPropagationTarget, PropagationProposalStoreError> {
    let PropagationProposalTarget::ScriptBlock { block_id } = &proposal.target else {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal target is not a script block: {}",
            proposal.id.as_str()
        )));
    };
    let proposed_text = proposal.proposed_text.as_ref().ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(
            "accepted script block propagation proposal requires proposed_text".to_string(),
        )
    })?;
    if proposed_text.trim().is_empty() {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "accepted script block propagation proposal requires proposed_text".to_string(),
        ));
    }

    let document_id = script_store::document_id_for_block(conn, block_id)?.ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(format!(
            "script block does not exist: {}",
            block_id.as_str()
        ))
    })?;
    let before = script_store::load_document_projection(conn, &document_id)?.ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(format!(
            "script document does not exist for block: {}",
            block_id.as_str()
        ))
    })?;
    let (segment_projection, block_projection) = script_block_context(&before, block_id)?;
    let command = set_script_block_command(
        &before,
        segment_projection,
        block_projection,
        proposed_text.clone(),
    );
    script_document_command::validate_block_command(&command)?;
    script_document_command::validate_locked_spans(Some(&before), &command)?;
    let block = script_document_command::command_block(&command);
    let span = script_document_command::generated_span_for_block(
        &block,
        ScriptSpanProvenance::AiGenerated,
    )?;
    let old_text = script_document_command::find_block_text(&before, &command.block_id);
    Ok(AcceptedPropagationTarget::ScriptBlock {
        document_exists: true,
        command,
        old_text,
        span,
        before,
    })
}

pub(crate) fn script_segment_target_for_proposal(
    conn: &rusqlite::Connection,
    proposal: &PropagationProposal,
) -> Result<AcceptedPropagationTarget, PropagationProposalStoreError> {
    let PropagationProposalTarget::ScriptSegment { segment_id } = &proposal.target else {
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "propagation proposal target is not a script segment: {}",
            proposal.id.as_str()
        )));
    };
    let patch = proposal.proposed_script_patch.as_ref().ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand(
            "accepted script segment propagation proposal requires proposed_script_patch"
                .to_string(),
        )
    })?;
    let patched_segment = single_patch_segment(patch, segment_id)?;
    let before =
        script_store::load_document_projection(conn, &patch.document_id)?.ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "script document does not exist for segment patch: {}",
                patch.document_id.as_str()
            ))
        })?;
    let existing_segment = before
        .segments
        .iter()
        .find(|segment| segment.segment.id == *segment_id)
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "script segment does not exist in document projection: {}",
                segment_id.as_str()
            ))
        })?;
    if patched_segment.segment.document_id != patch.document_id {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "script segment patch document_id does not match patch document".to_string(),
        ));
    }

    let document = before.document.clone();
    let segment = patched_segment.segment.clone();
    let mut commands = Vec::new();
    let mut spans = Vec::new();
    let mut retained_block_ids = Vec::new();
    for block in &patched_segment.blocks {
        if block.block.segment_id != *segment_id {
            return Err(PropagationProposalStoreError::InvalidCommand(format!(
                "script block patch segment_id does not match target segment: {}",
                block.block.id.as_str()
            )));
        }
        let command =
            set_script_block_command(&before, patched_segment, block, block.block.text.clone());
        script_document_command::validate_block_command(&command)?;
        script_document_command::validate_locked_spans(Some(&before), &command)?;
        let span = script_document_command::generated_span_for_block(
            &script_document_command::command_block(&command),
            ScriptSpanProvenance::AiGenerated,
        )?;
        retained_block_ids.push(block.block.id.clone());
        commands.push(command);
        spans.push(span);
    }
    reject_omitted_locked_blocks(existing_segment, &retained_block_ids)?;
    let omitted_object_revisions = omitted_object_revisions(existing_segment, &retained_block_ids);

    Ok(AcceptedPropagationTarget::ScriptSegment {
        document,
        segment,
        commands,
        spans,
        retained_block_ids,
        omitted_object_revisions,
        before,
    })
}

fn single_patch_segment<'a>(
    patch: &'a ScriptPatch,
    segment_id: &ScriptSegmentId,
) -> Result<&'a ScriptSegmentProjection, PropagationProposalStoreError> {
    if patch.segments.len() != 1 {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "script segment regeneration patch must contain exactly one segment".to_string(),
        ));
    }
    let segment = &patch.segments[0];
    if segment.segment.id != *segment_id {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "script segment regeneration patch does not match target segment".to_string(),
        ));
    }
    Ok(segment)
}

fn reject_omitted_locked_blocks(
    existing_segment: &ScriptSegmentProjection,
    retained_block_ids: &[ScriptBlockId],
) -> Result<(), PropagationProposalStoreError> {
    for block in &existing_segment.blocks {
        if retained_block_ids.contains(&block.block.id) || block.locks.is_empty() {
            continue;
        }
        return Err(PropagationProposalStoreError::InvalidCommand(format!(
            "script segment regeneration would remove locked block: {}",
            block.block.id.as_str()
        )));
    }
    Ok(())
}

fn omitted_object_revisions(
    existing_segment: &ScriptSegmentProjection,
    retained_block_ids: &[ScriptBlockId],
) -> Vec<ObjectRevision> {
    let mut revisions = Vec::new();
    for block in &existing_segment.blocks {
        if retained_block_ids.contains(&block.block.id) {
            continue;
        }
        for lock in &block.locks {
            revisions.push(delete_revision(ObjectKind::ScriptLock, lock.id.as_str()));
        }
        for span in &block.spans {
            revisions.push(delete_revision(ObjectKind::ScriptSpan, span.id.as_str()));
        }
        revisions.push(delete_revision(
            ObjectKind::ScriptBlock,
            block.block.id.as_str(),
        ));
    }
    revisions
}

fn delete_revision(object_kind: ObjectKind, object_id: &str) -> ObjectRevision {
    ObjectRevision::new(
        object_kind,
        object_id,
        eidetic_core::contracts::ChangeEventId::new(),
        RevisionOperation::Delete,
    )
    .with_field(eidetic_core::contracts::FieldDelta::new(
        "deleted",
        Some(FieldValue::Bool(false)),
        Some(FieldValue::Bool(true)),
    ))
}

fn script_block_context<'a>(
    projection: &'a ScriptDocumentProjection,
    block_id: &ScriptBlockId,
) -> Result<(&'a ScriptSegmentProjection, &'a ScriptBlockProjection), PropagationProposalStoreError>
{
    projection
        .segments
        .iter()
        .find_map(|segment| {
            segment
                .blocks
                .iter()
                .find(|block| block.block.id == *block_id)
                .map(|block| (segment, block))
        })
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand(format!(
                "script block does not exist in document projection: {}",
                block_id.as_str()
            ))
        })
}

fn set_script_block_command(
    projection: &ScriptDocumentProjection,
    segment: &ScriptSegmentProjection,
    block: &ScriptBlockProjection,
    text: String,
) -> SetScriptBlockCommand {
    SetScriptBlockCommand {
        document_id: projection.document.id.clone(),
        document_title: projection.document.title.clone(),
        document_sort_order: projection.document.sort_order,
        segment_id: segment.segment.id.clone(),
        source_node_id: segment.segment.source_node_id.clone(),
        segment_start_ms: segment.segment.start_ms,
        segment_end_ms: segment.segment.end_ms,
        segment_status: segment.segment.status.clone(),
        segment_sort_order: segment.segment.sort_order,
        block_id: block.block.id.clone(),
        block_kind: block.block.block_kind.clone(),
        text,
        sort_order: block.block.sort_order,
        span_provenance: ScriptSpanProvenance::AiGenerated,
    }
}
