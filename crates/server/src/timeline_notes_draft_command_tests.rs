use super::*;
use crate::{timeline_node_store, timeline_notes_lineage};
use eidetic_core::{Project, Template};

fn fixture() -> (Connection, Project) {
    let project = Template::MultiCam.build_project("Notes custody");
    let mut conn = Connection::open_in_memory().unwrap();
    history_store::create_schema(&conn).unwrap();
    conn.execute_batch("CREATE TABLE project (id INTEGER PRIMARY KEY, total_duration_ms INTEGER);")
        .unwrap();
    conn.execute(
        "INSERT INTO project VALUES (1, ?1)",
        [project.timeline.total_duration_ms],
    )
    .unwrap();
    let tx = conn.transaction().unwrap();
    timeline_node_store::upsert_nodes_in_transaction(&tx, &project.timeline.nodes).unwrap();
    timeline_node_store::replace_node_arcs_in_transaction(&tx, &project.timeline.node_arcs)
        .unwrap();
    crate::timeline_relationship_store::upsert_relationships_in_transaction(
        &tx,
        &project.timeline.relationships,
    )
    .unwrap();
    tx.commit().unwrap();
    (conn, project)
}
fn command(
    conn: &Connection,
    project: &Project,
    text: &str,
) -> CommandEnvelope<SetTimelineNodeNotesCommand> {
    let node_id = project.timeline.nodes[0].id;
    CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id,
        notes: text.into(),
        expected: Some(timeline_notes_lineage::capture(conn, node_id).unwrap()),
    })
}
fn save(
    conn: &mut Connection,
    project: &mut Project,
    command: &CommandEnvelope<SetTimelineNodeNotesCommand>,
) {
    assert_eq!(
        record_set_timeline_node_notes_history(conn, project, command, 1).unwrap(),
        RecordChangeOutcome::Recorded
    );
    reload(conn, project);
}
fn reload(conn: &Connection, project: &mut Project) {
    let loaded = timeline_node_store::load_nodes(conn).unwrap();
    for node in &mut project.timeline.nodes {
        *node = loaded
            .iter()
            .find(|current| current.id == node.id)
            .unwrap()
            .clone();
    }
}

