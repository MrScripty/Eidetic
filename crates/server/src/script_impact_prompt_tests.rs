use super::*;
use crate::script_impact_review::{
    capture,
    tests::{fixture, request},
};
use futures::stream;

#[tokio::test]
async fn type_review_binds_original_and_current_types_and_requires_fresh_explicit_acceptance() {
    use eidetic_core::contracts::*;
    let (mut conn, _, a, b, c) = fixture();
    let document = crate::script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let source = document
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == a.block_id)
        .unwrap();
    let exact = source.block.text.clone();
    let change = CommandEnvelope::new(EditScriptBlockCommand {
        document_id: a.document_id.clone(),
        block_id: a.block_id.clone(),
        expected_revision_event_id: source.revision_event_id.unwrap(),
        text: exact.clone(),
        block_kind: Some(ScriptBlockKind::Dialogue),
    });
    crate::script_block_edit::apply_edit_script_block(&mut conn, &change, 25).unwrap();
    let command = request(&conn, &b);
    let binding = capture(&conn, &command.payload).unwrap();
    assert_eq!(
        binding.script_previous_inputs.as_ref().unwrap()[0].block_kind,
        Some(ScriptBlockKind::Action)
    );
    let output = preview_with_provider(&binding, |prompt| async move {
        assert!(prompt.user.contains("block_type=dialogue"));
        assert!(
            prompt
                .user
                .contains("ORIGINAL CONSUMED SCREENPLAY TYPE (block.A")
        );
        assert!(prompt.user.contains("): action\nOriginal A"));
        assert!(prompt.user.contains(&exact));
        let response: GenerateStream = Box::pin(stream::iter(vec![Ok(
            "  Synthetic typed preview — 雨.\n\n  ".into(),
        )]));
        Ok(response)
    })
    .await
    .unwrap();
    crate::script_impact_review::record_proposal(
        &mut conn,
        &command,
        binding.clone(),
        output.clone(),
        30,
    )
    .unwrap();
    let original_b = crate::script_impact_review::tests::text(&conn, &b);
    let original_c = crate::script_impact_review::tests::text(&conn, &c);
    let current = binding
        .script_inputs
        .iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    crate::script_block_edit::apply_edit_script_block(
        &mut conn,
        &CommandEnvelope::new(EditScriptBlockCommand {
            document_id: a.document_id.clone(),
            block_id: a.block_id.clone(),
            expected_revision_event_id: current.revision_event_id,
            text: current.text.clone(),
            block_kind: Some(ScriptBlockKind::Character),
        }),
        40,
    )
    .unwrap();
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
        .unwrap();
    assert!(crate::script_impact_review::tests::accept(&mut conn, &command).is_err());
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM commands", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        before
    );
    assert_eq!(
        crate::script_impact_review::tests::text(&conn, &b),
        original_b
    );
    let fresh = request(&conn, &b);
    let fresh_binding = capture(&conn, &fresh.payload).unwrap();
    assert_eq!(
        fresh_binding
            .script_inputs
            .iter()
            .find(|input| input.block_id == a.block_id)
            .unwrap()
            .block_kind,
        Some(ScriptBlockKind::Character)
    );
    crate::script_impact_review::record_proposal(
        &mut conn,
        &fresh,
        fresh_binding,
        output.clone(),
        50,
    )
    .unwrap();
    assert_eq!(
        crate::script_impact_review::tests::text(&conn, &b),
        original_b
    );
    crate::script_impact_review::tests::accept(&mut conn, &fresh).unwrap();
    assert_eq!(crate::script_impact_review::tests::text(&conn, &b), output);
    assert_eq!(
        crate::script_impact_review::tests::text(&conn, &c),
        original_c
    );
    let current = crate::ai_script_context::load_script_context(
        &conn,
        eidetic_core::timeline::node::NodeId(
            uuid::Uuid::parse_str(a.source_node_id.as_ref().unwrap()).unwrap(),
        ),
        a.segment_start_ms,
        a.segment_end_ms,
    )
    .unwrap()
    .into_iter()
    .find(|input| input.block_id == a.block_id)
    .unwrap();
    crate::script_block_edit::apply_edit_script_block(
        &mut conn,
        &CommandEnvelope::new(EditScriptBlockCommand {
            document_id: a.document_id.clone(),
            block_id: a.block_id.clone(),
            expected_revision_event_id: current.revision_event_id,
            text: current.text.clone(),
            block_kind: Some(ScriptBlockKind::Dialogue),
        }),
        60,
    )
    .unwrap();
    let third = capture(&conn, &request(&conn, &b).payload).unwrap();
    assert_eq!(
        third
            .script_previous_inputs
            .as_ref()
            .unwrap()
            .iter()
            .find(|input| input.block_id == a.block_id)
            .unwrap()
            .block_kind,
        Some(ScriptBlockKind::Character)
    );
    assert_eq!(
        third
            .script_inputs
            .iter()
            .find(|input| input.block_id == a.block_id)
            .unwrap()
            .block_kind,
        Some(ScriptBlockKind::Dialogue)
    );
    assert_eq!(crate::script_impact_review::tests::text(&conn, &b), output);
}

#[tokio::test]
async fn deterministic_provider_receives_canonical_authored_evidence_and_resolved_context() {
    let (conn, _, _, b, _) = fixture();
    let command = request(&conn, &b);
    let binding = capture(&conn, &command.payload).unwrap();
    let output = preview_with_provider(&binding, |prompt| async move {
        assert!(
            prompt
                .user
                .contains("  A now carries a blue umbrella — 雨\n\n")
        );
        assert!(prompt.user.contains("TARGET BLOCK TO UPDATE:\nOriginal B"));
        assert!(
            prompt
                .system
                .contains("presentation placement is not fictional time")
        );
        let result: GenerateStream = Box::pin(stream::iter(vec![Ok("  Proposed B\n\n".into())]));
        Ok(result)
    })
    .await
    .unwrap();
    assert_eq!(output, "  Proposed B\n\n");
}

#[tokio::test]
async fn failed_or_empty_provider_output_never_returns_a_partial_preview() {
    for tokens in [
        vec![
            Ok("unfinished prefix".into()),
            Err(eidetic_core::Error::AiBackend("broken provider".into())),
        ],
        vec![],
        vec![Ok(" \n".into())],
    ] {
        let (conn, _, _, b, _) = fixture();
        let binding = capture(&conn, &request(&conn, &b).payload).unwrap();
        assert!(
            preview_with_provider(&binding, |_| async move {
                let result: GenerateStream = Box::pin(stream::iter(tokens));
                Ok(result)
            })
            .await
            .is_err()
        );
    }
}
