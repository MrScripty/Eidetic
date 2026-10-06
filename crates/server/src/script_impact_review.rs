use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;
use rusqlite::Connection;

use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_review;
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use crate::{ai_script_context, script_document_command, script_generation_lineage, script_store};

pub(crate) const REQUEST_TYPE: &str = "script.impact_proposal";

pub(crate) fn capture(
    conn: &Connection,
    request: &RequestScriptImpactProposalCommand,
) -> Result<ScriptImpactProposalBinding, HistoryStoreError> {
    script_store::create_schema(conn)?;
    crate::bible_graph_store::create_schema(conn)?;
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let binding = capture_in_snapshot(&tx, request)?;
        tx.commit()?;
        return Ok(binding);
    }
    capture_in_snapshot(conn, request)
}

fn capture_in_snapshot(
    conn: &Connection,
    request: &RequestScriptImpactProposalCommand,
) -> Result<ScriptImpactProposalBinding, HistoryStoreError> {
    let (_, segment, block, segment_revision) = target(conn, request)?;
    if block.revision_event_id != Some(request.expected_block_revision_event_id) {
        return Err(stale());
    }
    let impact = segment.impact.ok_or_else(stale)?;
    if impact.generation_event_id != request.generation_event_id
        || impact.output_block_id.as_ref() != Some(&request.block_id)
    {
        return Err(stale());
    }
    let bible_relationship_absence_revisions =
        Some(crate::bible_relationship_lineage::capture_absence_revisions(conn, &impact.causes)?);
    let cause = impact
        .causes
        .into_iter()
        .find(|cause| cause.dependency_id == request.dependency_id)
        .ok_or_else(stale)?;
    let source_node = segment.segment.source_node_id.as_deref().ok_or_else(|| {
        HistoryStoreError::InvalidValue("screenplay segment has no generation context node".into())
    })?;
    let node_id = NodeId(
        uuid::Uuid::parse_str(source_node)
            .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))?,
    );
    let node = crate::timeline_node_store::load_node_ancestor_stack(conn, node_id)?
        .into_iter()
        .find(|node| node.id == node_id)
        .ok_or_else(stale)?;
    if node.locked {
        return Err(HistoryStoreError::InvalidValue("node is locked".into()));
    }
    let mut script_inputs = ai_script_context::load_script_context(
        conn,
        node_id,
        segment.segment.start_ms,
        segment.segment.end_ms,
    )?;
    let script_context_scope = Some(crate::script_context_scope::capture(
        conn,
        node_id,
        segment.segment.start_ms,
        segment.segment.end_ms,
        &script_inputs,
    )?);
    // A changed input may have moved outside the normal continuity window.
    // Include that proven source explicitly; deletion is represented by the cause.
    if cause.current_revision_event_id.is_some()
        && matches!(
            cause.input,
            SemanticDependencyEndpoint::ScriptBlock { .. }
                | SemanticDependencyEndpoint::ScriptSegment { .. }
        )
    {
        let source_segment: String = match &cause.input {
            SemanticDependencyEndpoint::ScriptBlock { block_id } => conn.query_row(
                "SELECT segment_id FROM script_blocks WHERE id = ?1",
                [block_id.as_str()],
                |row| row.get(0),
            )?,
            SemanticDependencyEndpoint::ScriptSegment { segment_id } => segment_id.as_str().into(),
            _ => {
                return Err(HistoryStoreError::InvalidValue(
                    "unsupported screenplay review cause".into(),
                ));
            }
        };
        let (start, end, source): (u64, u64, Option<String>) = conn.query_row(
            "SELECT start_ms, end_ms, source_node_id FROM script_segments WHERE id = ?1",
            [&source_segment],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let source_node = source
            .as_deref()
            .map(uuid::Uuid::parse_str)
            .transpose()
            .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))?
            .unwrap_or_else(uuid::Uuid::nil);
        for input in ai_script_context::load_script_context(conn, NodeId(source_node), start, end)?
        {
            if input.segment_id.as_str() != source_segment {
                continue;
            }
            if let SemanticDependencyEndpoint::ScriptBlock { block_id } = &cause.input
                && input.block_id != *block_id
            {
                continue;
            }
            if !script_inputs
                .iter()
                .any(|existing| existing.block_id == input.block_id)
            {
                script_inputs.push(input);
            }
        }
    }
    let bible_context = crate::ai_context_projection::load_ai_bible_context_projection(
        conn,
        node_id,
        request.story_time_ms,
    )?;
    if let SemanticDependencyEndpoint::BibleField { node_id, .. } = &cause.input
        && cause.current_revision_event_id.is_some()
        && !bible_context
            .payload
            .nodes
            .iter()
            .any(|node| node.node_id == *node_id)
    {
        return Err(HistoryStoreError::InvalidValue(
            "Bible review source is outside the current context; restore its context before previewing".into(),
        ));
    }
    let bible_relationship_inputs = Some(crate::bible_relationship_lineage::capture(
        conn,
        &bible_context.payload,
    )?);
    crate::bible_relationship_lineage::validate_preview_inputs(
        conn,
        request.generation_event_id,
        &request.segment_id,
        bible_relationship_inputs.as_ref().unwrap(),
    )?;
    let bible_inputs = crate::bible_field_lineage::capture(conn, &bible_context.payload)?;
    crate::bible_context_scope::validate_preview_inputs(
        conn,
        request.generation_event_id,
        &request.segment_id,
        &bible_inputs,
    )?;
    let bible_context_scope = Some(crate::bible_context_scope::capture(
        conn,
        node_id,
        &bible_inputs,
    )?);
    Ok(ScriptImpactProposalBinding {
        request: request.clone(),
        cause,
        target_segment_revision_event_id: segment_revision,
        script_inputs,
        bible_context,
        bible_inputs,
        bible_relationship_inputs,
        bible_relationship_absence_revisions,
        bible_context_scope,
        script_context_scope,
    })
}

