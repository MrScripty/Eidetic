use super::*;
use crate::script_impact_review::tests::{accept, edit, fixture, request, text};
use crate::{bible_graph_command, script_document_command, script_impact_review, script_store};

#[path = "bible_membership_preview_limit_tests.rs"]
mod preview_limit;

fn entity(conn: &mut Connection, id: &str) {
    bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new(id).unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: id.into(),
            sort_order: 0,
        }),
        1,
    )
    .unwrap();
}

fn field(conn: &mut Connection, node: &str, key: &str, value: Option<&str>) {
    bible_graph_command::apply_set_bible_graph_field(
        conn,
        &CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new(node).unwrap(),
            part_id: BibleGraphPartId::new(format!("{node}.profile")).unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            part_sort_order: 0,
            field_id: BibleGraphFieldId::new(format!("{node}.{key}")).unwrap(),
            field_key: BibleGraphFieldKey::new(key).unwrap(),
            value: value.map(|value| FieldValue::Text(value.into())),
            field_sort_order: 0,
        }),
        2,
    )
    .unwrap();
}

fn node(block: &SetScriptBlockCommand) -> NodeId {
    NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap())
}

fn inputs(conn: &Connection, block: &SetScriptBlockCommand) -> Vec<BibleFieldInput> {
    let context =
        crate::ai_context_projection::load_ai_bible_context_projection(conn, node(block), None)
            .unwrap();
    crate::bible_field_lineage::capture(conn, &context.payload).unwrap()
}

fn generation(
    conn: &Connection,
    block: &SetScriptBlockCommand,
    scoped: bool,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let inputs = inputs(conn, block);
    CommandEnvelope::new(GenerateScriptBlockCommand {
        block: block.clone(),
        script_inputs: Some(vec![]),
        script_context_scope: None,
        target_binding: None,
        bible_context_scope: scoped.then(|| capture(conn, node(block), &inputs).unwrap()),
        bible_relationship_inputs: None,
        bible_inputs: Some(inputs),
    })
}

fn impact(conn: &Connection, block: &SetScriptBlockCommand) -> ScriptImpactProjection {
    script_store::load_document_projection(conn, &block.document_id)
        .unwrap()
        .unwrap()
        .segments
        .into_iter()
        .find(|segment| segment.segment.id == block.segment_id)
        .unwrap()
        .impact
        .unwrap()
}

fn membership(conn: &Connection, block: &SetScriptBlockCommand) -> Option<ScriptImpactCause> {
    impact(conn, block)
        .causes
        .into_iter()
        .find(|cause| cause.reason == ScriptImpactReason::ContextChanged)
}

fn assign(
    conn: &mut Connection,
    block: &SetScriptBlockCommand,
    target: Option<&str>,
    kind: ContextInfluenceKind,
) {
    let id = ContextEvaluationId::new();
    let evaluation = ContextEvaluation {
        id,
        target_node_id: node(block),
        task_kind: ContextEvaluationTaskKind::GenerateScript,
        summary: "Explicit fixture assignment".into(),
        distilled_context: None,
        created_at_ms: conn
            .query_row("SELECT COUNT(*)+100 FROM change_events", [], |r| {
                r.get::<_, u64>(0)
            })
            .unwrap(),
    };
    let influences = target
        .map(|target| {
            vec![ContextInfluenceRecord {
                id: ContextInfluenceId::new(),
                evaluation_id: id,
                timeline_node_id: node(block),
                source_layer: eidetic_core::timeline::node::StoryLevel::Scene,
                influence_kind: kind,
                confidence: 1.0,
                reason: "Fixture explicit selection".into(),
                provenance: ContextInfluenceProvenance::UserSelected,
                bible_node_id: Some(BibleGraphNodeId::new(target).unwrap()),
                bible_edge_id: None,
                introduced_by_node_id: None,
                sort_order: 0,
            }]
        })
        .unwrap_or_default();
    crate::context_influence_store::record_context_evaluation(
        conn,
        &CommandEnvelope::new(RecordContextEvaluationCommand {
            evaluation,
            influences,
        }),
        3,
    )
    .unwrap();
}

