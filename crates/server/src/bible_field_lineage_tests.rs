use super::*;
use crate::script_impact_review::tests::{accept, edit, fixture, request, text};
use crate::{bible_graph_command, script_document_command, script_impact_review, script_store};

const RED: &str = "Mara carries a red umbrella.";
const BLUE: &str = "Mara carries a blue umbrella — 雨.\nKeep the train waiting.";

fn fact(conn: &mut Connection, id: &str) -> SetBibleGraphFieldCommand {
    let node_id = BibleGraphNodeId::new(id).unwrap();
    bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: node_id.clone(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: id.into(),
            sort_order: 0,
        }),
        1,
    )
    .unwrap();
    let command = SetBibleGraphFieldCommand {
        node_id,
        part_id: BibleGraphPartId::new(format!("{id}.profile")).unwrap(),
        part_key: BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".into(),
        part_sort_order: 0,
        field_id: BibleGraphFieldId::new(format!("{id}.tagline")).unwrap(),
        field_key: BibleGraphFieldKey::new("tagline").unwrap(),
        value: Some(FieldValue::Text(RED.into())),
        field_sort_order: 0,
    };
    set(conn, &command, Some(RED));
    command
}

fn set(conn: &mut Connection, field: &SetBibleGraphFieldCommand, value: Option<&str>) {
    let mut field = field.clone();
    field.value = value.map(|value| FieldValue::Text(value.into()));
    bible_graph_command::apply_set_bible_graph_field(conn, &CommandEnvelope::new(field), 20)
        .unwrap();
}

fn capture_for(conn: &Connection, block: &SetScriptBlockCommand) -> Vec<BibleFieldInput> {
    let context = crate::ai_context_projection::load_ai_bible_context_projection(
        conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        None,
    )
    .unwrap();
    capture(conn, &context.payload).unwrap()
}

fn generate(
    conn: &mut Connection,
    block: &SetScriptBlockCommand,
    inputs: Vec<BibleFieldInput>,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        target_binding: None,
        script_context_scope: None,
        block: block.clone(),
        script_inputs: Some(vec![]),
        bible_inputs: Some(inputs),
        bible_context_scope: None,
    });
    script_document_command::apply_generated_script_block(conn, &command, 25).unwrap();
    command
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

#[tokio::test]
async fn manual_bible_fact_edit_previews_and_explicitly_accepts_only_target_with_refreshed_lineage()
{
    let (mut conn, _, a, b, c) = fixture();
    let field = fact(&mut conn, "Mara");
    let inputs = capture_for(&conn, &b);
    generate(&mut conn, &b, inputs);
    assert!(!impact(&conn, &b).needs_review);
    let before_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
        .unwrap()
        .unwrap()
        .version;
    set(&mut conn, &field, Some(BLUE));
    let after_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
        .unwrap()
        .unwrap()
        .version;
    assert!(
        after_version.0 > before_version.0,
        "Bible impact must advance the canonical projection version"
    );
    let cause = impact(&conn, &b).causes.remove(0);
    assert_eq!(cause.reason, ScriptImpactReason::Changed);
    assert_eq!(cause.input_excerpt.as_deref(), Some(RED));
    assert_eq!(text(&conn, &b), "Original B");
    // Human text written after generation remains the exact preview target.
    edit(&mut conn, &b, "  Manual B — keep this line.\n\n");
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let proposed =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(BLUE));
            assert!(prompt.user.contains("  Manual B — keep this line.\n\n"));
            Ok(Box::pin(futures::stream::iter([Ok(
                "  Manual B — keep this line.\nMara carries blue.\n\n".into(),
            )]))
                as eidetic_core::ai::backend::GenerateStream)
        })
        .await
        .unwrap();
    let before_a = text(&conn, &a);
    let before_c = text(&conn, &c);
    script_impact_review::record_proposal(&mut conn, &command, binding, proposed.clone(), 30)
        .unwrap();
    assert_eq!(text(&conn, &b), "  Manual B — keep this line.\n\n");
    accept(&mut conn, &command).unwrap();
    assert_eq!(text(&conn, &b), proposed);
    assert_eq!(text(&conn, &a), before_a);
    assert_eq!(text(&conn, &c), before_c);
    assert!(!impact(&conn, &b).needs_review);
    set(&mut conn, &field, Some("Mara carries green."));
    let impact = impact(&conn, &b);
    assert!(impact.needs_review);
    assert_eq!(impact.causes[0].input_excerpt.as_deref(), Some(BLUE));
}

#[test]
fn late_generation_uses_consumed_history_and_replay_does_not_rebind_or_overwrite_manual_text() {
    let (mut conn, _, _, b, c) = fixture();
    let field = fact(&mut conn, "Mara");
    let captured = capture_for(&conn, &b);
    let revision = captured[0].revision_event_id;
    set(&mut conn, &field, Some(BLUE));
    let command = generate(&mut conn, &b, captured);
    assert_eq!(
        impact(&conn, &b).causes[0].consumed_revision_event_id,
        revision
    );
    assert!(
        script_store::load_document_projection(&conn, &c.document_id)
            .unwrap()
            .unwrap()
            .segments
            .iter()
            .find(|segment| segment.segment.id == c.segment_id)
            .unwrap()
            .impact
            .is_none()
    );
    edit(&mut conn, &b, "Manual late edit");
    script_document_command::apply_generated_script_block(&mut conn, &command, 99).unwrap();
    assert_eq!(text(&conn, &b), "Manual late edit");
    let mut wrong = command.clone();
    wrong.payload.bible_inputs.as_mut().unwrap()[0].revision_event_id = impact(&conn, &b).causes[0]
        .current_revision_event_id
        .unwrap();
    assert!(script_document_command::apply_generated_script_block(&mut conn, &wrong, 100).is_err());
}

