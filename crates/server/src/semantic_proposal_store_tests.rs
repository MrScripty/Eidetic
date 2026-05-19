use eidetic_core::contracts::{
    BibleReferenceKind, CommandEnvelope, CreateBibleReferenceProposalCommand, ObjectKind,
    SemanticProposalId, SemanticProposalStatus,
};
use eidetic_core::timeline::node::NodeId;
use rusqlite::Connection;

use crate::history_store::{self, RecordChangeOutcome};

use super::*;

#[test]
fn records_and_loads_bible_reference_proposal() {
    let mut conn = Connection::open_in_memory().unwrap();
    let command = create_command("proposal.child.ada", "Ada", BibleReferenceKind::Character);

    let outcome = record_create_bible_reference_proposal(&mut conn, &command, 42).unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.child.ada",
    )
    .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].id.as_str(), "proposal.child.ada");
    assert_eq!(proposals[0].reference_text, "Ada");
    assert_eq!(proposals[0].status, SemanticProposalStatus::Pending);
    assert_eq!(proposals[0].created_at_ms, 42);
    assert_eq!(revisions.len(), 1);
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "rationale")
    );
}

#[test]
fn duplicate_command_replay_does_not_insert_again() {
    let mut conn = Connection::open_in_memory().unwrap();
    let command = create_command(
        "proposal.child.harbor",
        "Storm Harbor",
        BibleReferenceKind::Location,
    );

    let first = record_create_bible_reference_proposal(&mut conn, &command, 42).unwrap();
    let second = record_create_bible_reference_proposal(&mut conn, &command, 42).unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(proposals.len(), 1);
}

#[test]
fn different_command_cannot_reuse_existing_proposal_id() {
    let mut conn = Connection::open_in_memory().unwrap();
    let first = create_command(
        "proposal.child.ring",
        "Signal ring",
        BibleReferenceKind::Prop,
    );
    let second = create_command(
        "proposal.child.ring",
        "Other ring",
        BibleReferenceKind::Prop,
    );

    record_create_bible_reference_proposal(&mut conn, &first, 42).unwrap();
    let error = record_create_bible_reference_proposal(&mut conn, &second, 43).unwrap_err();

    assert!(matches!(
        error,
        SemanticProposalStoreError::InvalidCommand(message)
            if message.contains("semantic proposal already exists")
    ));
}

fn create_command(
    proposal_id: &str,
    reference_text: &str,
    reference_kind: BibleReferenceKind,
) -> CommandEnvelope<CreateBibleReferenceProposalCommand> {
    CommandEnvelope::new(CreateBibleReferenceProposalCommand {
        proposal_id: SemanticProposalId::new(proposal_id).unwrap(),
        source_node_id: NodeId::new(),
        child_name: "Opening Beat".to_string(),
        reference_kind,
        reference_text: reference_text.to_string(),
        rationale: Some("Detected in child plan".to_string()),
    })
}
