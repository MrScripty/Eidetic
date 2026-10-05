//! Actual public manual edit and child-planning consumer; providers are synthetic.
use super::*;
use crate::state::{AiConfig, BackendType};
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::StoryLevel;

const BEFORE: &str = "Mara carries a red umbrella.\n\n";
const AFTER: &str = "Mara carries a blue umbrella.\nThe train leaves at midnight.\n\n";

struct Fixture {
    state: AppState,
    path: PathBuf,
    parent: NodeId,
    scene: NodeId,
    block: ScriptBlockId,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.state.shutdown_tasks();
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}

async fn fixture() -> Fixture {
    let state = AppState::new().await;
    let mut project = eidetic_core::Template::MultiCam.build_project("Child plan manual memory");
    let scene = project
        .timeline
        .nodes
        .iter()
        .find(|n| n.level == StoryLevel::Scene)
        .unwrap()
        .clone();
    let parent = scene.id;
    project.timeline.node_mut(parent).unwrap().content.notes = "Develop Mara's departure.".into();
    project.timeline.node_mut(scene.id).unwrap().content.content = "STALE MIRROR SCRIPT".into();
    let path =
        std::env::temp_dir().join(format!("eidetic-child-plan-memory-{}.db", Uuid::new_v4()));
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    crate::project_service::replace_active_project(&state, project, path.clone());
    crate::command_service::create_script_block(
        &state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: scene.id,
            expected_start_ms: scene.time_range.start_ms,
            expected_end_ms: scene.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: BEFORE.into(),
        }),
    )
    .await
    .unwrap();
    let conn = crate::sqlite::open_write_connection(&path).unwrap();
    let projection = crate::script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap();
    let block = &projection.segments[0].blocks[0];
    crate::command_service::edit_script_block(
        &state,
        CommandEnvelope::new(EditScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: AFTER.into(),
        }),
    )
    .await
    .unwrap();
    Fixture {
        state,
        path,
        parent,
        scene: scene.id,
        block: block.block.id.clone(),
    }
}

#[tokio::test]
async fn manual_screenplay_edit_reaches_child_decomposition_prompt() {
    let fixture = fixture().await;
    let (project, path) = active_sqlite_project(&fixture.state).await.unwrap();
    let mut request = build_generate_children_request(&project, fixture.parent).unwrap();
    attach_ai_generation_context_to_children(&mut request, path, fixture.parent, None)
        .await
        .unwrap();
    let prompt = build_decompose_prompt(&request);
    println!(
        "saved_manual_text_present_in_child_prompt={}",
        prompt.user.contains(AFTER)
    );
    assert!(
        prompt.user.contains(AFTER),
        "child planning must consume exact saved screenplay after a public manual edit"
    );
    assert!(!prompt.user.contains(BEFORE));
    assert!(!prompt.user.contains("STALE MIRROR SCRIPT"));
}

fn script(fixture: &Fixture) -> ScriptDocumentProjection {
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    crate::script_store::load_document_projection(
        &conn,
        &ScriptDocumentId::new("script.document.main").unwrap(),
    )
    .unwrap()
    .unwrap()
}

async fn edit(fixture: &Fixture, text: &str) {
    let projection = script(fixture);
    let block = projection
        .segments
        .iter()
        .flat_map(|s| &s.blocks)
        .find(|b| b.block.id == fixture.block)
        .unwrap();
    crate::command_service::edit_script_block(
        &fixture.state,
        CommandEnvelope::new(EditScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            block_id: fixture.block.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: text.into(),
        }),
    )
    .await
    .unwrap();
}

fn provider() -> (
    String,
    tokio::sync::oneshot::Receiver<serde_json::Value>,
    std::sync::mpsc::Sender<()>,
    std::thread::JoinHandle<()>,
) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (seen, request) = tokio::sync::oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let thread = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = socket.read(&mut buffer).unwrap();
            assert_ne!(count, 0);
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                let length: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if bytes.len() >= end + 4 + length {
                    let body: serde_json::Value =
                        serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap();
                    assert_eq!(body["stream"], false);
                    seen.send(body).unwrap();
                    break;
                }
            }
        }
        wait.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let children = serde_json::json!([{"name":"Midnight departure", "outline":"Mara takes her blue umbrella to the midnight train.", "weight":1.0, "characters":[], "props":[]}]);
        let body = serde_json::json!({"choices":[{"message":{"content":children.to_string()}}]})
            .to_string();
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    (url, request, release, thread)
}

