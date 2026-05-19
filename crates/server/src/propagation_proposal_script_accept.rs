use eidetic_core::contracts::{
    PropagationProposal, PropagationProposalTarget, ScriptBlockId, ScriptBlockProjection,
    ScriptDocumentProjection, ScriptSegmentProjection, ScriptSpanProvenance, SetScriptBlockCommand,
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
