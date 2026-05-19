use eidetic_core::contracts::{
    AcceptPropagationProposalCommand, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, BibleGraphSnapshotFieldId, BibleGraphSnapshotId, CommandEnvelope,
    CreateBibleGraphNodeCommand, CreatePropagationProposalCommand, FieldValue, ObjectKind,
    PropagationProposalAction, PropagationProposalId, PropagationProposalTarget, ScriptBlock,
    ScriptBlockId, ScriptBlockKind, ScriptBlockProjection, ScriptDocumentId, ScriptLockId,
    ScriptPatch, ScriptPatchId, ScriptSegment, ScriptSegmentId, ScriptSegmentProjection,
    ScriptSegmentStatus, ScriptSpan, ScriptSpanId, ScriptSpanProvenance, SemanticDependencyId,
    SemanticProposalStatus, SetBibleGraphSnapshotFieldCommand, SetScriptBlockCommand,
    SetScriptLockCommand,
};

use super::record_accept_propagation_proposal;
use crate::history_store::{self, RecordChangeOutcome};
use crate::propagation_proposal_store::{
    PropagationProposalStoreError, load_propagation_proposals, record_create_propagation_proposal,
};

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
fn accepts_pending_bible_snapshot_field_propagation_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_character_snapshot_field(&mut conn, "Rain-soaked");
    let create = create_snapshot_field_proposal_command(
        "proposal.propagation.snapshot",
        FieldValue::Text("Dry and wary".to_string()),
    );
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.snapshot").unwrap(),
    });

    let outcome = record_accept_propagation_proposal(&mut conn, &accept, 101).unwrap();
    let proposals = load_propagation_proposals(&conn).unwrap();
    let detail = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("node.character.ada").unwrap(),
    )
    .unwrap()
    .expect("node detail");

    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(proposals[0].status, SemanticProposalStatus::Accepted);
    assert_eq!(
        detail.snapshots[0].fields[0].value,
        Some(FieldValue::Text("Dry and wary".to_string()))
    );
    let proposal_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::SemanticProposal,
        "proposal.propagation.snapshot",
    )
    .unwrap();
    let snapshot_revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::BibleSnapshot,
        "snapshot.character.ada.sequence-1",
    )
    .unwrap();
    assert_eq!(proposal_revisions.len(), 2);
    assert_eq!(snapshot_revisions.len(), 2);
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
fn accepts_pending_script_segment_regeneration_proposal() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    seed_script_block(&mut conn, "Ada enters with a wet umbrella.");
    seed_second_script_block(&mut conn);
    let create = CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.script").unwrap(),
        action: PropagationProposalAction::RegenerateScriptSegment,
        target: PropagationProposalTarget::ScriptSegment {
            segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        },
        summary: "Regenerate scene".to_string(),
        proposed_value: None,
        proposed_text: None,
        proposed_script_patch: Some(regenerated_segment_patch()),
        source_dependency_id: None,
        source_event_id: None,
        rationale: None,
    });
    record_create_propagation_proposal(&mut conn, &create, 100).unwrap();
    let accept = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: PropagationProposalId::new("proposal.propagation.script").unwrap(),
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
    assert_eq!(projection.segments[0].blocks.len(), 1);
    assert_eq!(
        projection.segments[0].blocks[0].block.id.as_str(),
        "script.block.regenerated-action"
    );
    assert_eq!(
        projection.segments[0].blocks[0].block.text,
        "Ada steps into rain-black light."
    );
    assert_eq!(
        projection.segments[0].blocks[0].spans[0].provenance,
        ScriptSpanProvenance::System
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
        proposed_script_patch: None,
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
        proposed_script_patch: None,
        source_dependency_id: Some(SemanticDependencyId::new("dependency.weather.scene").unwrap()),
        source_event_id: None,
        rationale: Some("Manual edit requires script wording propagation.".to_string()),
    })
}

fn create_snapshot_field_proposal_command(
    proposal_id: &str,
    proposed_value: FieldValue,
) -> CommandEnvelope<CreatePropagationProposalCommand> {
    CommandEnvelope::new(CreatePropagationProposalCommand {
        proposal_id: PropagationProposalId::new(proposal_id).unwrap(),
        action: PropagationProposalAction::SetBibleSnapshotField,
        target: PropagationProposalTarget::BibleSnapshotField {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            snapshot_id: BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            field_id: BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
        },
        summary: "Set character snapshot status".to_string(),
        proposed_value: Some(proposed_value),
        proposed_text: None,
        proposed_script_patch: None,
        source_dependency_id: Some(SemanticDependencyId::new("dependency.weather.scene").unwrap()),
        source_event_id: None,
        rationale: Some("Manual edit changed the character state.".to_string()),
    })
}

fn regenerated_segment_patch() -> ScriptPatch {
    ScriptPatch {
        id: ScriptPatchId::new("script.patch.regenerate-beat-1").unwrap(),
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        segments: vec![ScriptSegmentProjection {
            segment: ScriptSegment {
                id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                document_id: ScriptDocumentId::new("script.document.main").unwrap(),
                source_node_id: Some("node.beat.opening".to_string()),
                start_ms: 1_000,
                end_ms: 5_000,
                status: ScriptSegmentStatus::Current,
                sort_order: 1,
            },
            blocks: vec![ScriptBlockProjection {
                block: ScriptBlock {
                    id: ScriptBlockId::new("script.block.regenerated-action").unwrap(),
                    segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                    block_kind: ScriptBlockKind::Action,
                    text: "Ada steps into rain-black light.".to_string(),
                    sort_order: 1,
                },
                spans: vec![ScriptSpan {
                    id: ScriptSpanId::new("script.span.regenerated-action").unwrap(),
                    block_id: ScriptBlockId::new("script.block.regenerated-action").unwrap(),
                    start_byte: 0,
                    end_byte: 32,
                    provenance: ScriptSpanProvenance::System,
                }],
                locks: Vec::new(),
            }],
        }],
    }
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

fn seed_character_snapshot_field(conn: &mut rusqlite::Connection, value: &str) {
    crate::bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".to_string(),
            sort_order: 1,
        }),
        1,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_snapshot_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            at_ms: 12_000,
            label: "Sequence 1 state".to_string(),
            snapshot_sort_order: 1,
            field_id: BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".to_string(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text(value.to_string())),
            field_sort_order: 2,
        }),
        2,
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

fn seed_second_script_block(conn: &mut rusqlite::Connection) {
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
            block_id: ScriptBlockId::new("script.block.dialogue-1").unwrap(),
            block_kind: ScriptBlockKind::Dialogue,
            text: "It followed me home.".to_string(),
            span_provenance: ScriptSpanProvenance::AiGenerated,
            sort_order: 3,
        }),
        2,
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
