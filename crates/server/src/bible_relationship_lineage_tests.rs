use super::*;
use crate::script_impact_review::tests::{accept, fixture, request, text};
use crate::{bible_graph_command, script_document_command, script_impact_review};

fn node(conn: &mut Connection, id: &str, order: u32) {
    bible_graph_command::apply_create_bible_graph_node(
        conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new(id).unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: id.into(),
            sort_order: order,
        }),
        1,
    )
    .unwrap();
}

fn set(conn: &mut Connection, edge: &SetBibleGraphEdgeCommand) {
    bible_graph_command::apply_set_bible_graph_edge(conn, &CommandEnvelope::new(edge.clone()), 2)
        .unwrap();
}

fn setup(conn: &mut Connection) -> SetBibleGraphEdgeCommand {
    node(conn, "Mara", 0);
    node(conn, "Eli", 1);
    node(conn, "Noor", 2);
    let edge = SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
        from_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        to_node_id: BibleGraphNodeId::new("Eli").unwrap(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "Mara trusts Eli".into(),
        directed: true,
        sort_order: 0,
    };
    set(conn, &edge);
    edge
}

fn captured(conn: &Connection, b: &SetScriptBlockCommand) -> Vec<BibleRelationshipInput> {
    let context = crate::ai_context_projection::load_ai_bible_context_projection(
        conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        None,
    )
    .unwrap();
    capture(conn, &context.payload).unwrap()
}

fn generate(
    conn: &mut Connection,
    b: &SetScriptBlockCommand,
    relationships: Option<Vec<BibleRelationshipInput>>,
) -> CommandEnvelope<GenerateScriptBlockCommand> {
    let command = CommandEnvelope::new(GenerateScriptBlockCommand {
        arc_description_applicability: None,
        timeline_title_inputs: None,
        ancestor_notes_inputs: None,
        arc_inputs: None,
        block: b.clone(),
        script_inputs: Some(vec![]),
        bible_inputs: Some(vec![]),
        bible_node_name_inputs: None,
        bible_relationship_inputs: relationships,
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
        "Synthetic reviewed screenplay: Mara doubts Eli.".into(),
        4,
    )
    .unwrap();
    command
}

fn rows(conn: &Connection) -> std::collections::BTreeMap<String, Vec<Vec<rusqlite::types::Value>>> {
    let tables = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap().query_map([], |row| row.get::<_, String>(0)).unwrap()
        .collect::<Result<Vec<_>, _>>().unwrap();
    tables
        .into_iter()
        .map(|table| {
            let mut statement = conn
                .prepare(&format!(
                    "SELECT * FROM \"{}\" ORDER BY rowid",
                    table.replace('"', "\"\"")
                ))
                .unwrap();
            let count = statement.column_count();
            let values = statement
                .query_map([], |row| (0..count).map(|i| row.get(i)).collect())
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, values)
        })
        .collect()
}

#[test]
fn consumed_relationship_edit_reviews_only_generated_material_and_explicit_acceptance_refreshes_revision()
 {
    let (mut conn, _, a, b, c) = fixture();
    let mut edge = setup(&mut conn);
    let inputs = captured(&conn, &b);
    assert_eq!(inputs.len(), 1, "incoming/outgoing appearances deduplicate");
    let original = inputs[0].clone();
    generate(&mut conn, &b, Some(inputs));
    let original_generation = impact(&conn, &b).generation_event_id;
    assert!(!impact(&conn, &b).needs_review);
    let manual = (text(&conn, &a), text(&conn, &c));
    edge.label = "Mara doubts Eli".into();
    set(&mut conn, &edge);
    let review = impact(&conn, &b);
    assert!(review.needs_review);
    assert_eq!(review.causes.len(), 1);
    let cause = &review.causes[0];
    assert_eq!(cause.input, endpoint(&original));
    assert_eq!(cause.consumed_revision_event_id, original.revision_event_id);
    assert_eq!(cause.input_excerpt.as_deref(), Some("Mara trusts Eli"));
    assert!(
        crate::script_impact_projection::load_impact(&conn, &a.segment_id)
            .unwrap()
            .is_none()
    );
    assert!(
        crate::script_impact_projection::load_impact(&conn, &c.segment_id)
            .unwrap()
            .is_none()
    );
    let command = preview(&mut conn, &b);
    assert_eq!(text(&conn, &b), b.text);
    let receipt =
        crate::propagation_proposal_store::load_propagation_proposal_list_projection(&conn)
            .unwrap();
    let binding = receipt.payload.proposals[0]
        .script_review_binding
        .as_ref()
        .unwrap();
    assert_eq!(
        binding.bible_relationship_inputs.as_ref().unwrap()[0]
            .edge
            .label,
        edge.label
    );
    let acceptance = CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: command.payload.proposal_id.clone(),
    });
    crate::propagation_proposal_accept::record_accept_propagation_proposal(
        &mut conn,
        &acceptance,
        40,
    )
    .unwrap();
    let after_acceptance = rows(&conn);
    assert_eq!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &acceptance,
            41
        )
        .unwrap(),
        crate::history_store::RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(rows(&conn), after_acceptance);
    assert!(!impact(&conn, &b).needs_review);
    assert_ne!(impact(&conn, &b).generation_event_id, original_generation);
    assert_eq!(
        text(&conn, &b),
        "Synthetic reviewed screenplay: Mara doubts Eli."
    );
    assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
    assert_eq!(
        crate::bible_graph_edge_store::load_edge(&conn, &edge.edge_id)
            .unwrap()
            .unwrap(),
        edge.into_edge()
    );
}

