use super::*;
use crate::{propagation_proposal_accept, propagation_proposal_update, script_block_edit};
use eidetic_core::{Project, Template};

pub(crate) fn fixture() -> (
    Connection,
    Project,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
    let project = Template::MultiCam.build_project("Linked scenes");
    let tx = conn.transaction().unwrap();
    crate::timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    tx.commit().unwrap();
    let mut blocks = Vec::new();
    for (index, name) in ["A", "B", "C"].iter().enumerate() {
        let block = SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            document_title: "Linked scenes".into(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new(format!("segment.{name}")).unwrap(),
            source_node_id: Some(project.timeline.nodes[index].id.0.to_string()),
            segment_start_ms: index as u64 * 1000,
            segment_end_ms: (index as u64 + 1) * 1000,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: index as u32,
            block_id: ScriptBlockId::new(format!("block.{name}")).unwrap(),
            block_kind: ScriptBlockKind::Action,
            text: format!("Original {name}"),
            span_provenance: ScriptSpanProvenance::UserEdited,
            sort_order: 0,
        };
        script_document_command::apply_set_script_block(
            &mut conn,
            &CommandEnvelope::new(block.clone()),
            10,
        )
        .unwrap();
        blocks.push(block);
    }
    let [a, b, c]: [SetScriptBlockCommand; 3] = blocks.try_into().unwrap();
    let a_input = inputs(&conn, &a)
        .into_iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    script_document_command::apply_generated_script_block(
        &mut conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            arc_description_applicability: None,
            timeline_title_inputs: None,
            ancestor_notes_inputs: None,
            arc_inputs: None,
            target_binding: None,
            script_context_scope: None,
            bible_node_name_inputs: None,
            bible_relationship_inputs: None,
            bible_inputs: None,
            bible_context_scope: None,
            block: b.clone(),
            script_inputs: Some(vec![a_input]),
        }),
        15,
    )
    .unwrap();
    edit(&mut conn, &a, "  A now carries a blue umbrella — 雨\n\n");
    (conn, project, a, b, c)
}

fn inputs(conn: &Connection, block: &SetScriptBlockCommand) -> Vec<ScriptContextBlock> {
    ai_script_context::load_script_context(
        conn,
        NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap()),
        block.segment_start_ms,
        block.segment_end_ms,
    )
    .unwrap()
}

pub(crate) fn edit(conn: &mut Connection, block: &SetScriptBlockCommand, text: &str) {
    let revision = inputs(conn, block)
        .into_iter()
        .find(|input| input.block_id == block.block_id)
        .unwrap()
        .revision_event_id;
    script_block_edit::apply_edit_script_block(
        conn,
        &CommandEnvelope::new(EditScriptBlockCommand {
            block_kind: None,
            document_id: block.document_id.clone(),
            block_id: block.block_id.clone(),
            expected_revision_event_id: revision,
            text: text.into(),
        }),
        20,
    )
    .unwrap();
}

pub(crate) fn request(
    conn: &Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let document = script_store::load_document_projection(conn, &b.document_id)
        .unwrap()
        .unwrap();
    let segment = document
        .segments
        .iter()
        .find(|segment| segment.segment.id == b.segment_id)
        .unwrap();
    let impact = segment.impact.as_ref().unwrap();
    CommandEnvelope::new(RequestScriptImpactProposalCommand {
        proposal_id: PropagationProposalId::new(format!("review.{}", uuid::Uuid::new_v4()))
            .unwrap(),
        document_id: b.document_id.clone(),
        segment_id: b.segment_id.clone(),
        block_id: b.block_id.clone(),
        expected_block_revision_event_id: segment.blocks[0].revision_event_id.unwrap(),
        generation_event_id: impact.generation_event_id,
        dependency_id: impact.causes[0].dependency_id.clone(),
        story_time_ms: None,
        recall_selection: None,
    })
}

pub(crate) fn propose(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let command = request(conn, b);
    let binding = capture(conn, &command.payload).unwrap();
    assert_eq!(binding.bible_context.payload.story_time_ms, None);
    assert!(
        binding
            .script_inputs
            .iter()
            .any(|input| input.text == "  A now carries a blue umbrella — 雨\n\n")
    );
    record_proposal(
        conn,
        &command,
        binding,
        "  B notices the blue umbrella.\n\n".into(),
        30,
    )
    .unwrap();
    command
}

