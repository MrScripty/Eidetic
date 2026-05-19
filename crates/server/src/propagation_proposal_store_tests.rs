use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, CommandEnvelope,
    CreatePropagationProposalCommand, FieldValue, ObjectKind, PropagationProposalAction,
    PropagationProposalId, PropagationProposalTarget, RejectPropagationProposalCommand,
    SemanticDependencyId, SemanticProposalStatus,
};

use super::{
    PropagationProposalStoreError, load_propagation_proposal_list_projection,
    load_propagation_proposals, record_create_propagation_proposal,
};
use crate::history_store::{self, RecordChangeOutcome};
use crate::propagation_proposal_review::record_reject_propagation_proposal;

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