#[test]
fn label_kind_direction_endpoint_and_delete_recreate_aba_refuse_old_preview_without_writes() {
    for mutation in ["label", "kind", "direction", "endpoint", "delete"] {
        let (mut conn, _, a, b, c) = fixture();
        let mut edge = setup(&mut conn);
        let inputs = captured(&conn, &b);
        generate(&mut conn, &b, Some(inputs));
        edge.label = "Mara doubts Eli".into();
        set(&mut conn, &edge);
        let command = preview(&mut conn, &b);
        let original = edge.clone();
        match mutation {
            "label" => edge.label = "Mara distrusts Eli".into(),
            "kind" => edge.edge_kind = BibleGraphEdgeKind::LocatedIn,
            "direction" => edge.directed = false,
            "endpoint" => edge.to_node_id = BibleGraphNodeId::new("Noor").unwrap(),
            "delete" => {
                bible_graph_command::apply_delete_bible_graph_edge(
                    &mut conn,
                    &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                        edge_id: edge.edge_id.clone(),
                    }),
                    5,
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        if mutation != "delete" {
            set(&mut conn, &edge);
        }
        set(&mut conn, &original);
        let before = rows(&conn);
        assert!(
            accept(&mut conn, &command)
                .unwrap_err()
                .to_string()
                .contains("stale"),
            "{mutation}"
        );
        assert_eq!(
            rows(&conn),
            before,
            "{mutation}: refusal must roll back every row"
        );
        assert_eq!(text(&conn, &b), b.text);
        assert_eq!(text(&conn, &a), "  A now carries a blue umbrella — 雨\n\n");
        assert_eq!(text(&conn, &c), c.text);
    }
}

#[test]
fn deleted_relationship_preview_refuses_absence_aba_without_writes() {
    let (mut conn, _, a, b, c) = fixture();
    let edge = setup(&mut conn);
    let manual = (text(&conn, &a), text(&conn, &c));
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs));
    let delete = |conn: &mut Connection| {
        bible_graph_command::apply_delete_bible_graph_edge(
            conn,
            &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                edge_id: edge.edge_id.clone(),
            }),
            5,
        )
        .unwrap();
    };
    delete(&mut conn);
    let command = preview(&mut conn, &b);
    let captured = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert!(
        captured
            .bible_relationship_inputs
            .as_ref()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        captured
            .bible_relationship_absence_revisions
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    set(&mut conn, &edge);
    delete(&mut conn);
    let before = rows(&conn);
    assert!(
        accept(&mut conn, &command).is_err(),
        "An absent edge is a revision-bound read, not a timeless None"
    );
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &b), b.text);
    assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
    let fresh = preview(&mut conn, &b);
    accept(&mut conn, &fresh).unwrap();
    assert!(!impact(&conn, &b).needs_review);
    assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
}

