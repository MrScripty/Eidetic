use super::*;
use crate::{timeline_node_store, timeline_relationship_store};
use eidetic_core::timeline::relationship::{Relationship, RelationshipType};
use eidetic_core::{Project, Template};

fn fixture() -> (Connection, Project) {
    let mut project = Template::MultiCam.build_project("Shared timeline");
    project.timeline.relationships.push(Relationship::new(
        project.timeline.nodes[0].id,
        project.timeline.nodes[1].id,
        RelationshipType::Thematic,
    ));
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
    timeline_relationship_store::upsert_relationships_in_transaction(
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
    name: &str,
) -> CommandEnvelope<SetTimelineNodeNameCommand> {
    let node = &project.timeline.nodes[0];
    CommandEnvelope::new(SetTimelineNodeNameCommand {
        node_id: node.id,
        name: name.into(),
        expected: TimelineNodeNameRead {
            name: node.name.clone(),
            revision_event_id: timeline_node_store::latest_name_event(conn, node.id, None).unwrap(),
        },
    })
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
fn count(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM commands", [], |r| r.get(0))
        .unwrap()
}
#[test]
fn exact_title_persists_with_old_new_history_and_replays_before_stale_read() {
    let (mut conn, mut project) = fixture();
    let before = project.timeline.clone();
    let command = command(&conn, &project, "  Station departure — 雨.  ");
    assert_eq!(
        record_set_timeline_node_name_history(&mut conn, &project, &command, 1).unwrap(),
        RecordChangeOutcome::Recorded
    );
    reload(&conn, &mut project);
    let mut expected = before.clone();
    expected.nodes[0].name = command.payload.name.clone();
    assert_eq!(
        serde_json::to_value(&project.timeline).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    let revisions = history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &command.payload.node_id.0.to_string(),
    )
    .unwrap();
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].fields,
        vec![FieldDelta::new(
            "name",
            Some(FieldValue::Text(before.nodes[0].name.clone())),
            Some(FieldValue::Text(command.payload.name.clone()))
        )]
    );
    assert_eq!(
        timeline_node_store::latest_name_event(&conn, command.payload.node_id, None).unwrap(),
        Some(revisions[0].change_event_id)
    );
    let path = std::env::temp_dir().join(format!("eidetic-title-{}.db", uuid::Uuid::new_v4()));
    conn.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    let mut reopened = Connection::open(&path).unwrap();
    assert_eq!(
        record_set_timeline_node_name_history(&mut reopened, &project, &command, 2).unwrap(),
        RecordChangeOutcome::AlreadyRecorded
    );
    assert_eq!(count(&reopened), 1);
    let mut restored = project.clone();
    reload(&reopened, &mut restored);
    assert_eq!(
        serde_json::to_value(&restored.timeline).unwrap(),
        serde_json::to_value(&project.timeline).unwrap()
    );
    let mut changed = command.clone();
    changed.payload.name = "Another".into();
    assert!(record_set_timeline_node_name_history(&mut reopened, &project, &changed, 3).is_err());
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn concurrent_and_aba_title_reads_refuse_without_history_writes() {
    let (mut conn, mut project) = fixture();
    let initial = project.clone();
    let stale = command(&conn, &project, "My pending title");
    let first = command(&conn, &project, "Other writer");
    record_set_timeline_node_name_history(&mut conn, &project, &first, 1).unwrap();
    let n = count(&conn);
    assert!(record_set_timeline_node_name_history(&mut conn, &initial, &stale, 2).is_err());
    assert_eq!(count(&conn), n);
    reload(&conn, &mut project);
    let restore = command(&conn, &project, &initial.timeline.nodes[0].name);
    record_set_timeline_node_name_history(&mut conn, &project, &restore, 3).unwrap();
    reload(&conn, &mut project);
    let n = count(&conn);
    assert!(record_set_timeline_node_name_history(&mut conn, &project, &stale, 4).is_err());
    assert_eq!(count(&conn), n);
}
#[test]
fn non_title_edit_keeps_name_clock_and_invalid_title_never_records() {
    let (mut conn, mut project) = fixture();
    let intent = command(&conn, &project, "Author title");
    let notes = CommandEnvelope::new(SetTimelineNodeNotesCommand {
        node_id: intent.payload.node_id,
        notes: "Authored notes".into(),
    });
    record_set_timeline_node_notes_history(&mut conn, &project, &notes, 1).unwrap();
    reload(&conn, &mut project);
    record_set_timeline_node_name_history(&mut conn, &project, &intent, 2).unwrap();
    reload(&conn, &mut project);
    assert_eq!(project.timeline.nodes[0].content.notes, "Authored notes");
    let n = count(&conn);
    for name in [" ", "bad\nline", "\0"] {
        let invalid = command(&conn, &project, name);
        assert!(record_set_timeline_node_name_history(&mut conn, &project, &invalid, 3).is_err());
        assert_eq!(count(&conn), n);
    }
}

#[tokio::test]
async fn public_title_command_refreshes_native_projection_prompt_and_survives_save_reload() {
    use crate::{
        command_service, persistence, project_service, projection_service, state::AppState,
    };
    use eidetic_core::timeline::node::StoryLevel;
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Title service custody");
    let node = project.timeline.nodes_at_level(StoryLevel::Scene)[0].id;
    let path = persistence::default_project_dir()
        .join(format!("title-service-{}.db", uuid::Uuid::new_v4()));
    persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    project_service::replace_active_project(&state, project, path.clone());
    let original = projection_service::selected_node_editor_projection(
        &state,
        projection_service::SelectedNodeEditorProjectionRequest {
            node_id: Some(node),
        },
    )
    .await
    .unwrap();
    let expected = original.payload.node.unwrap().name_read.unwrap();
    let command = CommandEnvelope::new(SetTimelineNodeNameCommand {
        node_id: node,
        name: "  Station departure — 雨.  ".into(),
        expected,
    });
    let response = command_service::set_timeline_node_name(&state, command.clone())
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&response).unwrap()["outcome"],
        "recorded"
    );
    let rendered = serde_json::to_value(&response).unwrap();
    assert!(
        rendered["projection"]["payload"]["clips"]
            .as_array()
            .unwrap()
            .iter()
            .any(|clip| clip["node_id"] == node.0.to_string()
                && clip["name"] == "  Station departure — 雨.  ")
    );
    let after = projection_service::selected_node_editor_projection(
        &state,
        projection_service::SelectedNodeEditorProjectionRequest {
            node_id: Some(node),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        after.payload.node.as_ref().unwrap().name,
        command.payload.name
    );
    assert_eq!(
        after
            .payload
            .node
            .as_ref()
            .unwrap()
            .name_read
            .as_ref()
            .unwrap()
            .name,
        command.payload.name
    );
    let (current, _) = crate::ai_service::active_sqlite_project(&state)
        .await
        .unwrap();
    let mut request = eidetic_core::ai::prompt::build_generate_request(&current, node).unwrap();
    crate::ai_service::attach_ai_generation_context(&mut request, path.clone(), node)
        .await
        .unwrap();
    assert!(
        crate::prompt_format::build_chat_prompt(&request)
            .user
            .contains(&command.payload.name)
    );
    assert!(
        request
            .timeline_title_inputs
            .as_ref()
            .unwrap()
            .iter()
            .any(|input| input.node_id == node
                && input.name == command.payload.name
                && input.revision_event_id.is_some())
    );
    let before = crate::timeline_postcommit_custody_tests::history(&path);
    command_service::set_timeline_node_name(&state, command)
        .await
        .unwrap();
    assert_eq!(
        crate::timeline_postcommit_custody_tests::history(&path),
        before
    );
    project_service::save_project(&state, project_service::SaveProjectRequest { path: None })
        .await
        .unwrap();
    let (reopened, _) = persistence::load_project(&path).await.unwrap();
    assert_eq!(
        reopened.timeline.node(node).unwrap().name,
        "  Station departure — 雨.  "
    );
    state.shutdown_tasks();
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
}
