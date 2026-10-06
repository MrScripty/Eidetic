use super::*;
use crate::script_impact_review::tests::{accept, fixture, request, text};
use crate::{bible_graph_command, script_document_command, script_impact_review};

fn node(conn: &mut Connection, id: &str) {
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
fn rename(conn: &mut Connection, id: &str, name: &str) {
    bible_graph_command::apply_set_bible_graph_node_name(
        conn,
        &CommandEnvelope::new(SetBibleGraphNodeNameCommand {
            node_id: BibleGraphNodeId::new(id).unwrap(),
            name: name.into(),
        }),
        2,
    )
    .unwrap();
}
fn captured(conn: &Connection, block: &SetScriptBlockCommand) -> Vec<BibleNodeNameInput> {
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
    b: &SetScriptBlockCommand,
    names: Option<Vec<BibleNodeNameInput>>,
) {
    script_document_command::apply_generated_script_block(
        conn,
        &CommandEnvelope::new(GenerateScriptBlockCommand {
            block: b.clone(),
            script_inputs: Some(vec![]),
            bible_inputs: Some(vec![]),
            bible_relationship_inputs: None,
            bible_node_name_inputs: names,
            bible_context_scope: None,
            script_context_scope: None,
            target_binding: None,
        }),
        3,
    )
    .unwrap();
}
fn impact(conn: &Connection, b: &SetScriptBlockCommand) -> ScriptImpactProjection {
    crate::script_impact_projection::load_impact(conn, &b.segment_id)
        .unwrap()
        .unwrap()
}
fn preview(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptImpactProposalCommand> {
    let command = request(conn, b);
    let binding = script_impact_review::capture(conn, &command.payload).unwrap();
    script_impact_review::record_proposal(
        conn,
        &command,
        binding,
        "Synthetic preview: Marisol boards the train.".into(),
        4,
    )
    .unwrap();
    command
}
fn rows(conn: &Connection) -> Vec<(String, Vec<Vec<rusqlite::types::Value>>)> {
    let tables = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap()
        .query_map([], |row| row.get::<_, String>(0)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT * FROM \"{}\" ORDER BY rowid",
                    table.replace('"', "\"\"")
                ))
                .unwrap();
            let count = stmt.column_count();
            let values = stmt
                .query_map([], |row| (0..count).map(|i| row.get(i)).collect())
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, values)
        })
        .collect()
}

#[test]
fn consumed_name_rename_reviews_only_affected_scene_and_acceptance_refreshes_receipts() {
    let (mut conn, _, a, b, c) = fixture();
    node(&mut conn, "Mara");
    let original = captured(&conn, &b);
    assert_eq!(original.len(), 1);
    generate(&mut conn, &b, Some(original.clone()));
    let manual = (text(&conn, &a), text(&conn, &c));
    rename(&mut conn, "Mara", "Marisol");
    let review = impact(&conn, &b);
    assert_eq!(review.causes.len(), 1);
    assert_eq!(review.causes[0].input, endpoint(&original[0]));
    assert_eq!(review.causes[0].input_excerpt.as_deref(), Some("Mara"));
    assert_eq!(
        review.causes[0].consumed_revision_event_id,
        original[0].revision_event_id
    );
    assert!(review.needs_review);
    let command = preview(&mut conn, &b);
    assert_eq!(text(&conn, &b), b.text);
    let proposals =
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap();
    let binding = proposals.payload.proposals[0]
        .script_review_binding
        .as_ref()
        .unwrap();
    assert_eq!(
        binding.bible_node_name_inputs.as_ref().unwrap()[0].name,
        "Marisol"
    );
    accept(&mut conn, &command).unwrap();
    assert!(!impact(&conn, &b).needs_review);
    assert_eq!(
        text(&conn, &b),
        "Synthetic preview: Marisol boards the train."
    );
    assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
    rename(&mut conn, "Mara", "Mara");
    assert!(impact(&conn, &b).needs_review);
}

#[test]
fn late_generation_keeps_the_supplied_name_revision_including_aba() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    let inputs = captured(&conn, &b);
    rename(&mut conn, "Mara", "Marisol");
    rename(&mut conn, "Mara", "Mara");
    generate(&mut conn, &b, Some(inputs.clone()));
    let review = impact(&conn, &b);
    assert!(review.needs_review);
    assert_eq!(
        review.causes[0].consumed_revision_event_id,
        inputs[0].revision_event_id
    );
    assert_ne!(
        review.causes[0].current_revision_event_id,
        Some(inputs[0].revision_event_id)
    );
}

