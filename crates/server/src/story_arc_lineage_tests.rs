use super::*;
use crate::script_impact_review::tests::{accept, fixture, request, text};
use crate::{script_document_command, script_impact_review, story_arc_command};
use eidetic_core::story::arc::{ArcType, Color};

#[path = "story_arc_lineage_guard_tests.rs"]
mod guards;

const OLD: &str = "Mara conceals the witness's identity.";
const NEW: &str = "Mara reveals the witness's identity.";
const PREVIEW: &str = "Synthetic preview: Mara names the witness.\n\n";

fn create(conn: &mut Connection, description: &str) -> ArcId {
    let id = ArcId::new();
    story_arc_command::record_create_story_arc_history(
        conn,
        &CommandEnvelope::new(CreateStoryArcCommand {
            arc_id: id,
            parent_arc_id: None,
            name: "Witness".into(),
            description: description.into(),
            arc_type: ArcType::APlot,
            color: Color::new(100, 149, 237),
        }),
        1,
    )
    .unwrap();
    id
}
fn change(conn: &mut Connection, arc: ArcId, description: &str) {
    metadata(
        conn,
        SetStoryArcMetadataCommand {
            arc_id: arc,
            name: None,
            description: Some(description.into()),
            arc_type: None,
            color: None,
        },
    );
}
fn metadata(conn: &mut Connection, payload: SetStoryArcMetadataCommand) {
    story_arc_command::record_set_story_arc_metadata_history(
        conn,
        &CommandEnvelope::new(payload),
        2,
    )
    .unwrap();
}
fn tag(conn: &Connection, block: &SetScriptBlockCommand, arc: ArcId) {
    conn.execute(
        "INSERT INTO node_arcs (node_id,arc_id) VALUES (?1,?2)",
        params![block.source_node_id, arc.0.to_string()],
    )
    .unwrap();
}
fn inputs(conn: &Connection, b: &SetScriptBlockCommand) -> Vec<StoryArcFieldInput> {
    capture(
        conn,
        NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap()),
        &[],
    )
    .unwrap()
    .1
}
fn generate(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
    inputs: Option<Vec<StoryArcFieldInput>>,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        arc_inputs: inputs,
        block: b.clone(),
        script_inputs: Some(vec![]),
        bible_inputs: Some(vec![]),
        bible_node_name_inputs: None,
        bible_relationship_inputs: None,
        bible_context_scope: None,
        script_context_scope: None,
        target_binding: None,
    });
    script_document_command::apply_generated_script_block(conn, &command, 3).unwrap();
    command
}
fn impact(conn: &Connection, b: &SetScriptBlockCommand) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &b.segment_id)
        .unwrap()
        .unwrap()
}
fn setup() -> (
    Connection,
    ArcId,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
    SetScriptBlockCommand,
) {
    let (mut conn, _, a, b, c) = fixture();
    let arc = create(&mut conn, OLD);
    tag(&conn, &b, arc);
    let evidence = inputs(&conn, &b);
    generate(&mut conn, &b, Some(evidence));
    generate(&mut conn, &c, Some(vec![]));
    (conn, arc, a, b, c)
}
fn preview(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let command = request(conn, b);
    let binding = script_impact_review::capture(conn, &command.payload).unwrap();
    script_impact_review::record_proposal(conn, &command, binding, PREVIEW.into(), 4).unwrap();
    command
}
fn delete(conn: &mut Connection, arc: ArcId) {
    story_arc_command::record_delete_story_arc_history(
        conn,
        &CommandEnvelope::new(DeleteStoryArcCommand { arc_id: arc }),
        5,
    )
    .unwrap();
}

