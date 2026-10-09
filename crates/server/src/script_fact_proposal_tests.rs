//! Labelled synthetic suggestions; no real-model quality claim.
use super::*;
use crate::bible_field_lineage::tests::{capture_for, fact, set};
use crate::script_impact_review::tests::{edit, fixture as scenes};
use rusqlite::params;

pub(crate) const SAVED: &str = "  Mara carries a blue umbrella — 雨.\n\n  ";
pub(crate) const VALUE: &str = "Mara's umbrella is blue — 雨.\n\n";

#[test]
fn malformed_optional_edit_receipt_keeps_impact_and_refuses_capture() {
    for malformed in ["json", "block", "document", "text"] {
        let (conn, _, block, _, _, request) = fixture();
        let original = crate::script_store::load_document_projection(&conn, &block.document_id)
            .unwrap()
            .unwrap();
        let mut payload: serde_json::Value = conn
            .query_row(
                "SELECT c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id JOIN script_blocks b ON b.updated_event_id=e.id WHERE b.id=?1",
                [block.block_id.as_str()],
                |r| r.get::<_, String>(0),
            )
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();
        if malformed != "json" {
            if malformed == "text" {
                payload["text"] = serde_json::json!("Other saved text");
            } else {
                payload[format!("{malformed}_id")] = serde_json::json!("other");
            }
        }
        let serialized = if malformed == "json" {
            "{".into()
        } else {
            payload.to_string()
        };
        conn.execute(
            "UPDATE commands SET payload_json=?1 WHERE id=(SELECT e.command_id FROM change_events e JOIN script_blocks b ON b.updated_event_id=e.id WHERE b.id=?2)",
            params![serialized, block.block_id.as_str()],
        ).unwrap();
        let current = crate::script_store::load_document_projection(&conn, &block.document_id)
            .unwrap()
            .unwrap();
        let old = original
            .segments
            .iter()
            .find(|s| s.segment.id == block.segment_id)
            .unwrap()
            .impact
            .as_ref()
            .unwrap();
        let impact = current
            .segments
            .iter()
            .find(|s| s.segment.id == block.segment_id)
            .unwrap()
            .impact
            .as_ref()
            .unwrap();
        assert!(impact.fact_edit.is_none(), "{malformed}");
        assert_eq!(impact.generation_event_id, old.generation_event_id);
        assert_eq!(impact.output_block_id, old.output_block_id);
        assert_eq!(impact.lineage_available, old.lineage_available);
        assert_eq!(impact.causes, old.causes);
        assert_eq!(impact.needs_review, old.needs_review);
        assert!(crate::script_fact_evidence::capture(&conn, &request.payload).is_err());
    }
}

pub(crate) fn fixture() -> (
    Connection,
    eidetic_core::Project,
    SetScriptBlockCommand,
    SetBibleGraphFieldCommand,
    SetBibleGraphEdgeCommand,
    CommandEnvelope<RequestScriptFactProposalCommand>,
) {
    let (mut conn, project, a, b, _) = scenes();
    let field = fact(&mut conn, "Mara");
    fact(&mut conn, "Eli");
    let edge = SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
        from_node_id: field.node_id.clone(),
        to_node_id: BibleGraphNodeId::new("Eli").unwrap(),
        edge_kind: BibleGraphEdgeKind::References,
        label: "Mara trusts Eli".into(),
        directed: true,
        sort_order: 0,
    };
    crate::bible_graph_command::apply_set_bible_graph_edge(
        &mut conn,
        &CommandEnvelope::new(edge.clone()),
        22,
    )
    .unwrap();
    let inputs: Vec<_> = capture_for(&conn, &b)
        .into_iter()
        .filter(|f| f.node_id == field.node_id)
        .collect();
    let context = crate::ai_context_projection::load_ai_bible_context_projection(
        &conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(b.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        None,
    )
    .unwrap();
    let names = crate::bible_node_name_lineage::capture(&conn, &context.payload).unwrap();
    let relationships =
        crate::bible_relationship_lineage::capture(&conn, &context.payload).unwrap();
    for block in [&a, &b] {
        crate::script_document_command::apply_generated_script_block(
            &mut conn,
            &CommandEnvelope::new(GenerateScriptBlockCommand {
                arc_description_applicability: None,
                timeline_title_inputs: None,
                ancestor_notes_inputs: None,
                block: block.clone(),
                script_inputs: Some(vec![]),
                bible_inputs: Some(inputs.clone()),
                bible_node_name_inputs: Some(names.clone()),
                bible_relationship_inputs: Some(relationships.clone()),
                bible_context_scope: None,
                script_context_scope: None,
                target_binding: None,
                arc_inputs: None,
            }),
            23,
        )
        .unwrap();
    }
    edit(&mut conn, &b, SAVED);
    let command = request(&conn, &b);
    (conn, project, b, field, edge, command)
}

