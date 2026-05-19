use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, CommandEnvelope, CreateBibleGraphNodeCommand,
    CreatePropagationProposalCommand, FieldValue, ObjectKind, PropagationProposalAction,
    PropagationProposalId, PropagationProposalTarget, RejectPropagationProposalCommand,
    ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptLockId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanId, ScriptSpanProvenance, SemanticDependencyId,
    SemanticProposalStatus, SetScriptBlockCommand, SetScriptLockCommand,
};

use super::{
    PropagationProposalStoreError, load_propagation_proposal_list_projection,
    load_propagation_proposals, record_create_propagation_proposal,
};
use crate::history_store::{self, RecordChangeOutcome};
use crate::propagation_proposal_review::{
    record_accept_propagation_proposal, record_reject_propagation_proposal,
};

#[test]
fn records_and_projects_propagation_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let command = create_field_proposal_command("proposal.propagation.weather");

    let outcome = record_create_propagation_proposal(&mut conn, &command, 100).unwrap();
    let projection = load_propagation_proposal_list_projection(&conn).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(projection.version.0, 2);
    assert_eq!(projection.payload.proposals.len(), 1);
    assert_eq!(
        projection.payload.proposals[0].id.as_str(),
        "proposal.propagation.weather"
    );
    assert_eq!(
        projection.payload.proposals[0].status,
        SemanticProposalStatus::Pending
    );
    assert_eq!(
        projection.payload.proposals[0].proposed_value,
        Some(FieldValue::Text("rainy".to_string()))
    );
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.propagation.weather",
    )
    .unwrap();
    assert_eq!(revisions.len(), 1);
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "proposal_kind")
    );
}

#[test]
fn replays_duplicate_create_command() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let command = create_field_proposal_command("proposal.propagation.replay");

    let first = record_create_propagation_proposal(&mut conn, &command, 100).unwrap();
    let second = record_create_propagation_proposal(&mut conn, &command, 100).unwrap();
    let proposals = load_propagation_proposals(&conn).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(proposals.len(), 1);
}

#[test]
fn rejects_mismatched_action_target() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut command = create_field_proposal_command("proposal.propagation.invalid");
    command.payload.action = PropagationProposalAction::PatchScriptBlock;

    let error = record_create_propagation_proposal(&mut conn, &command, 100).unwrap_err();

    assert!(
        matches!(error, PropagationProposalStoreError::InvalidCommand(message) if message.contains("target kind"))
    );
}

#[test]
fn rejects_missing_bible_value() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let mut command = create_field_proposal_command("proposal.propagation.missing-value");
    command.payload.proposed_value = None;

    let error = record_create_propagation_proposal(&mut conn, &command, 100).unwrap_err();

    assert!(
        matches!(error, PropagationProposalStoreError::InvalidCommand(message) if message.contains("proposed_value"))
    );
}

#[test]
fn rejects_pending_propagation_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let create = create_field_proposal_command("proposal.propagation.reject");
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let reject = CommandEnvelope::new(RejectPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.reject").unwrap(),
        reason: Some("Wrong scope".to_string()),
    });

    let outcome = record_reject_propagation_proposal(&mut conn, &reject, 101).unwrap();
    let proposals = load_propagation_proposals(&conn).unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Rejected);
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.propagation.reject",
    )
    .unwrap();
    assert_eq!(revisions.len(), 2);
    assert!(
        revisions[1]
            .fields
            .iter()
            .any(|field| field.field_key == "rejection_reason")
    );
}

#[test]
fn reject_replays_without_second_update() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let create = create_field_proposal_command("proposal.propagation.reject-replay");
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let reject = CommandEnvelope::new(RejectPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.reject-replay").unwrap(),
        reason: None,
    });

    let first = record_reject_propagation_proposal(&mut conn, &reject, 101).unwrap();
    let second = record_reject_propagation_proposal(&mut conn, &reject, 101).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
}

#[test]
fn reject_requires_existing_pending_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let reject = CommandEnvelope::new(RejectPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.missing").unwrap(),
        reason: None,
    });

    let error = record_reject_propagation_proposal(&mut conn, &reject, 101).unwrap_err();

    assert!(matches!(error, PropagationProposalStoreError::NotFound(_)));
}

#[test]
fn accepts_pending_bible_field_propagation_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_location_node(&mut conn);
    let create = create_field_proposal_command("proposal.propagation.accept");
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.accept").unwrap(),
    });

    let outcome = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap();
    let proposals = load_propagation_proposals(&conn).unwrap();
    let detail = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("node.location.harbor").unwrap(),
    )
    .unwrap()
    .expect("node detail");

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    let field = detail
        .parts
        .iter()
        .flat_map(|part| part.fields.iter())
        .find(|field| field.field_key.as_str() == "weather")
        .expect("weather field");
    assert_eq!(field.value, Some(FieldValue::Text("rainy".to_string())));
    let proposal_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.propagation.accept",
    )
    .unwrap();
    let field_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::BiblePartField,
        field.id.as_str(),
    )
    .unwrap();
    assert_eq!(proposal_revisions.len(), 2);
    assert_eq!(field_revisions.len(), 1);
}

#[test]
fn accept_replays_without_second_field_update() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_location_node(&mut conn);
    let create = create_field_proposal_command("proposal.propagation.accept-replay");
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.accept-replay").unwrap(),
    });

    let first = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap();
    let second = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
}

