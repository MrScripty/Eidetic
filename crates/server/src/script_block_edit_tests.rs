use super::*;
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::NodeId;

fn seed(conn: &mut Connection, node: NodeId, start: u64, text: &str) -> ScriptBlockId {
    let block_id = ScriptBlockId::new(format!("block.{}", node.0)).unwrap();
    let command = CommandEnvelope::new(SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: "Screenplay".into(),
        document_sort_order: 3,
        segment_id: ScriptSegmentId::new(format!("segment.{}", node.0)).unwrap(),
        source_node_id: Some(node.0.to_string()),
        segment_start_ms: start,
        segment_end_ms: start + 1000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 7,
        block_id: block_id.clone(),
        block_kind: ScriptBlockKind::Action,
        text: text.into(),
        span_provenance: ScriptSpanProvenance::AiGenerated,
        sort_order: 4,
    });
    script_document_command::apply_set_script_block(conn, &command, 10).unwrap();
    block_id
}

fn edit(
    conn: &Connection,
    block: &ScriptBlockId,
    text: &str,
) -> CommandEnvelope<EditScriptBlockCommand> {
    let document_id = ScriptDocumentId::new("script.document.main").unwrap();
    let projection = script_store::load_document_projection(conn, &document_id)
        .unwrap()
        .unwrap();
    let current = projection
        .segments
        .iter()
        .flat_map(|segment| &segment.blocks)
        .find(|candidate| candidate.block.id == *block)
        .unwrap();
    CommandEnvelope::new(EditScriptBlockCommand {
        block_kind: None,
        document_id,
        block_id: block.clone(),
        expected_revision_event_id: current.revision_event_id.unwrap(),
        text: text.into(),
    })
}

fn commands(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn type_only_edit_preserves_exact_text_spans_and_placement_with_owned_old_new_history() {
    let mut conn = Connection::open_in_memory().unwrap();
    let node = NodeId::new();
    let exact = "  Exact authored line — 雨.\n\n  ";
    let block = seed(&mut conn, node, 0, exact);
    let mut command = edit(&conn, &block, exact);
    let before = projection(&conn, &command).unwrap();
    command.payload.block_kind = Some(ScriptBlockKind::Dialogue);
    let (_, after) = apply_edit_script_block(&mut conn, &command, 20).unwrap();
    let old = &before.payload.segments[0];
    let new = &after.payload.segments[0];
    assert_eq!(new.segment, old.segment);
    assert_eq!(new.blocks[0].block.text, exact);
    assert_eq!(
        new.blocks[0].block.sort_order,
        old.blocks[0].block.sort_order
    );
    assert_eq!(new.blocks[0].spans, old.blocks[0].spans);
    assert_eq!(new.blocks[0].locks, old.blocks[0].locks);
    assert_eq!(new.blocks[0].block.block_kind, ScriptBlockKind::Dialogue);
    let delta: (String,String) = conn.query_row("SELECT f.old_text,f.new_text FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id JOIN change_events e ON e.id=r.change_event_id WHERE e.command_id=?1 AND f.field_key='block_kind'", [command.id.0.to_string()], |row| Ok((row.get(0)?,row.get(1)?))).unwrap();
    assert_eq!(delta, ("action".into(), "dialogue".into()));
    let captured = crate::ai_script_context::load_script_context(&conn, node, 0, 1000).unwrap();
    assert_eq!(captured[0].block_kind, Some(ScriptBlockKind::Dialogue));
    let mut prompt = String::new();
    crate::prompt_format::append_script_context(&mut prompt, &captured);
    assert!(prompt.contains("block_type=dialogue"));
    assert!(prompt.contains(exact));
    assert_eq!(
        apply_edit_script_block(&mut conn, &command, 30).unwrap().0,
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(projection(&conn, &command).unwrap(), after);
}

#[test]
fn type_change_aba_and_a_late_lock_refuse_without_partial_history_or_span_changes() {
    let mut conn = Connection::open_in_memory().unwrap();
    let block = seed(&mut conn, NodeId::new(), 0, "Exact protected text");
    let mut stale = edit(&conn, &block, "Exact protected text");
    stale.payload.block_kind = Some(ScriptBlockKind::Character);
    let mut first = edit(&conn, &block, "Exact protected text");
    first.payload.block_kind = Some(ScriptBlockKind::Dialogue);
    apply_edit_script_block(&mut conn, &first, 20).unwrap();
    let mut restored = edit(&conn, &block, "Exact protected text");
    restored.payload.block_kind = Some(ScriptBlockKind::Action);
    apply_edit_script_block(&mut conn, &restored, 30).unwrap();
    let before = projection(&conn, &stale).unwrap();
    let count = commands(&conn);
    assert!(
        apply_edit_script_block(&mut conn, &stale, 40)
            .unwrap_err()
            .to_string()
            .contains("changed")
    );
    assert_eq!(projection(&conn, &stale).unwrap(), before);
    assert_eq!(commands(&conn), count);
    assert_eq!(
        apply_edit_script_block(&mut conn, &first, 50).unwrap().0,
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(projection(&conn, &stale).unwrap(), before);
    let mut fresh = edit(&conn, &block, "Exact protected text");
    fresh.payload.block_kind = Some(ScriptBlockKind::Dialogue);
    script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("type.lock").unwrap(),
            span_id: ScriptSpanId::new(format!("{}.span.main", block.as_str())).unwrap(),
            reason: "Protected after type read".into(),
        }),
        60,
    )
    .unwrap();
    let before = projection(&conn, &fresh).unwrap();
    let count = commands(&conn);
    assert!(
        apply_edit_script_block(&mut conn, &fresh, 70)
            .unwrap_err()
            .to_string()
            .contains("cannot change the type of a locked")
    );
    assert_eq!(projection(&conn, &fresh).unwrap(), before);
    assert_eq!(commands(&conn), count);
}