#[test]
fn typed_description_change_reviews_only_consuming_scene_then_explicit_acceptance_refreshes_lineage()
 {
    let (mut conn, arc, a, b, c) = setup();
    let before = (text(&conn, &a), text(&conn, &b), text(&conn, &c));
    let original = inputs(&conn, &b);
    change(&mut conn, arc, NEW);
    let review = impact(&conn, &b);
    assert_eq!(review.causes.len(), 1);
    assert_eq!(
        review.causes[0].input,
        SemanticDependencyEndpoint::StoryArcField {
            arc_id: arc,
            field: StoryArcPromptField::Description
        }
    );
    assert_eq!(review.causes[0].input_excerpt.as_deref(), Some(OLD));
    assert!(!impact(&conn, &c).needs_review);
    let command = preview(&mut conn, &b);
    let proposals =
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap();
    let binding = proposals.payload.proposals[0]
        .script_review_binding
        .as_ref()
        .unwrap();
    assert_eq!(binding.arc_previous_inputs.as_ref().unwrap(), &original);
    let current = binding
        .arc_inputs
        .as_ref()
        .unwrap()
        .iter()
        .find(|input| input.field == StoryArcPromptField::Description)
        .unwrap();
    assert_eq!(current.value, NEW);
    assert_eq!(
        current.revision_event_id,
        review.causes[0].current_revision_event_id
    );
    assert_eq!((text(&conn, &a), text(&conn, &b), text(&conn, &c)), before);
    accept(&mut conn, &command).unwrap();
    assert_eq!(text(&conn, &b), PREVIEW);
    assert_eq!((text(&conn, &a), text(&conn, &c)), (before.0, before.2));
    assert!(!impact(&conn, &b).needs_review);
    change(&mut conn, arc, OLD);
    assert!(impact(&conn, &b).needs_review);
    assert_eq!(
        recorded(&conn, impact(&conn, &b).generation_event_id)
            .unwrap()
            .unwrap()
            .iter()
            .find(|input| input.field == StoryArcPromptField::Description)
            .unwrap()
            .value,
        NEW
    );
}

#[test]
fn late_generation_keeps_original_field_revision_across_aba_and_replay() {
    let (mut conn, arc, _, b, _) = setup();
    let original = inputs(&conn, &b);
    change(&mut conn, arc, NEW);
    change(&mut conn, arc, OLD);
    let command = generate(&mut conn, &b, Some(original.clone()));
    let review = impact(&conn, &b);
    assert!(review.needs_review);
    assert_ne!(
        review.causes[0].current_revision_event_id,
        Some(review.causes[0].consumed_revision_event_id)
    );
    assert_eq!(
        recorded(&conn, review.generation_event_id).unwrap(),
        Some(original)
    );
    script_document_command::apply_generated_script_block(&mut conn, &command, 6).unwrap();
    assert_eq!(impact(&conn, &b), review);
}

#[test]
fn unconsumed_arcs_and_color_changes_do_not_create_causes_or_stale_a_preview() {
    let (mut conn, arc, _, b, _) = setup();
    metadata(
        &mut conn,
        SetStoryArcMetadataCommand {
            arc_id: arc,
            name: None,
            description: None,
            arc_type: None,
            color: Some(Color::new(1, 2, 3)),
        },
    );
    let other = create(&mut conn, "Unconsumed plot.");
    change(&mut conn, other, "Unconsumed edit.");
    assert!(!impact(&conn, &b).needs_review);
    change(&mut conn, arc, NEW);
    let command = preview(&mut conn, &b);
    metadata(
        &mut conn,
        SetStoryArcMetadataCommand {
            arc_id: arc,
            name: None,
            description: None,
            arc_type: None,
            color: Some(Color::new(4, 5, 6)),
        },
    );
    change(&mut conn, other, "Further unrelated edit.");
    accept(&mut conn, &command).unwrap();
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn name_and_type_have_separate_owned_clocks() {
    let (mut conn, arc, _, b, _) = setup();
    metadata(
        &mut conn,
        SetStoryArcMetadataCommand {
            arc_id: arc,
            name: Some("Confession".into()),
            description: None,
            arc_type: Some(ArcType::BPlot),
            color: None,
        },
    );
    let review = impact(&conn, &b);
    assert_eq!(review.causes.len(), 2);
    assert!(review.causes.iter().all(|cause| matches!(
        cause.input,
        SemanticDependencyEndpoint::StoryArcField {
            field: StoryArcPromptField::Name | StoryArcPromptField::ArcType,
            ..
        }
    )));
}

#[test]
fn legacy_receipts_are_not_reconstructed_from_current_tags() {
    let (mut conn, arc, _, b, _) = setup();
    generate(&mut conn, &b, None);
    change(&mut conn, arc, NEW);
    assert!(!impact(&conn, &b).needs_review);
    assert_eq!(
        recorded(&conn, impact(&conn, &b).generation_event_id).unwrap(),
        None
    );
}

#[test]
fn template_fields_without_owned_history_remain_unbound() {
    let (mut conn, _, _, b, _) = setup();
    let arc =
        eidetic_core::story::arc::StoryArc::new("Legacy", ArcType::CRunner, Color::new(1, 2, 3));
    let tx = conn.transaction().unwrap();
    crate::story_arc_store::insert_arc_in_transaction(
        &tx,
        &arc,
        ChangeEventId(uuid::Uuid::new_v4()),
    )
    .unwrap();
    tx.commit().unwrap();
    tag(&conn, &b, arc.id);
    let captured = inputs(&conn, &b);
    assert!(
        captured
            .iter()
            .filter(|input| input.arc_id == arc.id)
            .all(|input| input.revision_event_id.is_none())
    );
    generate(&mut conn, &b, Some(captured));
    change(&mut conn, arc.id, NEW);
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn unconsumed_empty_description_is_not_backfilled_after_an_edit() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, "");
    let captured = inputs(&conn, &b);
    assert!(
        !captured
            .iter()
            .any(|input| input.field == StoryArcPromptField::Description)
    );
    generate(&mut conn, &b, Some(captured));
    change(&mut conn, arc, NEW);
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn forged_field_value_refuses_generation_atomically_and_canonical_history_drift_refuses_capture() {
    let (mut conn, arc, _, b, _) = setup();
    let mut captured = inputs(&conn, &b);
    captured[0].value = "Forged name".into();
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        arc_inputs: Some(captured),
        block: b.clone(),
        script_inputs: Some(vec![]),
        bible_inputs: None,
        bible_node_name_inputs: None,
        bible_relationship_inputs: None,
        bible_context_scope: None,
        script_context_scope: None,
        target_binding: None,
    });
    let before = impact(&conn, &b);
    assert!(script_document_command::apply_generated_script_block(&mut conn, &command, 6).is_err());
    assert_eq!(impact(&conn, &b), before);
    assert_eq!(text(&conn, &b), b.text);
    conn.execute(
        "UPDATE arcs SET description='unrecorded' WHERE id=?1",
        [arc.0.to_string()],
    )
    .unwrap();
    assert!(
        capture(
            &conn,
            NodeId(uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap()),
            &[]
        )
        .is_err()
    );
}