#[test]
fn scoped_and_legacy_absent_to_present_mark_review_without_rewriting_human_or_unrelated_text() {
    for scoped in [false, true] {
        for existed_null in [false, true] {
            let (mut conn, _, a, b, c) = fixture();
            entity(&mut conn, "Mara");
            field(&mut conn, "Mara", "tagline", Some("Blue umbrella"));
            if existed_null {
                field(&mut conn, "Mara", "motivation", None);
            }
            let command = generation(&conn, &b, scoped);
            script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
            edit(&mut conn, &b, "Exact manual B.\n\n");
            let before_a = text(&conn, &a);
            let before_c = text(&conn, &c);
            entity(&mut conn, "Unrelated");
            field(
                &mut conn,
                "Unrelated",
                "tagline",
                Some("Unrelated new fact"),
            );
            assert!(!impact(&conn, &b).needs_review);
            field(
                &mut conn,
                "Mara",
                "motivation",
                Some("Keep the station key"),
            );
            let cause = membership(&conn, &b).unwrap();
            assert!(
                cause
                    .input_excerpt
                    .unwrap()
                    .contains("Mara.profile.motivation")
            );
            assert_eq!(text(&conn, &b), "Exact manual B.\n\n");
            assert_eq!(text(&conn, &a), before_a);
            assert_eq!(text(&conn, &c), before_c);
            // Legacy replay retains exact prior payload and does not erase review.
            script_document_command::apply_generated_script_block(&mut conn, &command, 99).unwrap();
            assert_eq!(text(&conn, &b), "Exact manual B.\n\n");
            assert!(membership(&conn, &b).is_some());
        }
    }
}

#[tokio::test]
async fn preview_preserves_manual_text_acceptance_refreshes_membership_and_clear_remove_require_review()
 {
    for scoped in [false, true] {
        let (mut conn, _, a, b, c) = fixture();
        entity(&mut conn, "Mara");
        field(&mut conn, "Mara", "tagline", Some("Blue umbrella"));
        let command = generation(&conn, &b, scoped);
        script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
        edit(&mut conn, &b, "Exact manual B.\n\n");
        field(
            &mut conn,
            "Mara",
            "motivation",
            Some("Keep the station key"),
        );
        let request = request(&conn, &b);
        let binding = script_impact_review::capture(&conn, &request.payload).unwrap();
        assert_eq!(binding.cause.reason, ScriptImpactReason::ContextChanged);
        let proposed =
            crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
                assert!(prompt.user.contains("Exact manual B.\n\n"));
                assert!(prompt.user.contains("Keep the station key"));
                Ok(Box::pin(futures::stream::iter([Ok(
                    "Exact manual B.\nMara keeps the station key.\n\n".into(),
                )]))
                    as eidetic_core::ai::backend::GenerateStream)
            })
            .await
            .unwrap();
        let before_a = text(&conn, &a);
        let before_c = text(&conn, &c);
        script_impact_review::record_proposal(&mut conn, &request, binding, proposed.clone(), 5)
            .unwrap();
        assert_eq!(text(&conn, &b), "Exact manual B.\n\n");
        accept(&mut conn, &request).unwrap();
        assert_eq!(text(&conn, &b), proposed);
        assert!(!impact(&conn, &b).needs_review);
        field(&mut conn, "Mara", "motivation", None);
        assert!(
            membership(&conn, &b)
                .unwrap()
                .input_excerpt
                .unwrap()
                .contains("removed: Mara.profile.motivation")
        );
        assert_eq!(text(&conn, &b), proposed);
        bible_graph_command::apply_delete_bible_graph_node(
            &mut conn,
            &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
                node_id: BibleGraphNodeId::new("Mara").unwrap(),
            }),
            6,
        )
        .unwrap();
        assert!(membership(&conn, &b).is_some());
        assert_eq!(text(&conn, &a), before_a);
        assert_eq!(text(&conn, &c), before_c);
    }
}

#[test]
fn empty_assigned_entities_and_new_direct_assignments_are_scoped_but_ignored_candidates_are_not() {
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
    let command = generation(&conn, &b, true);
    assert!(command.payload.bible_inputs.as_ref().unwrap().is_empty());
    assert_eq!(
        command
            .payload
            .bible_context_scope
            .as_ref()
            .unwrap()
            .node_ids
            .len(),
        1
    );
    script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
    entity(&mut conn, "Other");
    field(&mut conn, "Other", "tagline", Some("Other fact"));
    assign(&mut conn, &b, Some("Other"), ContextInfluenceKind::Ignored);
    assert!(!impact(&conn, &b).needs_review);
    assign(
        &mut conn,
        &b,
        Some("Other"),
        ContextInfluenceKind::Candidate,
    );
    assert!(!impact(&conn, &b).needs_review);
    let before_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
        .unwrap()
        .unwrap()
        .version;
    assign(&mut conn, &b, Some("Other"), ContextInfluenceKind::Direct);
    let after_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
        .unwrap()
        .unwrap()
        .version;
    assert!(after_version.0 > before_version.0);
    assert!(membership(&conn, &b).is_some());
    assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
    assert!(!impact(&conn, &b).needs_review);
    field(&mut conn, "Mara", "motivation", Some("Mara new fact"));
    assert!(membership(&conn, &b).is_some());
}