#[test]
fn legacy_missing_type_history_stays_unknown_until_an_explicit_owned_write() {
    let mut conn = Connection::open_in_memory().unwrap();
    let node = NodeId::new();
    let block = seed(&mut conn, node, 0, "Legacy exact text");
    conn.execute(
        "DELETE FROM object_revision_fields WHERE field_key='block_kind'",
        [],
    )
    .unwrap();
    let unknown = crate::ai_script_context::load_script_context(&conn, node, 0, 1000).unwrap();
    assert_eq!(unknown[0].block_kind, None);
    let mut prompt = String::new();
    crate::prompt_format::append_script_context(&mut prompt, &unknown);
    assert!(!prompt.contains("block_type="));
    let mut command = edit(&conn, &block, "Legacy exact text");
    command.payload.block_kind = Some(ScriptBlockKind::Dialogue);
    apply_edit_script_block(&mut conn, &command, 20).unwrap();
    assert_eq!(
        crate::ai_script_context::load_script_context(&conn, node, 0, 1000).unwrap()[0].block_kind,
        Some(ScriptBlockKind::Dialogue)
    );
    assert_eq!(unknown[0].block_kind, None);
}

#[test]
fn manual_edit_reopens_exact_text_and_reaches_canonical_prompt_context() {
    let path =
        std::env::temp_dir().join(format!("eidetic-manual-script-{}.db", uuid::Uuid::new_v4()));
    let mut conn = Connection::open(&path).unwrap();
    let first = NodeId::new();
    let second = NodeId::new();
    let first_block = seed(&mut conn, first, 0, "Ada leaves in rain.");
    seed(&mut conn, second, 1000, "Ben opens the door.");
    let before = script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();
    let text = "  Ada leaves under clear skies.\n\nBEN\nWait—please.  ";
    let command = edit(&conn, &first_block, text);
    let (_, saved) = apply_edit_script_block(&mut conn, &command, 20).unwrap();
    assert_eq!(saved.payload.document, before.document);
    assert_eq!(
        saved.payload.segments[0].segment,
        before.segments[0].segment
    );
    assert_eq!(saved.payload.segments[0].blocks[0].block.text, text);
    assert_eq!(
        saved.payload.segments[0].blocks[0].spans[0].provenance,
        ScriptSpanProvenance::UserEdited
    );
    let history = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptBlock,
        first_block.as_str(),
    )
    .unwrap();
    assert_eq!(history.len(), 2);
    assert!(history[1].fields.iter().any(|field| field.old_value
        == Some(FieldValue::Text("Ada leaves in rain.".into()))
        && field.new_value == Some(FieldValue::Text(text.into()))));
    drop(conn);
    let conn = Connection::open(&path).unwrap();
    let reopened =
        script_store::load_document_projection_envelope(&conn, &command.payload.document_id)
            .unwrap()
            .unwrap();
    assert_eq!(reopened, saved);
    let evidence =
        crate::ai_script_context::load_script_context(&conn, second, 1000, 2000).unwrap();
    assert_eq!(evidence.len(), 2);
    assert_eq!(evidence[0].text, text);
    assert_eq!(evidence[0].block_id, first_block);
    assert_eq!(
        evidence[0].revision_event_id,
        saved.payload.segments[0].blocks[0]
            .revision_event_id
            .unwrap()
    );
    let project = eidetic_core::Template::MultiCam.build_project("Context");
    let target = project.timeline.nodes[0].id;
    let mut request = eidetic_core::ai::prompt::build_generate_request(&project, target).unwrap();
    request
        .surrounding_context
        .preceding_scripts
        .push("OLD NODE SCRIPT".into());
    crate::ai_script_context::attach_script_context(&mut request, evidence);
    let prompt = crate::prompt_format::build_chat_prompt(&request);
    assert!(prompt.user.contains(text));
    assert!(prompt.user.contains(first_block.as_str()));
    assert!(prompt.user.contains(&format!(
            "block_revision={}",
            saved.payload.segments[0].blocks[0]
                .revision_event_id
                .unwrap()
                .0
        )));
    assert!(prompt.user.contains(&format!(
        "segment_revision={}",
        command.payload.expected_revision_event_id.0
    )));
    assert!(!prompt.user.contains("Ada leaves in rain."));
    assert!(!prompt.user.contains("OLD NODE SCRIPT"));
    drop(conn);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn stale_edit_and_aba_are_refused_without_history_or_text_changes() {
    let mut conn = Connection::open_in_memory().unwrap();
    let block = seed(&mut conn, NodeId::new(), 0, "Original");
    let stale = edit(&conn, &block, "My pending draft");
    let next = edit(&conn, &block, "Other writer");
    apply_edit_script_block(&mut conn, &next, 20).unwrap();
    let revert = edit(&conn, &block, "Original");
    apply_edit_script_block(&mut conn, &revert, 30).unwrap();
    let before = projection(&conn, &stale).unwrap();
    assert!(
        apply_edit_script_block(&mut conn, &stale, 40)
            .unwrap_err()
            .to_string()
            .contains("changed")
    );
    assert_eq!(projection(&conn, &stale).unwrap(), before);
    assert_eq!(commands(&conn), 3);
    // Replay after intervening edits does not rewrite the later author text.
    assert_eq!(
        apply_edit_script_block(&mut conn, &next, 50).unwrap().0,
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(projection(&conn, &stale).unwrap(), before);
}

#[test]
fn lock_added_after_edit_began_is_enforced_and_rolls_back_history() {
    let mut conn = Connection::open_in_memory().unwrap();
    let block = seed(&mut conn, NodeId::new(), 0, "Protected words");
    let draft = edit(&conn, &block, "Replacement");
    let lock = CommandEnvelope::new(SetScriptLockCommand {
        lock_id: ScriptLockId::new("manual.lock").unwrap(),
        span_id: ScriptSpanId::new(format!("{}.span.main", block.as_str())).unwrap(),
        reason: "Author protected".into(),
    });
    script_document_command::apply_set_script_lock(&mut conn, &lock, 20).unwrap();
    let before = projection(&conn, &draft).unwrap();
    assert!(
        apply_edit_script_block(&mut conn, &draft, 30)
            .unwrap_err()
            .to_string()
            .contains("locked")
    );
    assert_eq!(commands(&conn), 2);
    assert_eq!(projection(&conn, &draft).unwrap(), before);
}

#[test]
fn context_selects_two_adjacent_segments_and_excludes_deleted_text() {
    let mut conn = Connection::open_in_memory().unwrap();
    for index in 0..8 {
        seed(
            &mut conn,
            NodeId::new(),
            index * 1000,
            &format!("Scene {index}"),
        );
    }
    let target = NodeId::new();
    seed(&mut conn, target, 8000, "Target");
    let evidence =
        crate::ai_script_context::load_script_context(&conn, target, 8000, 9000).unwrap();
    assert_eq!(
        evidence
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Scene 6", "Scene 7", "Target"]
    );
    conn.execute(
        "UPDATE script_blocks SET deleted_event_id = updated_event_id WHERE text = 'Scene 7'",
        [],
    )
    .unwrap();
    let evidence =
        crate::ai_script_context::load_script_context(&conn, target, 8000, 9000).unwrap();
    assert!(!evidence.iter().any(|block| block.text == "Scene 7"));
}

#[test]
fn explicit_current_revision_does_not_bypass_a_lock_added_after_the_read() {
    let mut conn = Connection::open_in_memory().unwrap();
    let block = seed(&mut conn, NodeId::new(), 0, "Protected exact text — 雨");
    let compared = edit(&conn, &block, "My retained replacement");
    let lock = CommandEnvelope::new(SetScriptLockCommand {
        lock_id: ScriptLockId::new("comparison.lock").unwrap(),
        span_id: ScriptSpanId::new(format!("{}.span.main", block.as_str())).unwrap(),
        reason: "Protected after comparison".into(),
    });
    script_document_command::apply_set_script_lock(&mut conn, &lock, 20).unwrap();
    let before = projection(&conn, &compared).unwrap();
    let count = commands(&conn);
    assert!(
        apply_edit_script_block(&mut conn, &compared, 30)
            .unwrap_err()
            .to_string()
            .contains("locked")
    );
    assert_eq!(projection(&conn, &compared).unwrap(), before);
    assert_eq!(commands(&conn), count);
}