async fn planning(
    fixture: &Fixture,
) -> (
    tokio::task::JoinHandle<Result<ChildPlan, BackendError>>,
    serde_json::Value,
    std::sync::mpsc::Sender<()>,
    std::thread::JoinHandle<()>,
) {
    let (url, request, release, thread) = provider();
    *fixture.state.ai_config.lock() = AiConfig {
        backend_type: BackendType::LlamaCpp,
        model: "synthetic-child-plan".into(),
        base_url: url,
        ..AiConfig::default()
    };
    let state = fixture.state.clone();
    let parent = fixture.parent;
    let task = tokio::spawn(async move {
        generate_children(
            &state,
            AiGenerateChildrenRequest {
                node_id: parent.0,
                story_time_ms: None,
            },
        )
        .await
    });
    (task, request.await.unwrap(), release, thread)
}

fn accept_command(plan: &ChildPlan) -> serde_json::Value {
    serde_json::json!({"id": Uuid::new_v4(), "payload": {"parent_id":plan.parent_node_id, "child_plan_id":plan.id, "children": plan.children.iter().map(|c| serde_json::json!({"name":c.name,"outline":c.outline,"weight":c.weight,"beat_type":c.beat_type,"characters":c.characters,"location":c.location,"props":c.props})).collect::<Vec<_>>()}})
}

#[tokio::test]
async fn actual_http_child_plan_is_reviewable_preserves_screenplay_and_accepts_only_explicitly() {
    let fixture = fixture().await;
    let before = script(&fixture);
    let (old_project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let (task, request, release, provider) = planning(&fixture).await;
    assert!(
        request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains(AFTER)
    );
    release.send(()).unwrap();
    let plan = task.await.unwrap().unwrap();
    provider.join().unwrap();
    let evidence = plan.script_context.as_ref().unwrap();
    assert_eq!(evidence[0].text, AFTER);
    assert_eq!(
        evidence[0].source_node_id.as_deref(),
        Some(fixture.scene.0.to_string().as_str())
    );
    assert_eq!(script(&fixture), before);
    let (pending_project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    assert_eq!(
        serde_json::to_value(&pending_project.timeline).unwrap(),
        serde_json::to_value(&old_project.timeline).unwrap()
    );
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    let pending =
        crate::child_plan_projection_store::load_child_plan_list_projection(&conn).unwrap();
    assert_eq!(
        pending.payload.plans[0].status,
        eidetic_core::ai::backend::ChildPlanStatus::Pending
    );
    assert_eq!(
        pending.payload.plans[0].plan.script_context,
        plan.script_context
    );
    let command = accept_command(&plan);
    let accepted = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(command.clone()).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(accepted).unwrap()["outcome"],
        "recorded"
    );
    assert_eq!(script(&fixture), before);
    let (changed, _) = active_sqlite_project(&fixture.state).await.unwrap();
    assert!(changed.timeline.nodes.iter().any(
        |n| n.parent_id == Some(fixture.parent) && n.content.notes == plan.children[0].outline
    ));
    let applied =
        crate::child_plan_projection_store::load_child_plan_list_projection(&conn).unwrap();
    assert_eq!(
        applied.payload.plans[0].status,
        eidetic_core::ai::backend::ChildPlanStatus::Applied
    );
    edit(&fixture, "Exact later human edit.\n\n").await;
    let later = script(&fixture);
    let replay = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(command).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(replay).unwrap()["outcome"],
        "already_recorded"
    );
    assert_eq!(script(&fixture), later);
}

#[tokio::test]
async fn delayed_child_plan_refuses_manual_text_aba_without_plan_or_partial_history() {
    let fixture = fixture().await;
    let (task, request, release, provider) = planning(&fixture).await;
    assert!(
        request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains(AFTER)
    );
    edit(&fixture, "Intervening human text").await;
    edit(&fixture, AFTER).await;
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM commands", [], |row| row.get(0))
        .unwrap();
    let exact = script(&fixture);
    release.send(()).unwrap();
    let error = task.await.unwrap().unwrap_err();
    provider.join().unwrap();
    assert!(
        error
            .to_string()
            .contains("Child plan story context changed")
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM child_plans", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM commands", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        count
    );
    assert_eq!(script(&fixture), exact);
}

#[tokio::test]
async fn pending_child_plan_refuses_manual_text_aba_on_acceptance_and_retains_pending_evidence() {
    let fixture = fixture().await;
    let (task, _, release, provider) = planning(&fixture).await;
    release.send(()).unwrap();
    let plan = task.await.unwrap().unwrap();
    provider.join().unwrap();
    edit(&fixture, "Intervening human text").await;
    edit(&fixture, AFTER).await;
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM commands", [], |row| row.get(0))
        .unwrap();
    let exact = script(&fixture);
    let error = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(accept_command(&plan)).unwrap(),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Child plan story context changed")
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM commands", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        count
    );
    let pending =
        crate::child_plan_projection_store::load_child_plan_list_projection(&conn).unwrap();
    assert_eq!(
        pending.payload.plans[0].status,
        eidetic_core::ai::backend::ChildPlanStatus::Pending
    );
    assert_eq!(
        pending.payload.plans[0].plan.script_context,
        plan.script_context
    );
    assert_eq!(script(&fixture), exact);
}