#[test]
fn absent_consumed_relationship_is_guarded_when_preview_selects_another_cause() {
    let (mut conn, _, a, b, c) = fixture();
    let edge = setup(&mut conn);
    let manual = (text(&conn, &a), text(&conn, &c));
    let mut other = edge.clone();
    other.edge_id = BibleGraphEdgeId::new("Mara.Noor").unwrap();
    other.to_node_id = BibleGraphNodeId::new("Noor").unwrap();
    set(&mut conn, &other);
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs));
    let delete = |conn: &mut Connection| {
        bible_graph_command::apply_delete_bible_graph_edge(
            conn,
            &CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                edge_id: edge.edge_id.clone(),
            }),
            5,
        )
        .unwrap();
    };
    delete(&mut conn);
    other.label = "Mara doubts Noor".into();
    set(&mut conn, &other);
    let mut command = request(&conn, &b);
    command.payload.dependency_id = impact(&conn, &b)
        .causes
        .iter()
        .find(|cause| {
            cause.input
                == SemanticDependencyEndpoint::BibleEdge {
                    edge_id: other.edge_id.clone(),
                }
        })
        .unwrap()
        .dependency_id
        .clone();
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    assert_eq!(
        binding
            .bible_relationship_absence_revisions
            .as_ref()
            .unwrap()[0]
            .0,
        edge.edge_id
    );
    script_impact_review::record_proposal(
        &mut conn,
        &command,
        binding.clone(),
        "Synthetic review of two inputs".into(),
        4,
    )
    .unwrap();
    let mut legacy = binding;
    legacy.bible_relationship_absence_revisions = None;
    assert!(script_impact_review::validate_binding(&conn, &legacy).is_err());
    set(&mut conn, &edge);
    delete(&mut conn);
    let before = rows(&conn);
    assert!(accept(&mut conn, &command).is_err());
    assert_eq!(rows(&conn), before);
    assert_eq!(text(&conn, &b), b.text);
    assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
}

#[test]
fn off_scope_relationship_edits_do_not_create_review_or_invalidate_current_preview() {
    let (mut conn, _, _, b, _) = fixture();
    let mut edge = setup(&mut conn);
    for index in 3..205 {
        node(&mut conn, &format!("Filler{index:03}"), index);
    }
    node(&mut conn, "OutsideA", 900);
    node(&mut conn, "OutsideB", 901);
    let mut outside = edge.clone();
    outside.edge_id = BibleGraphEdgeId::new("outside.edge").unwrap();
    outside.from_node_id = BibleGraphNodeId::new("OutsideA").unwrap();
    outside.to_node_id = BibleGraphNodeId::new("OutsideB").unwrap();
    outside.label = "Outside initial".into();
    set(&mut conn, &outside);
    let inputs = captured(&conn, &b);
    assert_eq!(inputs.len(), 1);
    generate(&mut conn, &b, Some(inputs));
    outside.label = "Outside edited".into();
    set(&mut conn, &outside);
    assert!(!impact(&conn, &b).needs_review);
    edge.label = "Mara doubts Eli".into();
    set(&mut conn, &edge);
    let command = preview(&mut conn, &b);
    outside.label = "Outside edited again".into();
    set(&mut conn, &outside);
    accept(&mut conn, &command).unwrap();
    assert!(!impact(&conn, &b).needs_review);
}

#[test]
fn late_generation_binds_original_read_and_false_or_duplicate_relationship_receipts_roll_back() {
    let (mut conn, _, _, b, _) = fixture();
    let mut edge = setup(&mut conn);
    let inputs = captured(&conn, &b);
    edge.label = "Mara doubts Eli".into();
    set(&mut conn, &edge);
    let generated = generate(&mut conn, &b, Some(inputs.clone()));
    assert!(impact(&conn, &b).needs_review);
    assert_eq!(
        impact(&conn, &b).causes[0].input_excerpt.as_deref(),
        Some("Mara trusts Eli")
    );
    for corruption in [
        "endpoint",
        "kind",
        "direction",
        "label",
        "revision",
        "duplicate",
    ] {
        let mut command = generated.clone();
        command.id = CommandId::new();
        let mut wrong = inputs.clone();
        match corruption {
            "endpoint" => wrong[0].edge.to_node_id = BibleGraphNodeId::new("Noor").unwrap(),
            "kind" => wrong[0].edge.edge_kind = BibleGraphEdgeKind::LocatedIn,
            "direction" => wrong[0].edge.directed = false,
            "label" => wrong[0].edge.label = edge.label.clone(),
            "revision" => {
                wrong[0].revision_event_id = current_revision(&conn, &endpoint(&wrong[0]))
                    .unwrap()
                    .unwrap()
            }
            "duplicate" => wrong.push(wrong[0].clone()),
            _ => unreachable!(),
        }
        command.payload.bible_relationship_inputs = Some(wrong);
        let before = rows(&conn);
        assert!(
            script_document_command::apply_generated_script_block(&mut conn, &command, 6).is_err(),
            "{corruption}"
        );
        assert_eq!(rows(&conn), before, "{corruption}");
    }
}