#[test]
fn membership_aba_and_assignment_aba_refuse_pending_acceptance_and_stale_requests() {
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue"));
    let command = generation(&conn, &b, true);
    script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
    field(&mut conn, "Mara", "motivation", Some("Keep key"));
    let req = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &req,
        binding,
        "Pending replacement".into(),
        5,
    )
    .unwrap();
    field(&mut conn, "Mara", "motivation", None);
    assert!(script_impact_review::capture(&conn, &req.payload).is_err());
    field(&mut conn, "Mara", "motivation", Some("Keep key"));
    let events: i64 = conn
        .query_row("SELECT COUNT(*) FROM change_events", [], |r| r.get(0))
        .unwrap();
    assert!(accept(&mut conn, &req).is_err());
    assert_eq!(text(&conn, &b), "Original B");
    assert_eq!(
        events,
        conn.query_row("SELECT COUNT(*) FROM change_events", [], |r| r
            .get::<_, i64>(0))
            .unwrap()
    );
    let req = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
    assign(&mut conn, &b, None, ContextInfluenceKind::Direct);
    assert!(
        script_impact_review::record_proposal(&mut conn, &req, binding, "Late preview".into(), 6)
            .is_err()
    );
    let req = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    edit(&mut conn, &b, "Human typed while provider was pending.\n");
    assert!(
        script_impact_review::record_proposal(&mut conn, &req, binding, "Late preview".into(), 7)
            .is_err()
    );
    assert_eq!(text(&conn, &b), "Human typed while provider was pending.\n");
}

#[test]
fn late_generation_keeps_captured_membership_and_legacy_unknown_historical_fields_are_not_invented()
{
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue"));
    let command = generation(&conn, &b, true);
    field(&mut conn, "Mara", "motivation", Some("New during HTTP"));
    script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
    assert!(membership(&conn, &b).is_some());
    let mut legacy = generation(&conn, &b, false);
    legacy
        .payload
        .bible_inputs
        .as_mut()
        .unwrap()
        .retain(|input| input.field_key.as_str() == "tagline");
    script_document_command::apply_generated_script_block(&mut conn, &legacy, 5).unwrap();
    assert!(
        membership(&conn, &b).is_none(),
        "Unknown historical field is not fabricated as absent"
    );
    field(
        &mut conn,
        "Mara",
        "motivation",
        Some("Changed unknown field"),
    );
    assert!(membership(&conn, &b).is_none());
}

#[test]
fn forged_incomplete_or_unrelated_scope_rolls_back_and_timed_keys_do_not_enter_membership() {
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue"));
    entity(&mut conn, "Other");
    let command = generation(&conn, &b, true);
    let mut bad = command.clone();
    bad.id = CommandId(uuid::Uuid::new_v4());
    bad.payload
        .bible_context_scope
        .as_mut()
        .unwrap()
        .field_ids
        .clear();
    assert!(script_document_command::apply_generated_script_block(&mut conn, &bad, 4).is_err());
    let mut bad = command.clone();
    bad.id = CommandId(uuid::Uuid::new_v4());
    bad.payload
        .bible_context_scope
        .as_mut()
        .unwrap()
        .node_ids
        .push(BibleGraphNodeId::new("Other").unwrap());
    assert!(script_document_command::apply_generated_script_block(&mut conn, &bad, 4).is_err());
    script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
    bible_graph_command::apply_set_bible_graph_snapshot_field(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("Mara.future").unwrap(),
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
            at_ms: 100,
            label: "Future".into(),
            snapshot_sort_order: 0,
            field_id: BibleGraphSnapshotFieldId::new("Mara.future.motivation").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            field_key: BibleGraphFieldKey::new("motivation").unwrap(),
            value: Some(FieldValue::Text("Timed fact".into())),
            field_sort_order: 0,
        }),
        5,
    )
    .unwrap();
    field(
        &mut conn,
        "Mara",
        "motivation",
        Some("Baseline of timed field"),
    );
    assert!(membership(&conn, &b).is_none());
}

