use super::*;
use crate::script_impact_review::tests::{accept, edit, request, text};
use crate::{ai_script_context, context_influence_store, script_impact_review, script_store};
use eidetic_core::contracts::*;

fn node(block: &SetScriptBlockCommand) -> NodeId {
    NodeId(uuid::Uuid::parse_str(block.source_node_id.as_ref().unwrap()).unwrap())
}

fn count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
        .unwrap()
}

fn summary(conn: &mut Connection, target: NodeId, value: &str) {
    context_influence_store::record_context_evaluation(
        conn,
        &CommandEnvelope::new(RecordContextEvaluationCommand {
            evaluation: ContextEvaluation {
                id: ContextEvaluationId::new(),
                target_node_id: target,
                task_kind: ContextEvaluationTaskKind::InspectContext,
                summary: "Synthetic recorded summary".into(),
                distilled_context: Some(value.into()),
                created_at_ms: count(conn) as u64 + 100,
            },
            influences: vec![],
        }),
        30,
    )
    .unwrap();
}

#[test]
fn exact_saved_manual_evidence_uses_existing_bounded_window_and_does_not_rewrite_recorded_summaries()
 {
    let (mut conn, _, blocks) = crate::script_context_scope::tests::fixture();
    let a = &blocks[0];
    let b = &blocks[1];
    summary(&mut conn, node(b), "Old synthetic summary: umbrella is RED");
    let before = load(&conn, node(b)).unwrap().unwrap();
    let exact = "  Mara carries a BLUE umbrella — 雨\n\n";
    edit(&mut conn, a, exact);
    let writes = count(&conn);
    let after = load(&conn, node(b)).unwrap().unwrap();
    assert_eq!(
        count(&conn),
        writes,
        "Context reads create no canonical commands"
    );
    assert!(after.version.0 > before.version.0);
    assert_eq!(
        after
            .payload
            .layers
            .last()
            .unwrap()
            .distilled_context
            .as_deref(),
        Some("Old synthetic summary: umbrella is RED")
    );
    let evidence = after.payload.script_context.unwrap();
    let canonical = ai_script_context::load_script_context(
        &conn,
        node(b),
        b.segment_start_ms,
        b.segment_end_ms,
    )
    .unwrap();
    assert_eq!(evidence, canonical);
    let saved_a = evidence
        .iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    assert_eq!(saved_a.text, exact);
    assert_eq!(saved_a.source_node_id, a.source_node_id);
    assert_eq!(saved_a.segment_id, a.segment_id);
    let document = script_store::load_document_projection(&conn, &a.document_id)
        .unwrap()
        .unwrap();
    let projected_a = document
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == a.block_id)
        .unwrap();
    assert_eq!(
        Some(saved_a.revision_event_id),
        projected_a.revision_event_id
    );
    assert_eq!(evidence.len(), 5);
    assert!(
        !evidence
            .iter()
            .any(|input| input.block_id.as_str() == "block.E")
    );
    assert_eq!(text(&conn, b), "  Retained human B — 雨\n\n");
}

#[test]
fn context_evaluation_and_text_aba_advance_the_existing_projection_clock() {
    let (mut conn, _, a, b) = crate::timeline_script_placement::tests::fixture();
    let before = load(&conn, node(&b)).unwrap().unwrap();
    summary(&mut conn, node(&b), "Recorded context changed");
    let evaluated = load(&conn, node(&b)).unwrap().unwrap();
    assert!(evaluated.version.0 > before.version.0);
    assert_ne!(evaluated.change_event_id, before.change_event_id);
    assert_eq!(
        evaluated.payload.script_context,
        before.payload.script_context
    );
    let original = text(&conn, &a);
    edit(&mut conn, &a, "Intervening human version");
    edit(&mut conn, &a, &original);
    let restored = load(&conn, node(&b)).unwrap().unwrap();
    assert!(restored.version.0 > evaluated.version.0);
    let old_a = before
        .payload
        .script_context
        .unwrap()
        .into_iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    let new_a = restored
        .payload
        .script_context
        .unwrap()
        .into_iter()
        .find(|input| input.block_id == a.block_id)
        .unwrap();
    assert_eq!(new_a.text, old_a.text);
    assert_ne!(new_a.revision_event_id, old_a.revision_event_id);
}