#[test]
fn clearing_a_consumed_description_keeps_current_empty_evidence_and_new_lineage() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, "");
    let command = preview(&mut conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert!(
        binding
            .arc_inputs
            .as_ref()
            .unwrap()
            .iter()
            .any(|input| input.field == StoryArcPromptField::Description
                && input.value.is_empty()
                && input.revision_event_id.is_some())
    );
    accept(&mut conn, &command).unwrap();
    change(&mut conn, arc, NEW);
    assert!(impact(&conn, &b).needs_review);
}

#[test]
fn description_aba_refuses_stale_acceptance_without_changing_saved_text() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    let command = preview(&mut conn, &b);
    change(&mut conn, arc, OLD);
    change(&mut conn, arc, NEW);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(text(&conn, &b), b.text);
}

#[test]
fn source_edit_during_preview_refuses_late_output() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    change(&mut conn, arc, "Mara retracts her accusation.");
    assert!(
        script_impact_review::record_proposal(&mut conn, &command, binding, PREVIEW.into(), 6)
            .is_err()
    );
    assert_eq!(text(&conn, &b), b.text);
}

#[test]
fn deletion_refuses_old_preview_and_fresh_preview_binds_absence() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    let old = preview(&mut conn, &b);
    delete(&mut conn, arc);
    assert!(accept(&mut conn, &old).is_err());
    assert!(
        impact(&conn, &b)
            .causes
            .iter()
            .all(|cause| cause.reason == ScriptImpactReason::Deleted)
    );
    let fresh = preview(&mut conn, &b);
    let binding = script_impact_review::capture(&conn, &fresh.payload).unwrap();
    assert!(binding.arc_inputs.as_ref().unwrap().is_empty());
    assert_eq!(binding.arc_absence_revisions.as_ref().unwrap().len(), 1);
    accept(&mut conn, &fresh).unwrap();
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn live_consumed_arc_leaving_tags_refuses_preview_instead_of_dropping_lineage() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    conn.execute(
        "DELETE FROM node_arcs WHERE node_id=?1 AND arc_id=?2",
        params![b.source_node_id, arc.0.to_string()],
    )
    .unwrap();
    let command = request(&conn, &b);
    assert!(script_impact_review::capture(&conn, &command.payload).is_err());
}

#[test]
fn target_edit_refuses_arc_preview_without_losing_manual_text() {
    let (mut conn, arc, _, b, _) = setup();
    change(&mut conn, arc, NEW);
    let command = preview(&mut conn, &b);
    crate::script_impact_review::tests::edit(&mut conn, &b, "Exact manual screenplay\n\n");
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(text(&conn, &b), "Exact manual screenplay\n\n");
}
