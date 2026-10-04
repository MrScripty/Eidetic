use super::tests::{accept, fixture, propose, request, text};
use super::*;

#[test]
fn world_context_changes_refuse_acceptance_without_promoting_screenplay_to_world_facts() {
    let (mut conn, _, _, b, _) = fixture();
    let command = propose(&mut conn, &b);
    crate::bible_graph_command::apply_create_bible_graph_node(
        &mut conn,
        &CommandEnvelope::new(CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".into(),
            sort_order: 0,
        }),
        35,
    )
    .unwrap();
    crate::bible_graph_command::apply_set_bible_graph_snapshot_field(
        &mut conn,
        &CommandEnvelope::new(SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("snapshot.future").unwrap(),
            node_id: BibleGraphNodeId::new("character.ada").unwrap(),
            at_ms: 2000,
            label: "Future".into(),
            snapshot_sort_order: 0,
            field_id: BibleGraphSnapshotFieldId::new("snapshot.future.tagline").unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            field_key: BibleGraphFieldKey::new("tagline").unwrap(),
            value: Some(FieldValue::Text("Future world assertion".into())),
            field_sort_order: 0,
        }),
        36,
    )
    .unwrap();
    assert!(
        accept(&mut conn, &command)
            .unwrap_err()
            .to_string()
            .contains("stale")
    );
    let unknown = capture(&conn, &request(&conn, &b).payload).unwrap();
    assert_eq!(unknown.bible_context.payload.story_time_ms, None);
    assert!(unknown.bible_context.payload.nodes[0].snapshots.is_empty());
    assert!(
        !unknown.bible_context.payload.nodes[0]
            .unresolved_timed_fields
            .is_empty()
    );
    let mut explicit = request(&conn, &b);
    explicit.payload.story_time_ms = Some(2000);
    let at_time = capture(&conn, &explicit.payload).unwrap();
    assert_eq!(
        at_time.bible_context.payload.nodes[0].snapshots[0].fields[0].value,
        FieldValue::Text("Future world assertion".into())
    );
    assert_eq!(text(&conn, &b), "Original B");
}

#[test]
fn lock_added_after_preview_refuses_acceptance_and_allows_explicit_rejection() {
    let (mut conn, _, _, b, _) = fixture();
    let command = propose(&mut conn, &b);
    let document = script_store::load_document_projection(&conn, &b.document_id)
        .unwrap()
        .unwrap();
    let block = document
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|block| block.block.id == b.block_id)
        .unwrap();
    script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("lock.late").unwrap(),
            span_id: block.spans[0].id.clone(),
            reason: "Keep exact wording".into(),
        }),
        35,
    )
    .unwrap();
    let before = script_store::load_document_projection(&conn, &b.document_id).unwrap();
    assert!(accept(&mut conn, &command).is_err());
    propagation_proposal_review::record_reject_propagation_proposal(
        &mut conn,
        &CommandEnvelope::new(RejectPropagationProposalCommand {
            proposal_id: command.payload.proposal_id.clone(),
            reason: None,
        }),
        45,
    )
    .unwrap();
    assert_eq!(
        script_store::load_document_projection(&conn, &b.document_id).unwrap(),
        before
    );
}

#[test]
fn content_regeneration_lock_added_after_preview_is_enforced_at_acceptance() {
    let (mut conn, _, _, b, _) = fixture();
    let command = propose(&mut conn, &b);
    conn.execute(
        "UPDATE nodes SET locked = 1 WHERE id = ?1",
        [b.source_node_id.as_ref().unwrap()],
    )
    .unwrap();
    assert!(
        accept(&mut conn, &command)
            .unwrap_err()
            .to_string()
            .contains("node is locked")
    );
    assert_eq!(text(&conn, &b), "Original B");
}

#[test]
fn deleted_source_is_explainable_in_a_targeted_preview_and_explicit_acceptance_refreshes_it() {
    let (mut conn, _, a, b, _) = fixture();
    let delete = CommandEnvelope::new(a.block_id.clone());
    let event = ChangeEvent::new(delete.id, ChangeEventKind::UserEdit, "Delete source A");
    let revision = ObjectRevision::new(
        ObjectKind::ScriptBlock,
        a.block_id.as_str(),
        event.id,
        RevisionOperation::Delete,
    );
    history_store::record_change_with(
        &mut conn,
        &delete,
        "fixture.delete_source",
        &event,
        &[revision],
        |tx| {
            tx.execute(
                "UPDATE script_blocks SET deleted_event_id = ?1 WHERE id = ?2",
                rusqlite::params![event.id.0.to_string(), a.block_id.as_str()],
            )?;
            Ok(())
        },
    )
    .unwrap();
    let command = request(&conn, &b);
    let binding = capture(&conn, &command.payload).unwrap();
    assert_eq!(binding.cause.reason, ScriptImpactReason::Deleted);
    assert_eq!(binding.cause.input_excerpt.as_deref(), Some("Original A"));
    assert!(
        !binding
            .script_inputs
            .iter()
            .any(|input| input.block_id == a.block_id)
    );
    record_proposal(
        &mut conn,
        &command,
        binding,
        "B continues without A's removed detail".into(),
        35,
    )
    .unwrap();
    accept(&mut conn, &command).unwrap();
    assert_eq!(text(&conn, &b), "B continues without A's removed detail");
    assert!(
        !crate::script_impact_projection::load_impact(&conn, &b.segment_id)
            .unwrap()
            .unwrap()
            .needs_review
    );
}
