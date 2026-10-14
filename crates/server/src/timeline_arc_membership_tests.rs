use super::*;
use crate::script_impact_review::tests::{accept, request};
use crate::timeline_notes_lineage::tests::{logical_rows, materials};
use crate::{
    script_document_command, script_impact_review, story_arc_command,
    timeline_arc_assignment_history, timeline_node_store,
};
use eidetic_core::{
    Project,
    story::arc::{ArcId, ArcType, Color},
};

fn create(conn: &mut Connection, name: &str) -> ArcId {
    let arc_id = ArcId::new();
    story_arc_command::record_create_story_arc_history(
        conn,
        &CommandEnvelope::new(CreateStoryArcCommand {
            arc_id,
            parent_arc_id: None,
            name: name.into(),
            description: format!("{name} exact authored arc — 雨."),
            arc_type: ArcType::APlot,
            color: Color::new(100, 149, 237),
        }),
        1,
    )
    .unwrap();
    arc_id
}
fn refresh(conn: &Connection, project: &mut Project) {
    project.timeline.nodes = timeline_node_store::load_nodes(conn).unwrap();
    project.timeline.node_arcs = timeline_node_store::load_node_arcs(conn).unwrap();
}
fn assignment(
    conn: &Connection,
    node: NodeId,
    arc_ids: Vec<ArcId>,
) -> CommandEnvelope<SetTimelineNodeArcsCommand> {
    CommandEnvelope::new(SetTimelineNodeArcsCommand {
        node_id: node,
        arc_ids,
        expected: capture(conn, node).unwrap(),
    })
}
fn assign(
    conn: &mut Connection,
    project: &mut Project,
    node: NodeId,
    ids: Vec<ArcId>,
) -> CommandEnvelope<SetTimelineNodeArcsCommand> {
    refresh(conn, project);
    let command = assignment(conn, node, ids);
    timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
        conn, project, &command, 10,
    )
    .unwrap();
    refresh(conn, project);
    command
}
fn setup() -> (
    Connection,
    Project,
    CommandEnvelope<GenerateScriptBlockCommand>,
    ArcId,
    ArcId,
) {
    let (mut conn, project, generation) = crate::script_generation_target::tests::fixture();
    let a = create(&mut conn, "Witness");
    let b = create(&mut conn, "Departure");
    (conn, project, generation, a, b)
}
fn generate(
    conn: &mut Connection,
    project: &mut Project,
    generation: &mut CommandEnvelope<GenerateScriptBlockCommand>,
) {
    refresh(conn, project);
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    generation.payload.target_binding = Some(
        crate::script_generation_target::capture(conn, project.timeline.node(node).unwrap())
            .unwrap(),
    );
    generation.payload.arc_inputs = Some(
        crate::story_arc_lineage::capture(conn, node, &[])
            .unwrap()
            .1,
    );
    script_document_command::apply_generated_script_block(conn, generation, 20).unwrap();
}
fn review(
    conn: &Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &generation.payload.block.segment_id)
        .unwrap()
        .unwrap()
}
#[test]
fn first_assignment_last_removal_and_accepted_successor_keep_membership_custody() {
    let (mut conn, mut project, mut generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    generate(&mut conn, &mut project, &mut generation);
    assert!(!review(&conn, &generation).needs_review);
    let original = materials(&conn);
    assign(&mut conn, &mut project, node, vec![a]);
    assert_eq!(materials(&conn), original);
    assert_eq!(review(&conn, &generation).causes.len(), 1);
    let request = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
    assert!(
        binding
            .arc_membership_previous
            .as_ref()
            .unwrap()
            .arc_ids
            .is_empty()
    );
    assert_eq!(
        binding.arc_membership_current.as_ref().unwrap().arc_ids,
        vec![a]
    );
    script_impact_review::record_proposal(
        &mut conn,
        &request,
        binding,
        "Labelled synthetic first assignment preview".into(),
        30,
    )
    .unwrap();
    assert_eq!(materials(&conn), original);
    accept(&mut conn, &request).unwrap();
    assert!(!review(&conn, &generation).needs_review);
    assign(&mut conn, &mut project, node, vec![]);
    let review = review(&conn, &generation);
    assert!(review.needs_review);
    let second = request_for(
        &conn,
        &generation,
        review
            .causes
            .iter()
            .find(|cause| cause.dependency_id.as_str().ends_with(".arc_membership"))
            .unwrap()
            .dependency_id
            .clone(),
    );
    let binding = script_impact_review::capture(&conn, &second.payload).unwrap();
    assert_eq!(binding.arc_membership_previous.unwrap().arc_ids, vec![a]);
    assert!(binding.arc_membership_current.unwrap().arc_ids.is_empty());
    assert!(binding.arc_inputs.unwrap().is_empty());
    assert!(permits_withdrawal(&conn, node, second.payload.generation_event_id, a).unwrap());
    assign(&mut conn, &mut project, node, vec![b]);
    assert!(review_again(&conn, &generation));
}
fn request_for(
    conn: &Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
    id: SemanticDependencyId,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let mut command = request(conn, &generation.payload.block);
    command.payload.dependency_id = id;
    command
}
fn review_again(
    conn: &Connection,
    generation: &CommandEnvelope<GenerateScriptBlockCommand>,
) -> bool {
    review(conn, generation).needs_review
}
#[test]
fn save_is_atomic_deduplicated_and_replay_returns_original_receipt_after_newer_assignment() {
    let (mut conn, mut project, generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let first = assign(&mut conn, &mut project, node, vec![a]);
    let receipt = command_receipt(&conn, &first).unwrap();
    assign(&mut conn, &mut project, node, vec![b]);
    let before = logical_rows(&conn);
    assert_eq!(
        timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
            &mut conn, &project, &first, 40
        )
        .unwrap(),
        crate::history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(command_receipt(&conn, &first).unwrap(), receipt);
    assert_eq!(logical_rows(&conn), before);
    for ids in [vec![a, a], vec![ArcId::new()], vec![b]] {
        let command = assignment(&conn, node, ids);
        let before = logical_rows(&conn);
        assert!(
            timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
                &mut conn, &project, &command, 40
            )
            .is_err()
        );
        assert_eq!(logical_rows(&conn), before);
    }
}
#[test]
fn stale_save_and_aba_refuse_without_rows_or_manual_screenplay_changes() {
    let (mut conn, mut project, generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let stale = assignment(&conn, node, vec![b]);
    assign(&mut conn, &mut project, node, vec![a]);
    assign(&mut conn, &mut project, node, vec![]);
    let before = logical_rows(&conn);
    assert!(
        timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
            &mut conn, &project, &stale, 50
        )
        .is_err()
    );
    assert_eq!(logical_rows(&conn), before);
}
#[test]
fn provider_pending_generation_preview_and_acceptance_refuse_membership_aba() {
    let (mut conn, mut project, mut generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assign(&mut conn, &mut project, node, vec![a]);
    refresh(&conn, &mut project);
    generation.payload.target_binding = Some(
        crate::script_generation_target::capture(&conn, project.timeline.node(node).unwrap())
            .unwrap(),
    );
    generation.payload.arc_inputs = Some(
        crate::story_arc_lineage::capture(&conn, node, &[])
            .unwrap()
            .1,
    );
    assign(&mut conn, &mut project, node, vec![b]);
    assign(&mut conn, &mut project, node, vec![a]);
    let before = logical_rows(&conn);
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &generation, 20).is_err()
    );
    assert_eq!(logical_rows(&conn), before);
    generate(&mut conn, &mut project, &mut generation);
    assign(&mut conn, &mut project, node, vec![b]);
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assign(&mut conn, &mut project, node, vec![a]);
    assign(&mut conn, &mut project, node, vec![b]);
    crate::propagation_proposal_store::create_schema(&conn).unwrap();
    let before = logical_rows(&conn);
    assert!(
        script_impact_review::record_proposal(
            &mut conn,
            &command,
            binding,
            "Labelled synthetic stale result".into(),
            60
        )
        .is_err()
    );
    assert_eq!(logical_rows(&conn), before);
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &command,
        binding,
        "Labelled synthetic pending preview".into(),
        60,
    )
    .unwrap();
    assign(&mut conn, &mut project, node, vec![a]);
    assign(&mut conn, &mut project, node, vec![b]);
    let before = logical_rows(&conn);
    assert!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &CommandEnvelope::new(AcceptPropagationProposalCommand {
                proposal_id: command.payload.proposal_id
            }),
            70
        )
        .is_err()
    );
    assert_eq!(logical_rows(&conn), before);
}
#[test]
fn unrelated_clip_assignment_does_not_stale_target_and_legacy_consumption_stays_unknown() {
    let (mut conn, mut project, mut generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assign(&mut conn, &mut project, node, vec![a]);
    generate(&mut conn, &mut project, &mut generation);
    let other = project
        .timeline
        .nodes
        .iter()
        .find(|candidate| candidate.id != node)
        .unwrap()
        .id;
    assign(&mut conn, &mut project, other, vec![b]);
    assert!(!review(&conn, &generation).needs_review);
    generation.id = CommandId(uuid::Uuid::new_v4());
    generation.payload.target_binding = None;
    script_document_command::apply_generated_script_block(&mut conn, &generation, 80).unwrap();
    assign(&mut conn, &mut project, node, vec![]);
    assert!(
        recorded(&conn, review(&conn, &generation).generation_event_id)
            .unwrap()
            .is_none()
    );
    assert!(
        !permits_withdrawal(
            &conn,
            node,
            review(&conn, &generation).generation_event_id,
            a
        )
        .unwrap()
    );
    // A known arc-field edit still exposes the legacy review, but removal cannot
    // broaden the guarded prompt into unverified withdrawal.
    story_arc_command::record_set_story_arc_metadata_history(
        &mut conn,
        &CommandEnvelope::new(SetStoryArcMetadataCommand {
            arc_id: a,
            name: Some("Changed name".into()),
            description: None,
            arc_type: None,
            color: None,
        }),
        90,
    )
    .unwrap();
    let command = request(&conn, &generation.payload.block);
    assert!(script_impact_review::capture(&conn, &command.payload).is_err());
}

#[test]
fn locked_clip_refuses_fresh_assignment_but_original_acknowledgement_replays() {
    let (mut conn, mut project, generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    let original = assign(&mut conn, &mut project, node, vec![a]);
    let receipt = command_receipt(&conn, &original).unwrap();
    crate::timeline_command_history::record_set_timeline_node_lock_history(
        &mut conn,
        &project,
        &CommandEnvelope::new(SetTimelineNodeLockCommand {
            node_id: node,
            locked: true,
        }),
        80,
    )
    .unwrap();
    refresh(&conn, &mut project);
    let fresh = assignment(&conn, node, vec![b]);
    let before = logical_rows(&conn);
    assert!(
        timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
            &mut conn, &project, &fresh, 90
        )
        .is_err()
    );
    assert_eq!(logical_rows(&conn), before);
    assert_eq!(
        timeline_arc_assignment_history::record_set_timeline_node_arcs_history(
            &mut conn, &project, &original, 90
        )
        .unwrap(),
        crate::history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(command_receipt(&conn, &original).unwrap(), receipt);
    assert_eq!(logical_rows(&conn), before);
}

#[tokio::test]
async fn last_removal_provider_receives_historical_identity_and_explicit_current_empty_assignment()
{
    let (mut conn, mut project, mut generation, a, _) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assign(&mut conn, &mut project, node, vec![a]);
    generate(&mut conn, &mut project, &mut generation);
    assign(&mut conn, &mut project, node, vec![]);
    let command = request(&conn, &generation.payload.block);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let output =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(
                "ORIGINAL RECORDED STORY ARC ASSIGNMENT (historical, not current guidance)"
            ));
            assert!(prompt.user.contains(&format!("{} name: Witness", a.0)));
            assert!(
                prompt
                    .user
                    .contains("CURRENT SELECTED CLIP STORY ARC ASSIGNMENT")
            );
            assert!(prompt.user.contains("(no arcs)"));
            assert!(!prompt.user.contains("CURRENT TAGGED STORY ARC FIELDS"));
            assert!(!prompt.user.contains("Witness exact authored arc"));
            let stream: eidetic_core::ai::backend::GenerateStream = Box::pin(
                futures::stream::iter(vec![Ok("LABELLED SYNTHETIC last removal update".into())]),
            );
            Ok(stream)
        })
        .await
        .unwrap();
    assert_eq!(output, "LABELLED SYNTHETIC last removal update");
}