#[test]
fn delayed_preview_and_absent_legacy_receipt_require_fresh_review_without_writes() {
    let (mut conn, _, _, b, _) = fixture();
    let mut edge = setup(&mut conn);
    let inputs = captured(&conn, &b);
    generate(&mut conn, &b, Some(inputs));
    edge.label = "Mara doubts Eli".into();
    set(&mut conn, &edge);
    crate::propagation_proposal_store::create_schema(&conn).unwrap();
    let command = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
    let mut missing = binding.clone();
    missing.bible_relationship_inputs = None;
    assert!(script_impact_review::validate_binding(&conn, &missing).is_err());
    edge.label = "Mara distrusts Eli".into();
    set(&mut conn, &edge);
    let before = rows(&conn);
    assert!(
        script_impact_review::record_proposal(
            &mut conn,
            &command,
            binding,
            "Synthetic late preview".into(),
            7
        )
        .is_err()
    );
    assert_eq!(rows(&conn), before);
    let legacy = generate(&mut conn, &b, None);
    let json = serde_json::to_value(&legacy.payload).unwrap();
    assert!(json.get("bible_relationship_inputs").is_none());
    edge.label = "Mara trusts Eli again".into();
    set(&mut conn, &edge);
    assert!(
        !impact(&conn, &b).needs_review,
        "legacy unknown must not fabricate consumption"
    );
}

#[test]
fn writer_transaction_rechecks_relationship_before_recording_preview_or_accepting_saved_text() {
    for action in ["script.impact_proposal", "semantic.propagation_accept"] {
        let (mut conn, _, _, b, _) = fixture();
        let mut edge = setup(&mut conn);
        let inputs = captured(&conn, &b);
        generate(&mut conn, &b, Some(inputs));
        edge.label = "Mara doubts Eli".into();
        set(&mut conn, &edge);
        crate::propagation_proposal_store::create_schema(&conn).unwrap();
        let command = if action == "semantic.propagation_accept" {
            preview(&mut conn, &b)
        } else {
            request(&conn, &b)
        };
        let binding = script_impact_review::capture(&conn, &command.payload).unwrap();
        // Mutation occurs after outside validation, inside the writer's existing
        // command insertion. This deterministic fault proves its second check.
        conn.execute_batch(&format!("CREATE TRIGGER late_edge AFTER INSERT ON commands WHEN NEW.payload_type = '{action}' BEGIN UPDATE bible_graph_edges SET label = 'Writer race' WHERE id = 'Mara.Eli'; END;")).unwrap();
        let before = rows(&conn);
        let error = if action == "semantic.propagation_accept" {
            accept(&mut conn, &command).unwrap_err().to_string()
        } else {
            script_impact_review::record_proposal(
                &mut conn,
                &command,
                binding,
                "Synthetic preview".into(),
                8,
            )
            .unwrap_err()
            .to_string()
        };
        assert!(
            error.contains("stale")
                || error
                    .contains("Bible relationship input does not match canonical revision history"),
            "{action}: {error}"
        );
        assert_eq!(
            rows(&conn),
            before,
            "{action}: all writer and injected rows must roll back"
        );
    }
}

