use super::*;
use crate::{ai_script_context, script_block_edit, timeline_command_history};
use eidetic_core::contracts::*;
use eidetic_core::timeline::{
    node::{NodeId, StoryLevel, StoryNode},
    timing::TimeRange,
};

pub(crate) fn story_project() -> (eidetic_core::Project, NodeId, NodeId) {
    let mut project = eidetic_core::Template::MultiCam.build_project("Manual first scene");
    project.timeline.total_duration_ms = 10_000;
    project.timeline.nodes.clear();
    project.timeline.node_arcs.clear();
    project.timeline.relationships.clear();
    let mut parent = None;
    for level in [StoryLevel::Premise, StoryLevel::Act, StoryLevel::Sequence] {
        let mut node = StoryNode::new(level.label(), level, TimeRange::new(0, 10_000).unwrap());
        node.parent_id = parent;
        parent = Some(node.id);
        project.timeline.add_node(node).unwrap();
    }
    let mut scenes = Vec::new();
    for (name, start) in [("Cafe", 1000), ("Station", 4000)] {
        let mut node = StoryNode::new_child(
            name,
            StoryLevel::Scene,
            TimeRange::new(start, start + 1000).unwrap(),
            parent.unwrap(),
        );
        node.content.notes = "Planning context stays separate".into();
        scenes.push(node.id);
        project.timeline.add_node(node).unwrap();
    }
    (project, scenes[0], scenes[1])
}

pub(super) fn fixture() -> (Connection, eidetic_core::Project, CreateScriptBlockCommand) {
    let (project, node, _) = story_project();
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON; CREATE TABLE project (id INTEGER PRIMARY KEY, name TEXT NOT NULL, premise TEXT NOT NULL DEFAULT '', total_duration_ms INTEGER); INSERT INTO project VALUES (1, 'Manual first scene', '', 10000);").unwrap();
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    timeline_node_store::replace_node_arcs_in_transaction(&tx, &[]).unwrap();
    crate::timeline_relationship_store::upsert_relationships_in_transaction(&tx, &[]).unwrap();
    tx.commit().unwrap();
    (
        conn,
        project,
        CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: node,
            expected_start_ms: 1000,
            expected_end_ms: 2000,
            block_kind: ScriptBlockKind::Action,
            text: "  INT. CAFE — NIGHT\nMara sees 雨 outside.\n\n".into(),
        },
    )
}

fn count(conn: &Connection, table: &str) -> u64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