#[test]
fn forged_wrong_node_or_field_receipts_and_unlogged_tag_mutation_refuse() {
    let (mut conn, mut project, mut generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assign(&mut conn, &mut project, node, vec![a]);
    generate(&mut conn, &mut project, &mut generation);
    let generation_event = review(&conn, &generation).generation_event_id;
    let receipt = capture(&conn, node).unwrap();
    let other = project
        .timeline
        .nodes
        .iter()
        .find(|candidate| candidate.id != node)
        .unwrap()
        .id;
    assign(&mut conn, &mut project, other, vec![b]);
    let mut forged = receipt.clone();
    forged.revision_event_id = capture(&conn, other).unwrap().revision_event_id;
    assert!(validate_history(&conn, &forged).is_err());
    crate::timeline_notes_lineage::tests::notes(&mut conn, &mut project, node, "New Notes");
    forged.revision_event_id = crate::timeline_notes_lineage::capture(&conn, node)
        .unwrap()
        .revision_event_id;
    assert!(validate_history(&conn, &forged).is_err());
    // Deliberate corrupted fixture, not a public authoring path: raw tag removal
    // cannot masquerade as a proven owned transition or broaden the prompt.
    conn.execute(
        "DELETE FROM node_arcs WHERE node_id=?1",
        [node.0.to_string()],
    )
    .unwrap();
    assert!(capture(&conn, node).is_err());
    assert!(permits_withdrawal(&conn, node, generation_event, a).is_err());
}

#[test]
fn inherited_legacy_arc_field_order_normalizes_as_membership_set() {
    let (mut conn, mut project, generation, a, b) = setup();
    let node = generation.payload.target_binding.as_ref().unwrap().node_id;
    assign(&mut conn, &mut project, node, vec![a, b]);
    let mut reverse = vec![a, b];
    reverse.sort_by_key(|id| std::cmp::Reverse(id.0));
    let command = CommandEnvelope::new(SetTimelineNodeLockCommand {
        node_id: node,
        locked: false,
    });
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::UserEdit,
        "synthetic legacy inherited tag order",
    );
    let revision = ObjectRevision::new(
        ObjectKind::TimelineNode,
        node.0.to_string(),
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "arc_ids",
        None,
        Some(FieldValue::Text(serde_json::to_string(&reverse).unwrap())),
    ));
    crate::history_store::record_change(
        &mut conn,
        &command,
        "test.legacy_inherited_tags",
        &event,
        &[revision],
    )
    .unwrap();
    let read = capture(&conn, node).unwrap();
    assert_eq!(read.revision_event_id, Some(event.id));
    assert!(read.arc_ids[0].0 < read.arc_ids[1].0);
    validate_history(&conn, &read).unwrap();
}