#[test]
fn sparse_relationship_history_aba_is_not_hidden_by_unchanged_graph_storage() {
    for key in ["label", "to_node_id", "edge_kind", "directed"] {
        let (mut conn, _, _, b, _) = fixture();
        let mut edge = setup(&mut conn);
        let inputs = captured(&conn, &b);
        generate(&mut conn, &b, Some(inputs));
        edge.label = "Mara doubts Eli".into();
        set(&mut conn, &edge);
        let command = preview(&mut conn, &b);
        let (changed, restored) = match key {
            "label" => (
                FieldValue::Text("Sparse label".into()),
                FieldValue::Text(edge.label.clone()),
            ),
            "to_node_id" => (
                FieldValue::ObjectRef {
                    kind: ObjectKind::BibleNode,
                    id: "Noor".into(),
                },
                FieldValue::ObjectRef {
                    kind: ObjectKind::BibleNode,
                    id: "Eli".into(),
                },
            ),
            "edge_kind" => (
                FieldValue::Text("LocatedIn".into()),
                FieldValue::Text("References".into()),
            ),
            "directed" => (FieldValue::Bool(false), FieldValue::Bool(true)),
            _ => unreachable!(),
        };
        for value in [changed, restored] {
            crate::object_field_command::apply_set_object_field(
                &mut conn,
                &CommandEnvelope::new(SetObjectFieldCommand {
                    object_kind: ObjectKind::BibleEdge,
                    object_id: edge.edge_id.as_str().into(),
                    field_key: key.into(),
                    value: Some(value),
                }),
                9,
            )
            .unwrap();
        }
        assert_eq!(
            crate::bible_graph_edge_store::load_edge(&conn, &edge.edge_id)
                .unwrap()
                .unwrap(),
            edge.clone().into_edge()
        );
        let before = rows(&conn);
        assert!(
            accept(&mut conn, &command)
                .unwrap_err()
                .to_string()
                .contains("stale"),
            "{key}"
        );
        assert_eq!(rows(&conn), before);
        let fresh = captured(&conn, &b);
        validate_history(&conn, &fresh[0]).unwrap();
    }
}

#[test]
fn unrelated_event_cannot_be_forged_as_a_consumed_relationship_revision() {
    let (mut conn, _, _, b, _) = fixture();
    setup(&mut conn);
    let mut input = captured(&conn, &b).remove(0);
    node(&mut conn, "Later", 10);
    let event: String = conn
        .query_row(
            "SELECT change_event_id FROM object_revisions WHERE object_kind='bible_node' AND object_id='Later' ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    input.revision_event_id = ChangeEventId(uuid::Uuid::parse_str(&event).unwrap());
    assert!(validate_history(&conn, &input).is_err());
}

#[test]
fn consumed_live_relationship_outside_bounded_context_refuses_other_cause_preview() {
    let (mut conn, _, a, b, c) = fixture();
    node(&mut conn, "Mara", 0);
    node(&mut conn, "Eli", 900);
    node(&mut conn, "Noor", 1);
    let mut edge = SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
        from_node_id: BibleGraphNodeId::new("Mara").unwrap(),
        to_node_id: BibleGraphNodeId::new("Eli").unwrap(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "Mara trusts Eli".into(),
        directed: true,
        sort_order: 0,
    };
    set(&mut conn, &edge);
    let mut other = edge.clone();
    other.edge_id = BibleGraphEdgeId::new("Mara.Noor").unwrap();
    other.to_node_id = BibleGraphNodeId::new("Noor").unwrap();
    set(&mut conn, &other);
    let inputs = captured(&conn, &b);
    assert_eq!(inputs.len(), 2);
    generate(&mut conn, &b, Some(inputs));
    other.label = "Mara doubts Noor".into();
    set(&mut conn, &other);
    let pending = preview(&mut conn, &b);
    for index in 2..205 {
        node(&mut conn, &format!("Filler{index:03}"), index);
    }
    assert_eq!(captured(&conn, &b).len(), 1);
    let mut command = request(&conn, &b);
    command.payload.dependency_id = impact(&conn, &b)
        .causes
        .iter()
        .find(|cause| {
            cause.input
                == SemanticDependencyEndpoint::BibleEdge {
                    edge_id: other.edge_id.clone(),
                }
        })
        .unwrap()
        .dependency_id
        .clone();
    let manual = (text(&conn, &a), text(&conn, &c));
    for changed in [false, true] {
        if changed {
            edge.label = "Mara doubts Eli".into();
            set(&mut conn, &edge);
        }
        let before = rows(&conn);
        let error = script_impact_review::capture(&conn, &command.payload).unwrap_err();
        assert!(
            error.to_string().contains("outside the current context"),
            "{error}"
        );
        assert_eq!(rows(&conn), before);
        assert!(accept(&mut conn, &pending).is_err());
        assert_eq!(rows(&conn), before);
        assert_eq!(text(&conn, &b), b.text);
        assert_eq!((text(&conn, &a), text(&conn, &c)), manual);
    }
}