pub(crate) fn validate_binding(
    conn: &Connection,
    binding: &ScriptImpactProposalBinding,
) -> Result<(), HistoryStoreError> {
    let mut current = capture(conn, &binding.request)?;
    // Global projection clocks include unrelated graph edits. Payload and exact
    // consumed revisions remain authoritative, including edge edit/restore ABA.
    current.bible_context.version = binding.bible_context.version;
    current.bible_context.change_event_id = binding.bible_context.change_event_id;
    if current != *binding {
        return Err(stale());
    }
    Ok(())
}

fn stale() -> HistoryStoreError {
    HistoryStoreError::InvalidValue("screenplay proposal is stale; request a fresh preview".into())
}

fn target(
    conn: &Connection,
    request: &RequestScriptImpactProposalCommand,
) -> Result<
    (
        ScriptDocumentProjection,
        ScriptSegmentProjection,
        ScriptBlockProjection,
        ChangeEventId,
    ),
    HistoryStoreError,
> {
    let document =
        script_store::load_document_projection(conn, &request.document_id)?.ok_or_else(stale)?;
    let segment = document
        .segments
        .iter()
        .find(|segment| segment.segment.id == request.segment_id)
        .cloned()
        .ok_or_else(stale)?;
    let block = segment
        .blocks
        .iter()
        .find(|block| block.block.id == request.block_id)
        .cloned()
        .ok_or_else(stale)?;
    let revision: String = conn.query_row(
        "SELECT updated_event_id FROM script_segments WHERE id = ?1",
        [request.segment_id.as_str()],
        |row| row.get(0),
    )?;
    let revision = ChangeEventId(
        uuid::Uuid::parse_str(&revision)
            .map_err(|error| HistoryStoreError::InvalidId(error.to_string()))?,
    );
    Ok((document, segment, block, revision))
}

pub(crate) fn record_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<RequestScriptImpactProposalCommand>,
    binding: ScriptImpactProposalBinding,
    text: String,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_store::create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, REQUEST_TYPE)? {
        return Ok(outcome);
    }
    if binding.request != command.payload || text.trim().is_empty() {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "invalid screenplay proposal preview".into(),
        ));
    }
    validate_binding(conn, &binding)?;
    let proposal = PropagationProposal {
        id: command.payload.proposal_id.clone(),
        action: PropagationProposalAction::PatchScriptBlock,
        target: PropagationProposalTarget::ScriptBlock {
            block_id: command.payload.block_id.clone(),
        },
        status: SemanticProposalStatus::Pending,
        summary: "Update screenplay for a changed input".into(),
        proposed_value: None,
        proposed_text: Some(text),
        proposed_script_patch: None,
        source_dependency_id: Some(binding.cause.dependency_id.clone()),
        source_event_id: binding.cause.current_revision_event_id,
        rationale: Some(
            "Reviewable screenplay update; inferred world assertions require separate proposals."
                .into(),
        ),
        created_at_ms,
        script_review_binding: Some(binding.clone()),
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        proposal.summary.clone(),
    )
    .with_created_at_ms(created_at_ms);
    let revision = propagation_proposal_store::propagation_proposal_revision(&proposal, event.id)?;
    Ok(history_store::record_change_with(
        conn,
        command,
        REQUEST_TYPE,
        &event,
        &[revision],
        |tx| {
            validate_binding(tx, &binding)?;
            propagation_proposal_store::insert_proposal_in_transaction(tx, &proposal, event.id)
        },
    )?)
}

