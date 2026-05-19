use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, BibleGraphNodeId, BibleGraphSchemaKey, BibleReferenceKind,
    CommandEnvelope, CreateBibleGraphNodeCommand, CreateBibleReferenceProposalCommand,
    EnsureCanonicalBibleRootsCommand, ObjectKind, RejectBibleReferenceProposalCommand,
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
fn rejects_pending_bible_reference_proposal() {
    let mut conn = Connection::open_in_memory().unwrap();
    let create = create_command(
        "proposal.child.ring",
        "Signal ring",
        BibleReferenceKind::Prop,
    );
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let reject = reject_command("proposal.child.ring", Some("Not relevant"));

    let outcome = record_reject_bible_reference_proposal(&mut conn, &reject, 43).unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.child.ring",
    )
    .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Rejected);
    assert_eq!(revisions.len(), 2);
    assert!(revisions[1].fields.iter().any(|field| {
        field.field_key == "rejection_reason"
            && field.new_value
                == Some(eidetic_core::contracts::FieldValue::Text(
                    "Not relevant".to_string(),
                ))
    }));
}

#[test]
fn reject_command_replays_without_second_update() {
    let mut conn = Connection::open_in_memory().unwrap();
    let create = create_command(
        "proposal.child.ring",
        "Signal ring",
        BibleReferenceKind::Prop,
    );
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let reject = reject_command("proposal.child.ring", None);

    let first = record_reject_bible_reference_proposal(&mut conn, &reject, 43).unwrap();
    let second = record_reject_bible_reference_proposal(&mut conn, &reject, 43).unwrap();
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.child.ring",
    )
    .unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(revisions.len(), 2);
}

#[test]
fn reject_command_requires_existing_pending_proposal() {
    let mut conn = Connection::open_in_memory().unwrap();
    let reject = reject_command("proposal.child.missing", None);

    let error = record_reject_bible_reference_proposal(&mut conn, &reject, 43).unwrap_err();

    assert!(matches!(error, SemanticProposalStoreError::NotFound(_)));
}

#[test]
fn accepts_pending_bible_reference_proposal_by_creating_bible_node() {
    let mut conn = Connection::open_in_memory().unwrap();
    ensure_roots(&mut conn);
    let create = create_command(
        "proposal.child.harbor",
        "Storm Harbor",
        BibleReferenceKind::Location,
    );
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let accept = accept_command("proposal.child.harbor", "node.location.storm-harbor", None);

    let outcome = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 43,
    )
    .unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();
    let node = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("node.location.storm-harbor").unwrap(),
    )
    .unwrap()
    .expect("accepted bible node");
    let proposal_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.child.harbor",
    )
    .unwrap();
    let node_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::BibleNode,
        "node.location.storm-harbor",
    )
    .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    assert_eq!(node.node.name, "Storm Harbor");
    assert_eq!(node.node.parent_id.unwrap().as_str(), "canonical.places");
    assert_eq!(node.node.schema_key.as_str(), "location");
    assert_eq!(proposal_revisions.len(), 2);
    assert_eq!(node_revisions.len(), 1);
}

#[test]
fn accept_command_replays_without_second_node_insert() {
    let mut conn = Connection::open_in_memory().unwrap();
    ensure_roots(&mut conn);
    let create = create_command("proposal.child.ada", "Ada", BibleReferenceKind::Character);
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let accept = accept_command(
        "proposal.child.ada",
        "node.character.ada",
        Some("Ada Prime"),
    );

    let first = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 43,
    )
    .unwrap();
    let second = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 43,
    )
    .unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();
    let nodes = crate::bible_graph_store::load_node_list_projection(&conn).unwrap();

    assert_eq!(first, RecordChangeOutcome::Recorded);
    assert_eq!(second, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    assert_eq!(
        nodes
            .nodes
            .iter()
            .filter(|node| node.id.as_str() == "node.character.ada")
            .count(),
        1
    );
}

#[test]
fn accept_command_can_link_existing_bible_node() {
    let mut conn = Connection::open_in_memory().unwrap();
    let existing_node = CommandEnvelope::new(CreateBibleGraphNodeCommand {
        node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: "Ada".to_string(),
        sort_order: 10,
    });
    crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &existing_node, 40)
        .unwrap();
    let create = create_command("proposal.child.ada", "Ada", BibleReferenceKind::Character);
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let accept = accept_command("proposal.child.ada", "node.character.ada", None);

    let outcome = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 43,
    )
    .unwrap();
    let proposals = load_bible_reference_proposals(&conn).unwrap();
    let nodes = crate::bible_graph_store::load_node_list_projection(&conn).unwrap();
    let node_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::BibleNode,
        "node.character.ada",
    )
    .unwrap();

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    assert_eq!(
        nodes
            .nodes
            .iter()
            .filter(|node| node.id.as_str() == "node.character.ada")
            .count(),
        1
    );
    assert_eq!(
        node_revisions.len(),
        1,
        "linking should not mutate the existing bible node"
    );
}

#[test]
fn accept_command_requires_existing_canonical_parent() {
    let mut conn = Connection::open_in_memory().unwrap();
    let create = create_command("proposal.child.ada", "Ada", BibleReferenceKind::Character);
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let accept = accept_command("proposal.child.ada", "node.character.ada", None);

    let error = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 43,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        SemanticProposalStoreError::InvalidCommand(message)
            if message.contains("parent bible graph node does not exist")
    ));
}

#[test]
fn accept_command_requires_pending_proposal() {
    let mut conn = Connection::open_in_memory().unwrap();
    ensure_roots(&mut conn);
    let create = create_command(
        "proposal.child.ring",
        "Signal ring",
        BibleReferenceKind::Prop,
    );
    record_create_bible_reference_proposal(&mut conn, &create, 42).unwrap();
    let reject = reject_command("proposal.child.ring", None);
    record_reject_bible_reference_proposal(&mut conn, &reject, 43).unwrap();
    let accept = accept_command("proposal.child.ring", "node.prop.signal-ring", None);

    let error = crate::semantic_proposal_accept::record_accept_bible_reference_proposal(
        &mut conn, &accept, 44,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        SemanticProposalStoreError::InvalidCommand(message)
            if message.contains("semantic proposal is not pending")
    ));
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

fn reject_command(
    proposal_id: &str,
    reason: Option<&str>,
) -> CommandEnvelope<RejectBibleReferenceProposalCommand> {
    CommandEnvelope::new(RejectBibleReferenceProposalCommand {
        proposal_id: SemanticProposalId::new(proposal_id).unwrap(),
        reason: reason.map(str::to_string),
    })
}

fn accept_command(
    proposal_id: &str,
    node_id: &str,
    name: Option<&str>,
) -> CommandEnvelope<AcceptBibleReferenceProposalCommand> {
    CommandEnvelope::new(AcceptBibleReferenceProposalCommand {
        proposal_id: SemanticProposalId::new(proposal_id).unwrap(),
        node_id: BibleGraphNodeId::new(node_id).unwrap(),
        parent_id: None,
        name: name.map(str::to_string),
        sort_order: 9,
    })
}

fn ensure_roots(conn: &mut Connection) {
    crate::bible_graph_command::apply_ensure_canonical_bible_roots(
        conn,
        &CommandEnvelope::new(EnsureCanonicalBibleRootsCommand {}),
        1,
    )
    .unwrap();
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