#[test]
fn first_manual_block_persists_exact_text_authorship_and_neighbor_memory_on_reopen() {
    let (mut conn, project, payload) = fixture();
    let command = CommandEnvelope::new(payload.clone());
    let (outcome, document) = apply_create_script_block(&mut conn, &command, 10).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::Recorded);
    assert_eq!(document.payload.document.title, project.name);
    let segment = &document.payload.segments[0];
    let block = &segment.blocks[0];
    assert_eq!(block.block.text, payload.text);
    assert_eq!(block.block.block_kind, payload.block_kind);
    assert_eq!(block.spans[0].provenance, ScriptSpanProvenance::UserEdited);
    assert_eq!(block.spans[0].end_byte as usize, payload.text.len());
    assert_eq!(
        segment.segment.source_node_id,
        Some(payload.source_node_id.0.to_string())
    );
    assert_eq!(
        (segment.segment.start_ms, segment.segment.end_ms),
        (1000, 2000)
    );
    let mut stored_nodes = timeline_node_store::load_nodes(&conn).unwrap();
    let mut expected_nodes = project.timeline.nodes.clone();
    stored_nodes.sort_by_key(|node| node.id.0);
    expected_nodes.sort_by_key(|node| node.id.0);
    assert_eq!(
        serde_json::to_value(stored_nodes).unwrap(),
        serde_json::to_value(expected_nodes).unwrap()
    );
    assert_eq!(count(&conn, "commands"), 1);
    assert_eq!(count(&conn, "object_revisions"), 4);
    let directory =
        std::env::temp_dir().join(format!("eidetic-manual-first-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("project.db");
    conn.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    drop(conn);
    let mut reopened = crate::sqlite::open_write_connection(&path).unwrap();
    assert_eq!(
        script_store::load_document_projection_envelope(&reopened, &payload.document_id)
            .unwrap()
            .unwrap(),
        document
    );
    let neighbor = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.name == "Station")
        .unwrap();
    let memory =
        ai_script_context::load_script_context(&reopened, neighbor.id, 4000, 5000).unwrap();
    assert_eq!(memory[0].text, payload.text);
    assert_eq!(memory[0].block_id, block.block.id);
    script_block_edit::apply_edit_script_block(
        &mut reopened,
        &CommandEnvelope::new(EditScriptBlockCommand {
            document_id: payload.document_id,
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: "Mara catches the last train.\n".into(),
        }),
        20,
    )
    .unwrap();
    assert_eq!(
        ai_script_context::load_script_context(&reopened, neighbor.id, 4000, 5000).unwrap()[0].text,
        "Mara catches the last train.\n"
    );
    drop(reopened);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn appends_preserve_existing_block_locks_and_segment_metadata() {
    let (mut conn, _, payload) = fixture();
    let first = CommandEnvelope::new(payload.clone());
    let (_, before) = apply_create_script_block(&mut conn, &first, 10).unwrap();
    let first_block = &before.payload.segments[0].blocks[0];
    crate::script_document_command::apply_set_script_lock(
        &mut conn,
        &CommandEnvelope::new(SetScriptLockCommand {
            lock_id: ScriptLockId::new("manual.lock").unwrap(),
            span_id: first_block.spans[0].id.clone(),
            reason: "Keep exact words".into(),
        }),
        11,
    )
    .unwrap();
    conn.execute(
        "UPDATE nodes SET locked = 1 WHERE id = ?1",
        [payload.source_node_id.0.to_string()],
    )
    .unwrap();
    conn.execute("UPDATE script_segments SET status = 'stale'", [])
        .unwrap();
    let before = script_store::load_document_projection_envelope(&conn, &payload.document_id)
        .unwrap()
        .unwrap();
    let placement_event: String = conn
        .query_row("SELECT updated_event_id FROM script_segments", [], |row| {
            row.get(0)
        })
        .unwrap();
    let mut second = payload;
    second.text = "ELI\nWe can still make it.".into();
    second.block_kind = ScriptBlockKind::Dialogue;
    let (_, after) =
        apply_create_script_block(&mut conn, &CommandEnvelope::new(second), 12).unwrap();
    assert_eq!(after.payload.document, before.payload.document);
    assert_eq!(
        after.payload.segments[0].segment,
        before.payload.segments[0].segment
    );
    assert_eq!(
        after.payload.segments[0].blocks[0],
        before.payload.segments[0].blocks[0]
    );
    assert_eq!(after.payload.segments[0].blocks[1].block.sort_order, 2);
    assert_eq!(
        after.payload.segments[0].blocks[1].block.block_kind,
        ScriptBlockKind::Dialogue
    );
    let membership_event: String = conn
        .query_row("SELECT updated_event_id FROM script_segments", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_ne!(membership_event, placement_event);
    assert_eq!(count(&conn, "object_revisions"), 8); // first 4, lock 1, append 3
    let membership = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::ScriptSegment,
        before.payload.segments[0].segment.id.as_str(),
    )
    .unwrap();
    let revision = membership.last().unwrap();
    assert_eq!(revision.change_event_id.0.to_string(), membership_event);
    assert_eq!(revision.operation, RevisionOperation::Update);
    assert_eq!(revision.fields.len(), 1);
    assert_eq!(
        revision.fields[0],
        FieldDelta::new(
            format!(
                "block.{}",
                after.payload.segments[0].blocks[1].block.id.as_str()
            ),
            None,
            Some(FieldValue::ObjectRef {
                kind: ObjectKind::ScriptBlock,
                id: after.payload.segments[0].blocks[1].block.id.as_str().into()
            })
        )
    );
}

#[test]
fn manual_append_preserves_an_existing_generated_block_and_its_provenance() {
    let (mut conn, _, payload) = fixture();
    let generated = CommandEnvelope::new(SetScriptBlockCommand {
        document_id: payload.document_id.clone(),
        document_title: "Existing screenplay".into(),
        document_sort_order: 3,
        segment_id: ScriptSegmentId::new(format!("script.segment.{}", payload.source_node_id.0))
            .unwrap(),
        source_node_id: Some(payload.source_node_id.0.to_string()),
        segment_start_ms: payload.expected_start_ms,
        segment_end_ms: payload.expected_end_ms,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 4,
        block_id: ScriptBlockId::new(format!(
            "script.block.{}.generated",
            payload.source_node_id.0
        ))
        .unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: "Existing generated opening".into(),
        span_provenance: ScriptSpanProvenance::AiGenerated,
        sort_order: 0,
    });
    let (_, before) =
        script_document_command::apply_set_script_block(&mut conn, &generated, 10).unwrap();
    let (_, after) =
        apply_create_script_block(&mut conn, &CommandEnvelope::new(payload.clone()), 20).unwrap();
    assert_eq!(after.payload.document, before.payload.document);
    assert_eq!(
        after.payload.segments[0].segment,
        before.payload.segments[0].segment
    );
    assert_eq!(
        after.payload.segments[0].blocks[0],
        before.payload.segments[0].blocks[0]
    );
    assert_eq!(after.payload.segments[0].blocks[1].block.text, payload.text);
    assert_eq!(after.payload.segments[0].blocks[1].block.sort_order, 1);
    assert_eq!(
        after.payload.segments[0].blocks[1].spans[0].provenance,
        ScriptSpanProvenance::UserEdited
    );
    assert_eq!(count(&conn, "object_revisions"), 7);
}

#[test]
fn replay_after_edit_and_append_preserves_latest_text_and_rejects_payload_reuse() {
    let (mut conn, _, payload) = fixture();
    let first = CommandEnvelope::new(payload.clone());
    let (_, before) = apply_create_script_block(&mut conn, &first, 10).unwrap();
    let block = &before.payload.segments[0].blocks[0];
    crate::script_block_edit::apply_edit_script_block(
        &mut conn,
        &CommandEnvelope::new(EditScriptBlockCommand {
            document_id: payload.document_id.clone(),
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: "Later authored words".into(),
        }),
        20,
    )
    .unwrap();
    apply_create_script_block(&mut conn, &CommandEnvelope::new(payload), 30).unwrap();
    let count_before = count(&conn, "commands");
    let (outcome, after) = apply_create_script_block(&mut conn, &first, 40).unwrap();
    assert_eq!(outcome, RecordChangeOutcome::AlreadyRecorded);
    assert_eq!(
        after.payload.segments[0].blocks[0].block.text,
        "Later authored words"
    );
    assert_eq!(after.payload.segments[0].blocks.len(), 2);
    assert_eq!(count(&conn, "commands"), count_before);
    let mut reused = first;
    reused.payload.text = "Different retry".into();
    assert!(apply_create_script_block(&mut conn, &reused, 50).is_err());
    assert_eq!(count(&conn, "commands"), count_before);
}

#[test]
fn stale_deleted_and_empty_context_requests_leave_no_authoring_or_history_rows() {
    for kind in ["moved", "deleted", "empty"] {
        let (mut conn, _, mut payload) = fixture();
        match kind {
            "moved" => {
                conn.execute(
                    "UPDATE nodes SET start_ms = 1500 WHERE id = ?1",
                    [payload.source_node_id.0.to_string()],
                )
                .unwrap();
            }
            "deleted" => {
                conn.execute(
                    "DELETE FROM nodes WHERE id = ?1",
                    [payload.source_node_id.0.to_string()],
                )
                .unwrap();
            }
            _ => payload.text = " \n\t".into(),
        }
        assert!(apply_create_script_block(&mut conn, &CommandEnvelope::new(payload), 10).is_err());
        assert_eq!(count(&conn, "commands"), 0);
        assert_eq!(count(&conn, "change_events"), 0);
        assert_eq!(count(&conn, "script_documents"), 0);
        assert_eq!(count(&conn, "script_blocks"), 0);
    }
}

#[test]
fn placement_change_after_writer_admission_rolls_back_command_history_and_text() {
    let (mut conn, _, payload) = fixture();
    history_store::create_schema(&conn).unwrap();
    conn.execute_batch("CREATE TRIGGER move_context AFTER INSERT ON commands WHEN NEW.payload_type = 'script.create_block' BEGIN UPDATE nodes SET start_ms = start_ms + 1; END;").unwrap();
    assert!(apply_create_script_block(&mut conn, &CommandEnvelope::new(payload), 10).is_err());
    assert_eq!(count(&conn, "commands"), 0);
    assert_eq!(count(&conn, "object_revisions"), 0);
    assert_eq!(count(&conn, "script_blocks"), 0);
    assert_eq!(
        conn.query_row(
            "SELECT MIN(start_ms) FROM nodes WHERE level = 'Scene'",
            [],
            |row| row.get::<_, u64>(0)
        )
        .unwrap(),
        1000
    );
}

#[test]
fn timeline_range_write_moves_new_authored_segment_without_changing_text_revision() {
    let (mut conn, project, payload) = fixture();
    let (_, before) =
        apply_create_script_block(&mut conn, &CommandEnvelope::new(payload.clone()), 10).unwrap();
    let block = &before.payload.segments[0].blocks[0];
    let movement = CommandEnvelope::new(SetTimelineNodeRangeCommand {
        node_id: payload.source_node_id,
        start_ms: 6000,
        end_ms: 7000,
    });
    timeline_command_history::record_set_timeline_node_range_history(
        &mut conn, &project, &movement, 20,
    )
    .unwrap();
    let after = script_store::load_document_projection_envelope(&conn, &payload.document_id)
        .unwrap()
        .unwrap();
    let segment = &after.payload.segments[0];
    assert_eq!(
        (segment.segment.start_ms, segment.segment.end_ms),
        (6000, 7000)
    );
    assert_eq!(segment.blocks[0], *block);
    let memory =
        ai_script_context::load_script_context(&conn, payload.source_node_id, 6000, 7000).unwrap();
    assert_eq!(memory[0].text, payload.text);
    assert_eq!((memory[0].start_ms, memory[0].end_ms), (6000, 7000));
}