pub(crate) fn accept_bound_proposal(
    conn: &mut Connection,
    command: &CommandEnvelope<AcceptPropagationProposalCommand>,
    proposal: &PropagationProposal,
    created_at_ms: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    let binding = proposal.script_review_binding.as_ref().ok_or_else(|| {
        PropagationProposalStoreError::InvalidCommand("missing screenplay review binding".into())
    })?;
    validate_binding(conn, binding)?;
    let (document, segment, block, _) = target(conn, &binding.request)?;
    if proposal.action != PropagationProposalAction::PatchScriptBlock
        || proposal.target
            != (PropagationProposalTarget::ScriptBlock {
                block_id: binding.request.block_id.clone(),
            })
    {
        return Err(stale().into());
    }
    let text = proposal
        .proposed_text
        .as_ref()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| {
            PropagationProposalStoreError::InvalidCommand("missing screenplay preview text".into())
        })?;
    let write = SetScriptBlockCommand {
        document_id: document.document.id.clone(),
        document_title: document.document.title.clone(),
        document_sort_order: document.document.sort_order,
        segment_id: segment.segment.id.clone(),
        source_node_id: segment.segment.source_node_id.clone(),
        segment_start_ms: segment.segment.start_ms,
        segment_end_ms: segment.segment.end_ms,
        segment_status: segment.segment.status.clone(),
        segment_sort_order: segment.segment.sort_order,
        block_id: block.block.id.clone(),
        block_kind: block.block.block_kind.clone(),
        text: text.clone(),
        span_provenance: ScriptSpanProvenance::AiGenerated,
        sort_order: block.block.sort_order,
    };
    script_document_command::validate_locked_spans(Some(&document), &write)?;
    let generated = GenerateScriptBlockCommand {
        target_binding: None,
        block: write.clone(),
        script_inputs: Some(binding.script_inputs.clone()),
        bible_relationship_inputs: binding.bible_relationship_inputs.clone(),
        bible_inputs: Some(binding.bible_inputs.clone()),
        bible_context_scope: binding.bible_context_scope.clone(),
        script_context_scope: binding.script_context_scope.clone(),
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalAccepted,
        "accept screenplay impact update",
    )
    .with_created_at_ms(created_at_ms);
    let new_block = script_document_command::command_block(&write);
    let span = script_document_command::generated_span_for_block(
        &new_block,
        ScriptSpanProvenance::AiGenerated,
    )?;
    let dependencies =
        script_generation_lineage::dependencies(&generated, event.id, created_at_ms)?;
    let mut revisions = vec![
        propagation_proposal_review::proposal_status_revision(
            proposal,
            event.id,
            SemanticProposalStatus::Accepted,
            None,
        )?,
        script_document_command::block_revision(
            &new_block,
            Some(FieldValue::Text(block.block.text)),
            event.id,
        ),
        script_document_command::span_revision(&span, event.id),
    ];
    for dependency in &dependencies {
        revisions.push(crate::semantic_dependency_store::dependency_revision(
            dependency, event.id,
        )?);
    }
    Ok(history_store::record_change_with(
        conn,
        command,
        "semantic.propagation_accept",
        &event,
        &revisions,
        |tx| {
            validate_binding(tx, binding)?;
            let current = propagation_proposal_review::load_pending_proposal(tx, &proposal.id)
                .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
            if current != *proposal {
                return Err(stale());
            }
            let current_document =
                script_store::load_document_projection(tx, &binding.request.document_id)?
                    .ok_or_else(stale)?;
            script_document_command::validate_locked_spans(Some(&current_document), &write)
                .map_err(|error| HistoryStoreError::InvalidValue(error.to_string()))?;
            propagation_proposal_review::update_proposal_status_in_transaction(
                tx,
                &proposal.id,
                SemanticProposalStatus::Accepted,
            )?;
            script_store::upsert_block_in_transaction(tx, &new_block, event.id)?;
            script_store::upsert_span_in_transaction(tx, &span, event.id)?;
            script_generation_lineage::record_in_transaction(
                tx,
                &generated,
                event.id,
                &dependencies,
            )
        },
    )?)
}

#[cfg(test)]
#[path = "script_impact_review_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "script_impact_review_guard_tests.rs"]
mod guard_tests;