pub(crate) fn text(conn: &Connection, block: &SetScriptBlockCommand) -> String {
    inputs(conn, block)
        .into_iter()
        .find(|input| input.block_id == block.block_id)
        .unwrap()
        .text
}

pub(crate) fn accept(
    conn: &mut Connection,
    command: &CommandEnvelope<RequestScriptImpactProposalCommand>,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_accept::record_accept_propagation_proposal(
        conn,
        &CommandEnvelope::new(AcceptPropagationProposalCommand {
            proposal_id: command.payload.proposal_id.clone(),
        }),
        40,
    )
}

#[test]
fn linked_scene_preview_and_rejection_preserve_authored_canon_and_review_cause() {
    let (mut conn, _, a, b, c) = fixture();
    let before = script_store::load_document_projection(&conn, &b.document_id).unwrap();
    let command = propose(&mut conn, &b);
    assert_eq!(
        script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        before
    );
    propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: command.payload.proposal_id.clone(),
            reason: Some("Keep my version".into()),
        }),
        40,
    )
    .unwrap();
    assert_eq!(
        script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        before
    );
    assert_eq!(text(&conn, &a), "  A now carries a blue umbrella — 雨\n\n");
    assert_eq!(text(&conn, &b), "Original B");
    assert_eq!(text(&conn, &c), "Original C");
    let proposal =
        propagation_proposal_store::load_propagation_proposal(&conn, &command.payload.proposal_id)
            .unwrap()
            .unwrap();
    assert_eq!(proposal.status, SemanticProposalStatus::Rejected);
    assert!(accept(&mut conn, &command).is_err());
}