#[test]
fn guarded_notes_preserve_exact_text_and_replay_original_owned_receipt_after_another_save() {
    let (mut conn, mut project) = fixture();
    let first = command(&conn, &project, "  Mara reveals the witness — 雨.\n\n  ");
    assert_eq!(
        first.payload.expected.as_ref().unwrap().revision_event_id,
        None
    );
    let other_nodes = project.timeline.nodes[1..].to_vec();
    save(&mut conn, &mut project, &first);
    let receipt = timeline_notes_lineage::command_receipt(&conn, &first).unwrap();
    assert_eq!(receipt.notes, first.payload.notes);
    assert!(receipt.revision_event_id.is_some());
    assert_eq!(
        serde_json::to_value(&project.timeline.nodes[1..]).unwrap(),
        serde_json::to_value(&other_nodes).unwrap()
    );
    let second = command(&conn, &project, "Concurrent writer");
    save(&mut conn, &mut project, &second);
    let lock = CommandEnvelope::new(SetTimelineNodeLockCommand {
        node_id: first.payload.node_id,
        locked: true,
    });
    record_set_timeline_node_lock_history(&mut conn, &project, &lock, 8).unwrap();
    reload(&conn, &mut project);
    let before = timeline_notes_lineage::tests::logical_rows(&conn);
    assert_eq!(
        record_set_timeline_node_notes_history(&mut conn, &project, &first, 9).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(timeline_notes_lineage::tests::logical_rows(&conn), before);
    assert_eq!(
        timeline_notes_lineage::command_receipt(&conn, &first).unwrap(),
        receipt
    );
    assert_eq!(
        timeline_notes_lineage::capture(&conn, first.payload.node_id)
            .unwrap()
            .notes,
        second.payload.notes
    );
    let mut collision = first;
    collision.payload.notes = "Altered retry".into();
    assert!(record_set_timeline_node_notes_history(&mut conn, &project, &collision, 10).is_err());
    assert_eq!(timeline_notes_lineage::tests::logical_rows(&conn), before);
}
#[test]
fn stale_baseline_aba_mismatched_node_locked_and_unchanged_notes_leave_every_table_exact() {
    let (mut conn, mut project) = fixture();
    let stale = command(&conn, &project, "Unsaved intent");
    let initial = stale.payload.expected.as_ref().unwrap().notes.clone();
    let first = command(&conn, &project, "Temporary");
    save(&mut conn, &mut project, &first);
    let restore = command(&conn, &project, &initial);
    save(&mut conn, &mut project, &restore);
    let mut wrong_node = command(&conn, &project, "Another intent");
    wrong_node.payload.expected.as_mut().unwrap().node_id = project.timeline.nodes[1].id;
    let unchanged = command(&conn, &project, &initial);
    for candidate in [&stale, &wrong_node, &unchanged] {
        let before = timeline_notes_lineage::tests::logical_rows(&conn);
        assert!(record_set_timeline_node_notes_history(&mut conn, &project, candidate, 9).is_err());
        assert_eq!(timeline_notes_lineage::tests::logical_rows(&conn), before);
    }
    let lock = CommandEnvelope::new(SetTimelineNodeLockCommand {
        node_id: stale.payload.node_id,
        locked: true,
    });
    record_set_timeline_node_lock_history(&mut conn, &project, &lock, 10).unwrap();
    reload(&conn, &mut project);
    let locked = command(&conn, &project, "Locked intent");
    let before = timeline_notes_lineage::tests::logical_rows(&conn);
    assert!(
        record_set_timeline_node_notes_history(&mut conn, &project, &locked, 11)
            .unwrap_err()
            .to_string()
            .contains("locked")
    );
    assert_eq!(timeline_notes_lineage::tests::logical_rows(&conn), before);
}
#[test]
fn legacy_notes_wire_omits_guard_and_guarded_wire_reuses_notes_lineage_input() {
    let (conn, project) = fixture();
    let mut cmd = command(&conn, &project, " exact\n");
    let guarded = serde_json::to_value(&cmd.payload).unwrap();
    assert_eq!(
        guarded["expected"]["revision_event_id"],
        serde_json::Value::Null
    );
    assert_eq!(
        serde_json::from_value::<SetTimelineNodeNotesCommand>(guarded).unwrap(),
        cmd.payload
    );
    cmd.payload.expected = None;
    let legacy = serde_json::to_value(&cmd.payload).unwrap();
    assert!(legacy.get("expected").is_none());
    assert_eq!(
        serde_json::from_value::<SetTimelineNodeNotesCommand>(legacy)
            .unwrap()
            .expected,
        None
    );
}

#[tokio::test]
async fn public_guarded_notes_return_original_ack_refuse_stale_and_survive_save_reopen() {
    use crate::{
        command_service, persistence, project_service, projection_service, state::AppState,
    };
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Notes service custody");
    let node = project.timeline.nodes[0].id;
    let path = persistence::default_project_dir()
        .join(format!("notes-custody-{}.db", uuid::Uuid::new_v4()));
    persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    project_service::replace_active_project(&state, project, path.clone());
    let query = || projection_service::SelectedNodeEditorProjectionRequest {
        node_id: Some(node),
    };
    let original = projection_service::selected_node_editor_projection(&state, query())
        .await
        .unwrap()
        .payload
        .node
        .unwrap()
        .notes_read
        .unwrap();
    let first = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: node,
        notes: "  Mara reveals the witness — 雨.\n\n  ".into(),
        expected: Some(original.clone()),
    });
    let ack = serde_json::to_value(
        command_service::set_timeline_node_notes(&state, first.clone())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(ack["notes_read"]["notes"], first.payload.notes);
    let current = projection_service::selected_node_editor_projection(&state, query())
        .await
        .unwrap()
        .payload
        .node
        .unwrap()
        .notes_read
        .unwrap();
    assert_eq!(serde_json::to_value(&current).unwrap(), ack["notes_read"]);
    let stale = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: node,
        notes: "Unaccepted stale intent".into(),
        expected: Some(original),
    });
    let before = crate::timeline_postcommit_custody_tests::history(&path);
    let error = command_service::set_timeline_node_notes(&state, stale)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        crate::backend_error::BackendError::Conflict(_)
    ));
    assert!(error.to_string().starts_with("Notes edit refused: "));
    assert_eq!(
        crate::timeline_postcommit_custody_tests::history(&path),
        before
    );
    let second = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: node,
        notes: "Concurrent saved Notes".into(),
        expected: Some(current),
    });
    command_service::set_timeline_node_notes(&state, second)
        .await
        .unwrap();
    let before = crate::timeline_postcommit_custody_tests::history(&path);
    let retry = serde_json::to_value(
        command_service::set_timeline_node_notes(&state, first)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(retry["outcome"], "already_recorded");
    assert_eq!(retry["notes_read"], ack["notes_read"]);
    assert_eq!(
        crate::timeline_postcommit_custody_tests::history(&path),
        before
    );
    project_service::save_project(&state, project_service::SaveProjectRequest { path: None })
        .await
        .unwrap();
    let (reopened, _) = persistence::load_project(&path).await.unwrap();
    assert_eq!(
        reopened.timeline.node(node).unwrap().content.notes,
        "Concurrent saved Notes"
    );
    state.shutdown_tasks();
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