#[test]
fn accepts_pending_script_block_propagation_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_script_block(&mut conn, "Ada enters with a wet umbrella.");
    let create = create_script_block_proposal_command(
        "proposal.propagation.script-block",
        "Ada enters with a rain-black umbrella.",
    );
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.script-block").unwrap(),
    });

    let outcome = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap();
    let proposals = load_propagation_proposals(&conn).unwrap();
    let projection = crate::script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .expect("script projection");

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    assert_eq!(
        projection.segments[0].blocks[0].block.text,
        "Ada enters with a rain-black umbrella."
    );
    assert_eq!(
        projection.segments[0].blocks[0].spans[0].provenance,
        ScriptSpanProvenance::AiGenerated
    );
    let proposal_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.propagation.script-block",
    )
    .unwrap();
    let block_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptBlock,
        "script.block.action-1",
    )
    .unwrap();
    assert_eq!(proposal_revisions.len(), 2);
    assert_eq!(block_revisions.len(), 2);
}

#[test]
fn accept_rejects_script_block_proposal_that_modifies_locked_text() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_script_block(&mut conn, "Ada enters with a wet umbrella.");
    seed_script_lock(&mut conn);
    let create = create_script_block_proposal_command(
        "proposal.propagation.locked-script-block",
        "Ada enters with a dry umbrella.",
    );
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.locked-script-block")
            .unwrap(),
    });

    let error = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap_err();
    let projection = crate::script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .expect("script projection");

    assert!(
        matches!(error, PropagationProposalStoreError::ScriptDocumentCommand(error) if error.to_string().contains("locked span"))
    );
    assert_eq!(
        projection.segments[0].blocks[0].block.text,
        "Ada enters with a wet umbrella."
    );
}

#[test]
fn accept_rejects_unsupported_propagation_action() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    let create = CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.script").unwrap(),
        action: PropagationProposalAction::RegenerateScriptSegment,
        target: PropagationProposalTarget::ScriptSegment {
            segment_id: eidetic_core::contracts::ScriptSegmentId::new("script.segment.scene-1")
                .unwrap(),
        },
        summary: "Regenerate scene".to_string(),
        proposed_value: None,
        proposed_text: None,
        source_dependency_id: None,
        source_event_id: None,
        rationale: None,
    });
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.script").unwrap(),
    });

    let error = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap_err();

    assert!(
        matches!(error, PropagationProposalStoreError::InvalidCommand(message) if message.contains("cannot be accepted yet"))
    );
}

fn create_field_proposal_command(
    proposal_id: &str,
) -> CommandEnvelope<CreatePropagationProposalCommand> {
    CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new(proposal_id).unwrap(),
        action: PropagationProposalAction::SetBibleField,
        target: PropagationProposalTarget::BibleField {
            node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
            part_key: BibleGraphPartKey::new("environment").unwrap(),
            field_key: BibleGraphFieldKey::new("weather").unwrap(),
            field_id: None,
        },
        summary: "Set harbor weather to rainy".to_string(),
        proposed_value: Some(FieldValue::Text("rainy".to_string())),
        proposed_text: None,
        source_dependency_id: Some(SemanticDependencyId::new("dependency.weather.scene").unwrap()),
        source_event_id: None,
        rationale: Some("Manual script edit changed weather.".to_string()),
    })
}

fn create_script_block_proposal_command(
    proposal_id: &str,
    proposed_text: &str,
) -> CommandEnvelope<CreatePropagationProposalCommand> {
    CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new(proposal_id).unwrap(),
        action: PropagationProposalAction::PatchScriptBlock,
        target: PropagationProposalTarget::ScriptBlock {
            block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
        },
        summary: "Patch generated script block".to_string(),
        proposed_value: None,
        proposed_text: Some(proposed_text.to_string()),
        source_dependency_id: Some(SemanticDependencyId::new("dependency.weather.scene").unwrap()),
        source_event_id: None,
        rationale: Some("Manual edit requires script wording propagation.".to_string()),
    })
}

fn seed_location_node(conn: &mut rusqlite::Connection) {
    crate::bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("location").unwrap(),
            name: "Storm Harbor".to_string(),
            sort_order: 1,
        }),
        1,
    )
    .unwrap();
}

fn seed_script_block(conn: &mut rusqlite::Connection, text: &str) {
    crate::script_document_command::apply_set_script_block(
        conn,
        &CommandEnvelope::new(SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            document_title: "Pilot".to_string(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
            source_node_id: Some("node.beat.opening".to_string()),
            segment_start_ms: 1_000,
            segment_end_ms: 5_000,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: 1,
            block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
            block_kind: ScriptBlockKind::Action,
            text: text.to_string(),
            span_provenance: ScriptSpanProvenance::AiGenerated,
            sort_order: 2,
        }),
        1,
    )
    .unwrap();
}

fn seed_script_lock(conn: &mut rusqlite::Connection) {
    crate::script_document_command::apply_set_script_lock(
        conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("script.lock.action-1").unwrap(),
            span_id: ScriptSpanId::new("script.block.action-1.span.main").unwrap(),
            reason: "User approved wording.".to_string(),
        }),
        2,
    )
    .unwrap();
}