#[test]
fn known_empty_context_and_missing_target_remain_distinct_from_legacy_unknown() {
    let mut conn = Connection::open_in_memory().unwrap();
    let project = eidetic_core::Template::MultiCam.build_project("Empty");
    let target = project.timeline.nodes[0].id;
    let tx = conn.transaction().unwrap();
    crate::timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    tx.commit().unwrap();
    let result = load(&conn, target).unwrap().unwrap();
    assert_eq!(result.payload.script_context, Some(vec![]));
    assert_eq!(count(&conn), 0);
    assert!(load(&conn, NodeId::new()).unwrap().is_none());
    assert!(
        ContextStackProjection::from_nodes(&project.timeline.nodes, target)
            .unwrap()
            .script_context
            .is_none()
    );
}

#[test]
fn a_pinned_wal_read_cannot_mix_new_text_or_revision_clock_into_old_context() {
    let (conn, _, a, b) = crate::timeline_script_placement::tests::fixture();
    let before = load(&conn, node(&b)).unwrap().unwrap();
    let path =
        std::env::temp_dir().join(format!("eidetic-context-stack-{}.db", uuid::Uuid::new_v4()));
    conn.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    let reader = crate::sqlite::open_write_connection(&path).unwrap();
    let mut writer = crate::sqlite::open_write_connection(&path).unwrap();
    let tx = reader.unchecked_transaction().unwrap();
    let _: i64 = tx
        .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))
        .unwrap();
    edit(&mut writer, &a, "Committed while context reader was pinned");
    assert_eq!(load(&tx, node(&b)).unwrap().unwrap(), before);
    tx.commit().unwrap();
    let fresh = load(&reader, node(&b)).unwrap().unwrap();
    assert!(fresh.version.0 > before.version.0);
    assert!(
        fresh
            .payload
            .script_context
            .unwrap()
            .iter()
            .any(|input| input.block_id == a.block_id
                && input.text == "Committed while context reader was pinned")
    );
    drop(reader);
    drop(writer);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn manual_context_read_and_synthetic_preview_preserve_screenplay_until_explicit_targeted_acceptance()
 {
    let (mut conn, _, a, b) = crate::timeline_script_placement::tests::fixture();
    let exact = "  Manually authored A — 雨\n\n";
    edit(&mut conn, &a, exact);
    let before_b = text(&conn, &b);
    let stack = load(&conn, node(&b)).unwrap().unwrap();
    assert!(
        stack
            .payload
            .script_context
            .unwrap()
            .iter()
            .any(|input| input.text == exact)
    );
    let req = request(&conn, &b);
    let binding = script_impact_review::capture(&conn, &req.payload).unwrap();
    let replacement = "Synthetic preview: B acknowledges the reviewed human change.\n\n";
    let preview =
        crate::script_impact_prompt::preview_with_provider(&binding, |prompt| async move {
            assert!(prompt.user.contains(exact));
            Ok(
                Box::pin(futures::stream::iter(vec![Ok(replacement.to_string())]))
                    as eidetic_core::ai::backend::GenerateStream,
            )
        })
        .await
        .unwrap();
    script_impact_review::record_proposal(&mut conn, &req, binding, preview, 40).unwrap();
    assert_eq!(text(&conn, &b), before_b);
    let pending = load(&conn, node(&b)).unwrap().unwrap();
    assert!(
        pending
            .payload
            .script_context
            .unwrap()
            .iter()
            .any(|input| input.block_id == b.block_id && input.text == before_b)
    );
    accept(&mut conn, &req).unwrap();
    assert_eq!(text(&conn, &a), exact);
    assert_eq!(text(&conn, &b), replacement);
    let accepted = load(&conn, node(&b)).unwrap().unwrap();
    assert!(
        accepted
            .payload
            .script_context
            .unwrap()
            .iter()
            .any(|input| input.block_id == b.block_id && input.text == replacement)
    );
    assert!(
        !script_store::load_document_projection(&conn, &b.document_id)
            .unwrap()
            .unwrap()
            .segments
            .iter()
            .find(|segment| segment.segment.id == b.segment_id)
            .unwrap()
            .impact
            .as_ref()
            .unwrap()
            .needs_review
    );
}
