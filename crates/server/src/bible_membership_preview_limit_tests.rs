use super::*;

fn displace_mara(conn: &mut Connection) {
    // Real production default selection sorts these before Mara and caps at 200.
    for index in 0..201 {
        entity(conn, &format!("A{index:03}"));
    }
}

fn commands(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
        .unwrap()
}

fn assert_missing(conn: &Connection, b: &SetScriptBlockCommand) {
    let context =
        crate::ai_context_projection::load_ai_bible_context_projection(conn, node(b), None)
            .unwrap();
    assert_eq!(context.payload.nodes.len(), 200);
    assert!(
        !context
            .payload
            .nodes
            .iter()
            .any(|node| node.node_id.as_str() == "Mara")
    );
    assert!(
        crate::bible_field_lineage::capture(conn, &context.payload)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn real_node_limit_refuses_blind_membership_preview_then_restored_context_supplies_value_before_acceptance()
 {
    for scoped in [false, true] {
        let (mut conn, _, a, b, c) = fixture();
        entity(&mut conn, "Mara");
        field(&mut conn, "Mara", "tagline", Some("Blue umbrella"));
        field(&mut conn, "Mara", "motivation", None);
        let generated = generation(&conn, &b, scoped);
        script_document_command::apply_generated_script_block(&mut conn, &generated, 30).unwrap();
        edit(&mut conn, &b, "  Exact human B — 雨\n\n");
        let before_a = text(&conn, &a);
        let before_b = text(&conn, &b);
        let before_c = text(&conn, &c);
        displace_mara(&mut conn);
        assert!(!impact(&conn, &b).needs_review);
        field(
            &mut conn,
            "Mara",
            "motivation",
            Some("Keep the station key"),
        );
        assert_missing(&conn, &b);
        assert!(membership(&conn, &b).is_some());
        let writes = commands(&conn);
        let req = request(&conn, &b);
        let error = script_impact_review::capture(&conn, &req.payload).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("restore their context before previewing")
        );
        assert_eq!(commands(&conn), writes);
        assert_eq!(text(&conn, &b), before_b);
        assert!(membership(&conn, &b).is_some());

        assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
        let req = request(&conn, &b);
        let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
        assert!(
            binding
                .bible_inputs
                .iter()
                .any(|input| input.field_id.as_str() == "Mara.motivation"
                    && input.value == FieldValue::Text("Keep the station key".into()))
        );
        let replacement = "Synthetic reviewed B: keep the station key.\n\n";
        let preview =
            crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
                assert!(prompt.user.contains("Keep the station key"));
                assert!(prompt.user.contains("Blue umbrella"));
                assert!(prompt.user.contains("Exact human B"));
                Ok(
                    Box::pin(futures::stream::iter(vec![Ok(replacement.to_string())]))
                        as eidetic_core::ai::backend::GenerateStream,
                )
            })
            .await
            .unwrap();
        script_impact_review::record_proposal(&mut conn, &req, binding, preview, 40).unwrap();
        assert_eq!(text(&conn, &b), before_b);
        accept(&mut conn, &req).unwrap();
        assert!(!impact(&conn, &b).needs_review);
        assert_eq!(text(&conn, &a), before_a);
        assert_eq!(text(&conn, &c), before_c);
        assert_eq!(text(&conn, &b), replacement);
        field(
            &mut conn,
            "Mara",
            "motivation",
            Some("Later authored motivation"),
        );
        assert!(impact(&conn, &b).causes.iter().any(|cause| {
            cause.reason == ScriptImpactReason::Changed
                && matches!(&cause.input, SemanticDependencyEndpoint::BibleField { field_id: Some(id), .. } if id.as_str() == "Mara.motivation")
        }), "Explicit acceptance refreshes actual new-value lineage");
        assert_eq!(text(&conn, &b), replacement);
    }
}

#[test]
fn another_selected_cause_cannot_acknowledge_missing_values_and_context_loss_or_aba_refuses_pending_acceptance()
 {
    let (mut conn, _, a, b, _) = fixture();
    entity(&mut conn, "Mara");
    field(&mut conn, "Mara", "tagline", Some("Blue umbrella"));
    let mut generated = generation(&conn, &b, true);
    generated.payload.script_inputs = Some(
        crate::ai_script_context::load_script_context(
            &conn,
            node(&b),
            b.segment_start_ms,
            b.segment_end_ms,
        )
        .unwrap(),
    );
    script_document_command::apply_generated_script_block(&mut conn, &generated, 30).unwrap();
    edit(&mut conn, &b, "Exact human B remains");
    displace_mara(&mut conn);
    field(
        &mut conn,
        "Mara",
        "motivation",
        Some("Keep the station key"),
    );
    edit(&mut conn, &a, "Independent manual screenplay cause");
    let script_cause = impact(&conn, &b).causes.into_iter().find(|cause| matches!(&cause.input, SemanticDependencyEndpoint::ScriptBlock { block_id } if block_id == &a.block_id)).unwrap();
    let mut req = request(&conn, &b);
    req.payload.dependency_id = script_cause.dependency_id;
    assert!(
        script_impact_review::capture(&conn, &req.payload)
            .unwrap_err()
            .to_string()
            .contains("Bible membership review values")
    );
    assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    script_impact_review::record_proposal(
        &mut conn,
        &req,
        binding,
        "Synthetic pending replacement".into(),
        40,
    )
    .unwrap();
    assign(&mut conn, &b, None, ContextInfluenceKind::Direct);
    assert_missing(&conn, &b);
    let writes = commands(&conn);
    assert!(accept(&mut conn, &req).is_err());
    assert_eq!(commands(&conn), writes);
    assert_eq!(text(&conn, &b), "Exact human B remains");
    let pending = crate::propagation_proposal_store::load_propagation_proposal(
        &conn,
        &req.payload.proposal_id,
    )
    .unwrap()
    .unwrap();
    assert_eq!(pending.status, SemanticProposalStatus::Pending);
    assign(&mut conn, &b, Some("Mara"), ContextInfluenceKind::Direct);
    let writes = commands(&conn);
    assert!(
        accept(&mut conn, &req).is_err(),
        "Restoring visible context cannot revive an old pending binding"
    );
    assert_eq!(commands(&conn), writes);
    assert_eq!(text(&conn, &b), "Exact human B remains");
}