#[tokio::test]
async fn pending_child_plan_refuses_changed_window_membership_and_forged_reviewed_material() {
    let fixture = fixture().await;
    let (task, _, release, provider) = planning(&fixture).await;
    release.send(()).unwrap();
    let plan = task.await.unwrap().unwrap();
    provider.join().unwrap();
    let mut altered = accept_command(&plan);
    altered["payload"]["children"][0]["outline"] = serde_json::json!("Unreviewed substitution");
    let error = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(altered).unwrap(),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differ from the reviewed child plan")
    );
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let scene = project.timeline.node(fixture.scene).unwrap();
    crate::command_service::create_script_block(
        &fixture.state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: fixture.scene,
            expected_start_ms: scene.time_range.start_ms,
            expected_end_ms: scene.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: "New human scene material".into(),
        }),
    )
    .await
    .unwrap();
    let before = script(&fixture);
    let error = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(accept_command(&plan)).unwrap(),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Child plan story context changed")
    );
    assert_eq!(script(&fixture), before);
}

#[tokio::test]
async fn descendant_notes_aba_refuses_a_pending_plan_without_overwriting_manual_timeline_edits() {
    let fixture = fixture().await;
    let child = NodeId(Uuid::new_v4());
    crate::command_service_timeline::create_timeline_child_from_parent_core_command(
        &fixture.state,
        CommandEnvelope::new(CreateTimelineChildFromParentCommand {
            node_id: child,
            parent_id: fixture.parent,
        }),
    )
    .await
    .unwrap();
    crate::command_service::set_timeline_node_notes(
        &fixture.state,
        CommandEnvelope::new(SetTimelineNodeNotesCommand {
            node_id: child,
            notes: "Original manual beat notes".into(),
        }),
    )
    .await
    .unwrap();
    let (task, _, release, provider) = planning(&fixture).await;
    release.send(()).unwrap();
    let plan = task.await.unwrap().unwrap();
    provider.join().unwrap();
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let initial = project.timeline.node(child).unwrap().content.notes.clone();
    for notes in ["Intervening manual child notes".to_owned(), initial] {
        crate::command_service::set_timeline_node_notes(
            &fixture.state,
            CommandEnvelope::new(SetTimelineNodeNotesCommand {
                node_id: child,
                notes,
            }),
        )
        .await
        .unwrap();
    }
    let error = crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(accept_command(&plan)).unwrap(),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Child plan story context changed")
    );
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    assert!(project.timeline.node(child).is_ok());
}

#[tokio::test]
async fn unselected_distant_screenplay_text_edit_does_not_stale_the_pending_plan() {
    let fixture = fixture().await;
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let distant = project
        .timeline
        .nodes
        .iter()
        .filter(|node| node.level == StoryLevel::Scene)
        .max_by_key(|node| node.time_range.start_ms)
        .unwrap();
    crate::command_service::create_script_block(
        &fixture.state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: distant.id,
            expected_start_ms: distant.time_range.start_ms,
            expected_end_ms: distant.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: "Unselected distant scene".into(),
        }),
    )
    .await
    .unwrap();
    // Fill the existing two-following-segment bound so the far scene is excluded.
    for scene in project
        .timeline
        .nodes
        .iter()
        .filter(|n| {
            n.level == StoryLevel::Scene
                && n.time_range.start_ms
                    > project
                        .timeline
                        .node(fixture.scene)
                        .unwrap()
                        .time_range
                        .end_ms
        })
        .take(2)
    {
        crate::command_service::create_script_block(
            &fixture.state,
            CommandEnvelope::new(CreateScriptBlockCommand {
                document_id: ScriptDocumentId::new("script.document.main").unwrap(),
                source_node_id: scene.id,
                expected_start_ms: scene.time_range.start_ms,
                expected_end_ms: scene.time_range.end_ms,
                block_kind: ScriptBlockKind::Action,
                text: "Bounded neighbor evidence".into(),
            }),
        )
        .await
        .unwrap();
    }
    let (task, request, release, provider) = planning(&fixture).await;
    assert!(
        !request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains("Unselected distant scene")
    );
    release.send(()).unwrap();
    let plan = task.await.unwrap().unwrap();
    provider.join().unwrap();
    let projection = script(&fixture);
    let block = projection
        .segments
        .iter()
        .flat_map(|s| &s.blocks)
        .find(|b| b.block.text == "Unselected distant scene")
        .unwrap();
    crate::command_service::edit_script_block(
        &fixture.state,
        CommandEnvelope::new(EditScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            block_id: block.block.id.clone(),
            expected_revision_event_id: block.revision_event_id.unwrap(),
            text: "Later distant human edit".into(),
        }),
    )
    .await
    .unwrap();
    let before = script(&fixture);
    crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(accept_command(&plan)).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(script(&fixture), before);
}