#[test]
fn accepting_all_fields_cleared_retains_proven_entity_scope_for_restored_facts() {
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue"));
    let command = generation(&conn, &b, true);
    script_document_command::apply_generated_script_block(&mut conn, &command, 4).unwrap();
    field(&mut conn, "Mara", "tagline", None);
    let req = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    assert!(binding.bible_inputs.is_empty());
    assert_eq!(
        binding.bible_context_scope.as_ref().unwrap().node_ids.len(),
        1
    );
    script_impact_review::record_proposal(
        &mut conn,
        &req,
        binding,
        "Explicit accepted human-reviewed replacement".into(),
        5,
    )
    .unwrap();
    accept(&mut conn, &req).unwrap();
    assert!(!impact(&conn, &b).needs_review);
    field(&mut conn, "Mara", "tagline", Some("Restored fact"));
    assert!(membership(&conn, &b).is_some());
    assert_eq!(
        text(&conn, &b),
        "Explicit accepted human-reviewed replacement"
    );
}

#[test]
fn only_existing_outputs_with_proven_entity_scope_acquire_membership_review() {
    let (mut conn, _, _, b, c) = fixture();
    entity(&mut conn, "Other");
    field(&mut conn, "Other", "tagline", Some("Other fact"));
    let c_command = generation(&conn, &c, true);
    script_document_command::apply_generated_script_block(&mut conn, &c_command, 4).unwrap();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Mara blue"));
    let b_command = generation(&conn, &b, true);
    script_document_command::apply_generated_script_block(&mut conn, &b_command, 5).unwrap();
    edit(&mut conn, &b, "Manual B unchanged");
    edit(&mut conn, &c, "Manual C unchanged");
    entity(&mut conn, "Arbitrary");
    field(
        &mut conn,
        "Arbitrary",
        "tagline",
        Some("New unrelated fact"),
    );
    assert!(!impact(&conn, &b).needs_review);
    assert!(!impact(&conn, &c).needs_review);
    field(
        &mut conn,
        "Mara",
        "motivation",
        Some("New relevant motivation"),
    );
    assert!(membership(&conn, &b).is_some());
    assert!(!impact(&conn, &c).needs_review);
    assert_eq!(text(&conn, &b), "Manual B unchanged");
    assert_eq!(text(&conn, &c), "Manual C unchanged");
}

#[test]
fn retained_entity_membership_is_canonical_without_fabricating_unprovided_value_consumption() {
    let (mut conn, _, _, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue"));
    let first = generation(&conn, &b, true);
    script_document_command::apply_generated_script_block(&mut conn, &first, 4).unwrap();

    // The resolver can omit a previously relevant entity at its node limit.
    // Its membership watch remains canonical; actual supplied values stay empty.
    let scope = capture(&conn, node(&b), &[]).unwrap();
    assert_eq!(scope.node_ids, vec![BibleGraphNodeId::new("Mara").unwrap()]);
    assert_eq!(
        scope.field_ids,
        vec![BibleGraphFieldId::new("Mara.tagline").unwrap()]
    );
    let mut next = generation(&conn, &b, true);
    next.payload.bible_inputs = Some(vec![]);
    next.payload.bible_context_scope = Some(scope);
    let mut incomplete = next.clone();
    incomplete.id = CommandId(uuid::Uuid::new_v4());
    let incomplete_scope = incomplete.payload.bible_context_scope.as_mut().unwrap();
    incomplete_scope.node_ids.clear();
    incomplete_scope.field_ids.clear();
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &incomplete, 5).is_err()
    );
    script_document_command::apply_generated_script_block(&mut conn, &next, 5).unwrap();
    edit(&mut conn, &b, "Exact retained manual B");
    assert!(!impact(&conn, &b).needs_review);
    let uses_fact: u64 = conn.query_row(
        "SELECT COUNT(*) FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.source_id=?1 AND d.dependency_kind='uses_fact' AND r.source_revision_event_id=(SELECT event_id FROM script_generations WHERE segment_id=?1 ORDER BY rowid DESC LIMIT 1)",
        [b.segment_id.as_str()], |row| row.get(0),
    ).unwrap();
    assert_eq!(
        uses_fact, 0,
        "Unprovided values never gain consumption bindings"
    );
    field(
        &mut conn,
        "Mara",
        "tagline",
        Some("Different unprovided value"),
    );
    assert!(!impact(&conn, &b).needs_review);
    field(&mut conn, "Mara", "tagline", None);
    assert!(
        membership(&conn, &b)
            .unwrap()
            .input_excerpt
            .unwrap()
            .contains("removed: Mara.profile.tagline")
    );
    assert_eq!(text(&conn, &b), "Exact retained manual B");
}