#[test]
fn name_aba_after_preview_refuses_acceptance_without_any_database_writes() {
    let (mut conn, _, a, b, c) = fixture();
    node(&mut conn, "Mara");
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs));
    rename(&mut conn, "Mara", "Marisol");
    let command = preview(&mut conn, &b);
    rename(&mut conn, "Mara", "Mara");
    rename(&mut conn, "Mara", "Marisol");
    let before = rows(&conn);
    assert!(
        accept(&mut conn, &command)
            .unwrap_err()
            .to_string()
            .contains("stale")
    );
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &b), b.text);
    assert_eq!(text(&conn, &a), "  A now carries a blue umbrella — 雨\n\n");
    assert_eq!(text(&conn, &c), c.text);
}

#[test]
fn unrelated_name_and_metadata_edits_do_not_change_consumed_name_clock() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs.clone()));
    node(&mut conn, "Other");
    rename(&mut conn, "Other", "Unrelated");
    let event = ChangeEvent::new(CommandId::new(), ChangeEventKind::UserEdit, "metadata");
    let revision = ObjectRevision::new(
        ObjectKind::BibleNode,
        "Mara",
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "sort_order",
        Some(FieldValue::Integer(0)),
        Some(FieldValue::Integer(2)),
    ));
    let command = CommandEnvelope {
        id: event.command_id,
        payload: "qualification sparse revision",
    };
    crate::history_store::record_change(
        &mut conn,
        &command,
        "test.sparse_name",
        &event,
        &[revision],
    )
    .unwrap();
    assert_eq!(
        current_revision(&conn, &endpoint(&inputs[0])).unwrap(),
        Some(inputs[0].revision_event_id)
    );
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn deleted_name_keeps_historical_explanation_and_absence_custody() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs));
    bible_graph_command::apply_delete_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(DeleteBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
        }),
        5,
    )
    .unwrap();
    let review = impact(&conn, &b);
    assert_eq!(review.causes[0].reason, ScriptImpactReason::Deleted);
    assert_eq!(review.causes[0].input_excerpt.as_deref(), Some("Mara"));
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert!(binding.bible_node_name_inputs.as_ref().unwrap().is_empty());
    assert_eq!(
        binding
            .bible_node_name_absence_revisions
            .as_ref()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn all_live_consumed_names_must_remain_in_preview_even_when_another_cause_is_selected() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    node(&mut conn, "Eli");
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs.clone()));
    rename(&mut conn, "Mara", "Marisol");
    let review = impact(&conn, &b);
    let only_mara = captured(&conn, &b)
        .into_iter()
        .filter(|input| input.node_id.as_str() == "Mara")
        .collect::<Vec<_>>();
    assert!(
        validate_preview_inputs(&conn, review.generation_event_id, &b.segment_id, &only_mara)
            .unwrap_err()
            .to_string()
            .contains("outside the current context")
    );
    assert_eq!(text(&conn, &b), b.text);
}

#[test]
fn forged_ownership_and_sparse_live_history_divergence_refuse_name_capture() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    node(&mut conn, "Eli");
    let inputs = captured(&conn, &b);
    let mut forged = inputs
        .iter()
        .find(|i| i.node_id.as_str() == "Mara")
        .unwrap()
        .clone();
    forged.revision_event_id = inputs
        .iter()
        .find(|i| i.node_id.as_str() == "Eli")
        .unwrap()
        .revision_event_id;
    assert!(validate_history(&conn, &forged).is_err());
    let event = ChangeEvent::new(CommandId::new(), ChangeEventKind::UserEdit, "sparse name");
    let revision = ObjectRevision::new(
        ObjectKind::BibleNode,
        "Mara",
        event.id,
        RevisionOperation::Update,
    )
    .with_field(FieldDelta::new(
        "name",
        Some(FieldValue::Text("Mara".into())),
        Some(FieldValue::Text("Different history".into())),
    ));
    let command = CommandEnvelope {
        id: event.command_id,
        payload: "qualification sparse revision",
    };
    crate::history_store::record_change(
        &mut conn,
        &command,
        "test.sparse_name",
        &event,
        &[revision],
    )
    .unwrap();
    let context = crate::ai_context_projection::load_ai_bible_context_projection(
        &conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        None,
    )
    .unwrap();
    assert!(capture(&conn, &context.payload).is_err());
}

#[test]
fn legacy_generation_has_no_fabricated_name_dependency() {
    let (mut conn, _, _, b, _) = fixture();
    node(&mut conn, "Mara");
    generate(&mut conn, &b, None);
    rename(&mut conn, "Mara", "Marisol");
    assert!(!impact(&conn, &b).needs_review);
}