pub(crate) fn request(
    conn: &Connection,
    b: &SetScriptBlockCommand,
) -> CommandEnvelope<RequestScriptFactProposalCommand> {
    let doc = crate::script_store::load_document_projection(conn, &b.document_id)
        .unwrap()
        .unwrap();
    let evidence = doc
        .segments
        .iter()
        .find(|s| s.segment.id == b.segment_id)
        .unwrap()
        .impact
        .as_ref()
        .unwrap()
        .fact_edit
        .as_ref()
        .unwrap();
    let selected = &evidence.facts[0];
    CommandEnvelope::new(RequestScriptFactProposalCommand {
        proposal_id: PropagationProposalId::new(format!("fact.{}", uuid::Uuid::new_v4())).unwrap(),
        document_id: b.document_id.clone(),
        segment_id: b.segment_id.clone(),
        block_id: b.block_id.clone(),
        expected_block_revision_event_id: evidence.revision_event_id,
        expected_field_revision_event_id: selected.revision_event_id,
        generation_event_id: evidence.generation_event_id,
        dependency_id: selected.dependency_id.clone(),
    })
}
pub(crate) fn propose(
    conn: &mut Connection,
    command: &CommandEnvelope<RequestScriptFactProposalCommand>,
) {
    let binding = crate::script_fact_evidence::capture(conn, &command.payload).unwrap();
    record(
        conn,
        command,
        binding,
        FactSuggestion {
            value: VALUE.into(),
            rationale: "Synthetic fixture: the saved edit changes umbrella colour".into(),
        },
        30,
    )
    .unwrap();
}
fn decision(
    command: &CommandEnvelope<RequestScriptFactProposalCommand>,
) -> CommandEnvelope<AcceptPropagationProposalCommand> {
    CommandEnvelope::new(AcceptPropagationProposalCommand {
        proposal_id: command.payload.proposal_id.clone(),
    })
}
fn count(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM change_events", [], |r| r.get(0))
        .unwrap()
}
fn proposal(
    conn: &Connection,
    command: &CommandEnvelope<RequestScriptFactProposalCommand>,
) -> PropagationProposal {
    propagation_proposal_store::load_propagation_proposal(conn, &command.payload.proposal_id)
        .unwrap()
        .unwrap()
}

#[test]
fn exact_saved_edit_and_only_consumed_fact_are_eligible() {
    let (conn, _, b, field, _, command) = fixture();
    let binding = crate::script_fact_evidence::capture(&conn, &command.payload).unwrap();
    assert_eq!(binding.edit.before_text, "Original B");
    assert_eq!(binding.edit.text, SAVED);
    assert_ne!(
        binding.edit.before_revision_event_id,
        binding.edit.revision_event_id
    );
    assert_eq!(binding.edit.facts.len(), 1);
    assert_eq!(binding.edit.facts[0].field_id, field.field_id);
    assert!(!binding.context_dependencies.is_empty());
    let mut forged = command.payload.clone();
    forged.dependency_id = SemanticDependencyId::new("unconsumed.Eli.tagline").unwrap();
    assert!(crate::script_fact_evidence::capture(&conn, &forged).is_err());
    forged = command.payload.clone();
    forged.block_id = ScriptBlockId::new("block.C").unwrap();
    assert!(crate::script_fact_evidence::capture(&conn, &forged).is_err());
    let mut json = serde_json::to_value(&command.payload).unwrap();
    json["value"] = serde_json::json!("forged truth");
    assert!(serde_json::from_value::<RequestScriptFactProposalCommand>(json).is_err());
    assert_eq!(binding.edit.start_ms, b.segment_start_ms);
    assert_eq!(binding.edit.end_ms, b.segment_end_ms);
}