#[test]
fn explicit_acceptance_updates_only_target_and_refreshes_actual_input_bindings_atomically() {
    let (mut conn, _, a, b, c) = fixture();
    let command = propose(&mut conn, &b);
    let before = inputs(&conn, &b)
        .into_iter()
        .find(|input| input.block_id == b.block_id)
        .unwrap();
    let accept_command = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: command.payload.proposal_id.clone(),
    });
    assert_eq!(
        propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &accept_command,
            40
        )
        .unwrap(),
        RecordChangeOutcome::Recorded
    );
    assert_eq!(text(&conn, &b), "  B notices the blue umbrella.\n\n");
    assert_eq!(text(&conn, &a), "  A now carries a blue umbrella — 雨\n\n");
    assert_eq!(text(&conn, &c), "Original C");
    let after = inputs(&conn, &b)
        .into_iter()
        .find(|input| input.block_id == b.block_id)
        .unwrap();
    assert_ne!(before.revision_event_id, after.revision_event_id);
    assert_eq!(
        before.segment_revision_event_id,
        after.segment_revision_event_id
    );
    assert!(
        !crate::script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
    assert_eq!(
        propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &accept_command,
            50
        )
        .unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    edit(&mut conn, &a, "Later source A");
    assert!(
        crate::script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
}

#[test]
fn edits_and_aba_after_preview_refuse_acceptance_without_discarding_manual_text() {
    for source in [true, false] {
        let (mut conn, _, a, b, _) = fixture();
        let command = propose(&mut conn, &b);
        let edited = if source { &a } else { &b };
        let original = text(&conn, edited);
        edit(&mut conn, edited, "Manual intervening text");
        edit(&mut conn, edited, &original);
        let before = script_store::load_document_projection(&conn, &b.document_id).unwrap();
        assert!(
            accept(&mut conn, &command)
                .unwrap_err()
                .to_string()
                .contains("stale")
        );
        assert_eq!(
            script_store::load_document_projection(&conn, &b.document_id).unwrap(),
            before
        );
        assert_eq!(
            propagation_proposal_store::load_propagation_proposal(
                &conn,
                &command.payload.proposal_id
            )
            .unwrap()
            .unwrap()
            .status,
            SemanticProposalStatus::Pending
        );
        propagation_proposal_review::record_reject_propagation_proposal(
            &mut conn,
            &CommandEnvelope::new(RejectPropagationProposalCommand {
                proposal_id: command.payload.proposal_id.clone(),
                reason: None,
            }),
            50,
        )
        .unwrap();
        assert_eq!(text(&conn, edited), original);
    }
}

#[test]
fn edit_during_provider_io_or_an_unproven_cause_cannot_create_a_proposal() {
    let (mut conn, _, a, b, _) = fixture();
    let command = request(&conn, &b);
    let binding = capture(&conn, &command.payload).unwrap();
    edit(&mut conn, &a, "Changed during preview");
    assert!(record_proposal(&mut conn, &command, binding, "Obsolete draft".into(), 30).is_err());
    assert!(
        propagation_proposal_store::load_propagation_proposals(&conn)
            .unwrap()
            .is_empty()
    );
    let mut invented = request(&conn, &b);
    invented.payload.dependency_id = SemanticDependencyId::new("invented.cause").unwrap();
    assert!(capture(&conn, &invented.payload).is_err());

    let mut alternate = b.clone();
    alternate.block_id = ScriptBlockId::new("block.B.manual").unwrap();
    alternate.text = "Separate manually authored block".into();
    script_document_command::apply_set_script_block(
        &mut conn,
        &CommandEnvelope::new(alternate.clone()),
        35,
    )
    .unwrap();
    let mut wrong_target = request(&conn, &b);
    wrong_target.payload.block_id = alternate.block_id.clone();
    wrong_target.payload.expected_block_revision_event_id = inputs(&conn, &alternate)
        .into_iter()
        .find(|input| input.block_id == alternate.block_id)
        .unwrap()
        .revision_event_id;
    assert!(
        capture(&conn, &wrong_target.payload).is_err(),
        "another block cannot resolve the generated output's impact"
    );
}

#[test]
fn acceptance_failure_rolls_back_proposal_output_history_and_refreshed_lineage() {
    let (mut conn, _, _, b, _) = fixture();
    let command = propose(&mut conn, &b);
    let before = script_store::load_document_projection(&conn, &b.document_id).unwrap();
    let count = |conn: &Connection| {
        conn.query_row("SELECT (SELECT COUNT(*) FROM commands), (SELECT COUNT(*) FROM object_revisions), (SELECT COUNT(*) FROM script_generations)", [], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,i64>(1)?, r.get::<_,i64>(2)?))).unwrap()
    };
    let counts = count(&conn);
    conn.execute_batch("CREATE TRIGGER reject_binding BEFORE INSERT ON semantic_dependency_revisions BEGIN SELECT RAISE(ABORT, 'fixture acceptance failure'); END").unwrap();
    assert!(
        accept(&mut conn, &command)
            .unwrap_err()
            .to_string()
            .contains("fixture acceptance failure")
    );
    assert_eq!(count(&conn), counts);
    assert_eq!(
        script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        before
    );
    assert_eq!(
        propagation_proposal_store::load_propagation_proposal(&conn, &command.payload.proposal_id)
            .unwrap()
            .unwrap()
            .status,
        SemanticProposalStatus::Pending
    );
}

#[test]
fn bound_proposals_cannot_be_retargeted_and_request_replay_does_not_regenerate() {
    let (mut conn, _, _, b, _) = fixture();
    let command = propose(&mut conn, &b);
    assert_eq!(
        history_store::check_recorded_command(&conn, &command, REQUEST_TYPE).unwrap(),
        Some(RecordChangeOutcome::AlreadyRecorded)
    );
    let mut other = command.clone();
    other.payload.story_time_ms = Some(0);
    assert!(history_store::check_recorded_command(&conn, &other, REQUEST_TYPE).is_err());
    let proposal =
        propagation_proposal_store::load_propagation_proposal(&conn, &command.payload.proposal_id)
            .unwrap()
            .unwrap();
    let update = UpdatePropagationProposalCommand {
        proposal_id: proposal.id,
        action: proposal.action,
        target: proposal.target,
        summary: "Amend".into(),
        proposed_value: None,
        proposed_text: Some("New draft".into()),
        proposed_script_patch: None,
        source_dependency_id: proposal.source_dependency_id,
        source_event_id: proposal.source_event_id,
        rationale: None,
    };
    assert!(
        propagation_proposal_update::record_update_propagation_proposal(
            &mut conn,
            &CommandEnvelope::new(update),
            50
        )
        .is_err()
    );
}