#[test]
fn stale_aba_preview_refuses_atomically_and_remains_rejectable() {
    let (mut conn, _, _, b, _) = fixture();
    let field = fact(&mut conn, "Mara");
    let inputs = capture_for(&conn, &b);
    generate(&mut conn, &b, inputs);
    set(&mut conn, &field, Some(BLUE));
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    script_impact_review::record_proposal(&mut conn, &command, binding, "Preview blue".into(), 30)
        .unwrap();
    set(&mut conn, &field, Some(RED));
    set(&mut conn, &field, Some(BLUE));
    let events: i64 = conn
        .query_row("SELECT count(*) FROM change_events", [], |row| row.get(0))
        .unwrap();
    assert!(
        accept(&mut conn, &command)
            .unwrap_err()
            .to_string()
            .contains("stale")
    );
    assert_eq!(text(&conn, &b), "Original B");
    assert_eq!(
        conn.query_row::<i64, _, _>("SELECT count(*) FROM change_events", [], |row| row.get(0))
            .unwrap(),
        events
    );
    crate::propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: command.payload.proposal_id,
            reason: None,
        }),
        40,
    )
    .unwrap();
    assert!(impact(&conn, &b).needs_review);
}

#[test]
fn deleted_fact_owner_and_cleared_value_keep_explainable_consumed_history() {
    for delete_owner in [false, true] {
        let (mut conn, _, _, b, _) = fixture();
        let field = fact(&mut conn, "Mara");
        let inputs = capture_for(&conn, &b);
        generate(&mut conn, &b, inputs);
        let before_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
            .unwrap()
            .unwrap()
            .version;
        if delete_owner {
            bible_graph_command::apply_delete_bible_graph_node(
                &mut conn,
                &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
                    node_id: field.node_id,
                }),
                30,
            )
            .unwrap();
        } else {
            set(&mut conn, &field, None);
        }
        let after_version = script_store::load_document_projection_envelope(&conn, &b.document_id)
            .unwrap()
            .unwrap()
            .version;
        assert!(after_version.0 > before_version.0);
        let cause = &impact(&conn, &b).causes[0];
        assert_eq!(cause.input_excerpt.as_deref(), Some(RED));
        assert_eq!(
            cause.reason,
            if delete_owner {
                ScriptImpactReason::Deleted
            } else {
                ScriptImpactReason::Changed
            }
        );
        let command = request(&conn, &b);
        let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
        assert!(binding.bible_inputs.is_empty());
        assert_eq!(text(&conn, &b), "Original B");
    }
}

#[test]
fn forged_consumed_field_value_rolls_back_output_history_and_dependencies() {
    let (mut conn, _, _, b, _) = fixture();
    fact(&mut conn, "Mara");
    let mut inputs = capture_for(&conn, &b);
    inputs[0].value = FieldValue::Text("Forged canon".into());
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        target_binding: None,
        script_context_scope: None,
        block: b.clone(),
        script_inputs: Some(vec![]),
        bible_inputs: Some(inputs),
        bible_context_scope: None,
    });
    assert!(
        script_document_command::apply_generated_script_block(&mut conn, &command, 30).is_err()
    );
    assert_eq!(text(&conn, &b), "Original B");
    assert_eq!(
        conn.query_row::<i64, _, _>(
            "SELECT count(*) FROM change_events WHERE command_id = ?1",
            [command.id.0.to_string()],
            |row| row.get(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn timed_values_are_never_rebound_to_unconsumed_baseline_fields() {
    let (mut conn, _, _, b, _) = fixture();
    let field = fact(&mut conn, "Mara");
    bible_graph_command::apply_set_bible_graph_snapshot_field(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("future").unwrap(),
            node_id: field.node_id,
            at_ms: 1000,
            label: "Future".into(),
            snapshot_sort_order: 0,
            field_id: BibleGraphSnapshotFieldId::new("future.tagline").unwrap(),
            part_key: field.part_key,
            part_name: field.part_name,
            field_key: field.field_key,
            value: Some(FieldValue::Text("Timed fact".into())),
            field_sort_order: 0,
        }),
        30,
    )
    .unwrap();
    let id = eidetic_core::timeline::node::NodeId(
        uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
    );
    for time in [None, Some(1000)] {
        let context =
            crate::ai_context_projection::load_ai_bible_context_projection(&conn, id, time)
                .unwrap();
        assert!(capture(&conn, &context.payload).unwrap().is_empty());
    }
    let before =
        crate::ai_context_projection::load_ai_bible_context_projection(&conn, id, Some(999))
            .unwrap();
    assert_eq!(
        capture(&conn, &before.payload).unwrap()[0].value,
        FieldValue::Text(RED.into())
    );
}