#[test]
fn pending_rejection_and_explicit_acceptance_preserve_authored_material_and_derive_real_review() {
    let (mut conn, _, b, field, _, command) = fixture();
    let before = crate::script_store::load_document_projection(&conn, &b.document_id)
        .unwrap()
        .unwrap();
    propose(&mut conn, &command);
    assert_eq!(
        before,
        crate::script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap()
    );
    let reject = CommandEnvelope::new(RejectPropagationProposalCommand {
        proposal_id: command.payload.proposal_id.clone(),
        reason: Some("Author rejects synthetic suggestion".into()),
    });
    crate::propagation_proposal_review::record_reject_propagation_proposal(&mut conn, &reject, 40)
        .unwrap();
    assert_eq!(
        before,
        crate::script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        proposal(&conn, &command).status,
        SemanticProposalStatus::Rejected
    );
    let fresh = request(&conn, &b);
    propose(&mut conn, &fresh);
    let detail = crate::bible_graph_store::load_node_detail_projection(&conn, &field.node_id)
        .unwrap()
        .unwrap();
    let sibling = detail.parts[0]
        .fields
        .iter()
        .find(|f| f.value.is_none())
        .unwrap();
    let mut other = field.clone();
    other.field_id = BibleGraphFieldId::new("Mara.unselected").unwrap();
    other.field_key = sibling.field_key.clone();
    other.field_sort_order = sibling.sort_order;
    other.value = Some(FieldValue::Text("UNSELECTED exact sibling — 雨".into()));
    crate::bible_graph_command::apply_set_bible_graph_field(
        &mut conn,
        &CommandEnvelope::new(other.clone()),
        45,
    )
    .unwrap();
    let accept = decision(&fresh);
    crate::propagation_proposal_accept::record_accept_propagation_proposal(&mut conn, &accept, 50)
        .unwrap();
    let after = crate::script_store::load_document_projection(&conn, &b.document_id)
        .unwrap()
        .unwrap();
    for (old, new) in before.segments.iter().zip(&after.segments) {
        assert_eq!(old.segment, new.segment);
        assert_eq!(old.blocks, new.blocks);
    }
    for id in ["segment.A", "segment.B"] {
        let s = after
            .segments
            .iter()
            .find(|s| s.segment.id.as_str() == id)
            .unwrap();
        let impact = s.impact.as_ref().unwrap();
        assert!(impact.needs_review);
        assert!(impact.causes.iter().any(|c|matches!(&c.input,SemanticDependencyEndpoint::BibleField {field_id:Some(id),..} if id==&field.field_id)));
    }
    assert!(
        !after
            .segments
            .iter()
            .find(|s| s.segment.id.as_str() == "segment.C")
            .unwrap()
            .impact
            .as_ref()
            .is_some_and(|i| i.needs_review)
    );
    let detail = crate::bible_graph_store::load_node_detail_projection(&conn, &field.node_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        detail.parts[0]
            .fields
            .iter()
            .find(|f| f.id == field.field_id)
            .unwrap()
            .value,
        Some(FieldValue::Text(VALUE.into()))
    );
    let eli = crate::bible_graph_store::load_node_detail_projection(
        &conn,
        &BibleGraphNodeId::new("Eli").unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(eli.parts[0].fields[0].value, field.value);
    let part: (String, u32) = conn
        .query_row(
            "SELECT name,sort_order FROM bible_graph_parts WHERE id=?1",
            [field.part_id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(part, (field.part_name.clone(), field.part_sort_order));
    let detail = crate::bible_graph_store::load_node_detail_projection(&conn, &field.node_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        detail.parts[0]
            .fields
            .iter()
            .find(|f| f.id == other.field_id)
            .unwrap()
            .value,
        other.value
    );
    let counts = count(&conn);
    assert_eq!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn, &accept, 60
        )
        .unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(count(&conn), counts);
    assert!(
        crate::propagation_proposal_accept::record_accept_propagation_proposal(
            &mut conn,
            &decision(&fresh),
            61
        )
        .is_err()
    );
    assert_eq!(count(&conn), counts);
    assert_eq!(
        proposal(&conn, &fresh).status,
        SemanticProposalStatus::Accepted
    );
}

#[test]
fn stale_edit_fact_and_relationship_aba_refuse_atomically_and_keep_pending_proposal() {
    for mutation in ["edit", "fact", "relationship", "clear", "placement"] {
        let (mut conn, _, b, field, edge, command) = fixture();
        propose(&mut conn, &command);
        let saved = proposal(&conn, &command);
        match mutation {
            "edit" => {
                edit(&mut conn, &b, "Later author text");
                edit(&mut conn, &b, SAVED);
            }
            "fact" => {
                set(&mut conn, &field, Some("Later fact"));
                set(&mut conn, &field, Some("Mara carries a red umbrella."));
            }
            "clear" => set(&mut conn, &field, None),
            "relationship" => {
                let mut changed = edge.clone();
                changed.label = "Mara distrusts Eli".into();
                for e in [changed, edge] {
                    crate::bible_graph_command::apply_set_bible_graph_edge(
                        &mut conn,
                        &CommandEnvelope::new(e),
                        50,
                    )
                    .unwrap();
                }
            }
            _ => {
                let event = ChangeEvent::new(
                    CommandId(uuid::Uuid::new_v4()),
                    ChangeEventKind::UserEdit,
                    "test placement",
                );
                conn.execute(
                    "UPDATE script_segments SET start_ms=start_ms+1 WHERE id=?1",
                    [b.segment_id.as_str()],
                )
                .unwrap();
                let _ = event;
            }
        }
        let counts = count(&conn);
        let before = crate::script_store::load_document_projection(&conn, &b.document_id).unwrap();
        assert!(
            crate::propagation_proposal_accept::record_accept_propagation_proposal(
                &mut conn,
                &decision(&command),
                60
            )
            .is_err(),
            "{mutation}"
        );
        assert_eq!(count(&conn), counts);
        assert_eq!(saved, proposal(&conn, &command));
        assert_eq!(
            before,
            crate::script_store::load_document_projection(&conn, &b.document_id).unwrap()
        );
    }
}

#[test]
fn missing_bound_part_owner_refuses_acceptance_without_materializing_defaults() {
    let (mut conn, _, _, field, _, command) = fixture();
    propose(&mut conn, &command);
    let pending = proposal(&conn, &command);
    conn.execute(
        "UPDATE bible_graph_parts SET deleted_event_id=created_event_id WHERE id=?1",
        [field.part_id.as_str()],
    )
    .unwrap();
    let before = count(&conn);
    let result = crate::propagation_proposal_accept::record_accept_propagation_proposal(
        &mut conn,
        &decision(&command),
        60,
    );
    assert!(result.is_err());
    assert_eq!(count(&conn), before);
    assert_eq!(proposal(&conn, &command), pending);
    let deleted: bool = conn
        .query_row(
            "SELECT deleted_event_id IS NOT NULL FROM bible_graph_parts WHERE id=?1",
            [field.part_id.as_str()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(deleted);
}

#[test]
fn bound_bible_accept_preserves_custom_owner_name_and_order() {
    let (mut conn, _, block, mut field, _, _) = fixture();
    field.part_name = "Mara's authored profile — 雨".into();
    field.part_sort_order = 29;
    // Existing stored owner metadata can differ from merged schema defaults.
    conn.execute(
        "UPDATE bible_graph_parts SET name=?1,sort_order=?2 WHERE id=?3",
        params![
            field.part_name,
            field.part_sort_order,
            field.part_id.as_str()
        ],
    )
    .unwrap();
    let command = request(&conn, &block);
    propose(&mut conn, &command);
    crate::propagation_proposal_accept::record_accept_propagation_proposal(
        &mut conn,
        &decision(&command),
        60,
    )
    .unwrap();
    let metadata: (String, u32) = conn
        .query_row(
            "SELECT name,sort_order FROM bible_graph_parts WHERE id=?1",
            [field.part_id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(metadata, (field.part_name, field.part_sort_order));
}

#[test]
fn delayed_analysis_refuses_drift_and_repeated_request_is_idempotent() {
    let (mut conn, _, b, field, _, command) = fixture();
    let binding = crate::script_fact_evidence::capture(&conn, &command.payload).unwrap();
    set(
        &mut conn,
        &field,
        Some("Changed during synthetic provider request"),
    );
    let counts = count(&conn);
    assert!(
        record(
            &mut conn,
            &command,
            binding,
            FactSuggestion {
                value: VALUE.into(),
                rationale: "Synthetic".into()
            },
            40
        )
        .is_err()
    );
    assert_eq!(count(&conn), counts);
    let fresh = request(&conn, &b);
    propose(&mut conn, &fresh);
    let counts = count(&conn);
    let saved = proposal(&conn, &fresh);
    let binding = saved.script_fact_binding.unwrap();
    assert_eq!(
        record(
            &mut conn,
            &fresh,
            binding,
            FactSuggestion {
                value: "Different late retry output".into(),
                rationale: "Synthetic".into()
            },
            50
        )
        .unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(count(&conn), counts);
}

#[test]
fn bound_proposals_cannot_be_edited_and_rejection_remains_idempotent_after_source_drift() {
    let (mut conn, _, b, _, _, command) = fixture();
    propose(&mut conn, &command);
    let saved = proposal(&conn, &command);
    let counts = count(&conn);
    let update = CommandEnvelope::new(UpdatePropagationProposalCommand {
        proposal_id: saved.id.clone(),
        action: saved.action.clone(),
        target: saved.target.clone(),
        summary: saved.summary.clone(),
        proposed_value: Some(FieldValue::Text("Forged later value".into())),
        proposed_text: None,
        proposed_script_patch: None,
        source_dependency_id: saved.source_dependency_id.clone(),
        source_event_id: saved.source_event_id,
        rationale: saved.rationale.clone(),
    });
    assert!(
        crate::propagation_proposal_update::record_update_propagation_proposal(
            &mut conn, &update, 40
        )
        .is_err()
    );
    assert_eq!(count(&conn), counts);
    assert_eq!(saved, proposal(&conn, &command));
    edit(&mut conn, &b, "Later authored text — 雪\n\n");
    let reject = CommandEnvelope::new(RejectPropagationProposalCommand {
        proposal_id: saved.id,
        reason: None,
    });
    crate::propagation_proposal_review::record_reject_propagation_proposal(&mut conn, &reject, 50)
        .unwrap();
    let counts = count(&conn);
    assert_eq!(
        crate::propagation_proposal_review::record_reject_propagation_proposal(
            &mut conn, &reject, 60
        )
        .unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(count(&conn), counts);
}

#[tokio::test]
async fn synthetic_prompt_contains_only_canonical_selected_fact_and_exact_saved_edit() {
    let (conn, _, _, _, _, command) = fixture();
    let binding = crate::script_fact_evidence::capture(&conn, &command.payload).unwrap();
    let expected = binding.edit.clone();
    let suggestion=analyze_with_provider(&binding,|prompt| async move {
        let evidence:ScriptFactEditEvidence=serde_json::from_str(&prompt.user).unwrap();assert_eq!(evidence,expected);assert_eq!(evidence.facts.len(),1);assert_eq!(evidence.facts[0].node_id.as_str(),"Mara");assert_eq!(evidence.before_text,"Original B");assert_eq!(evidence.text,SAVED);
        Ok(Box::pin(futures::stream::iter(vec![Ok(serde_json::json!({"value":VALUE,"rationale":"Synthetic prompt assertion; no model quality claim"}).to_string())])) as eidetic_core::ai::backend::GenerateStream)
    }).await.unwrap();
    assert_eq!(suggestion.value, VALUE);
}
