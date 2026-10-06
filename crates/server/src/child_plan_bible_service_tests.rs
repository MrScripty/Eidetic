//! Public Bible writes and actual HTTP child planning; responses are synthetic.
use std::collections::BTreeMap;
use std::path::PathBuf;

use eidetic_core::ai::backend::{ChildPlan, ChildPlanStatus};
use eidetic_core::contracts::*;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use rusqlite::types::Value;
use uuid::Uuid;

use crate::ai_service::{AiGenerateChildrenRequest, active_sqlite_project, generate_children};
use crate::backend_error::BackendError;
use crate::state::{AiConfig, AppState, BackendType};

const BLUE: &str = "Mara's umbrella is blue.";
const GOLD: &str = "Mara's umbrella is gold.";
const SCRIPT: &str = "INT. STATION - NIGHT\n\nMara waits with her blue umbrella.\n\n";
const REFUSAL: &str =
    "Child plan story context changed; generate and review a fresh plan before accepting";

struct Fixture {
    state: AppState,
    path: PathBuf,
    parent: NodeId,
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
    let mut project = eidetic_core::Template::MultiCam.build_project("Bible-bound child plans");
    let scene = project
        .timeline
        .nodes
        .iter()
        .find(|node| node.level == StoryLevel::Scene)
        .unwrap()
        .clone();
    project.timeline.node_mut(scene.id).unwrap().content.notes =
        "Plan Mara's departure using the story Bible and exact saved screenplay.".into();
    let path = std::env::temp_dir().join(format!("eidetic-child-bible-{}.db", Uuid::new_v4()));
    crate::persistence::save_project(&project, &path, None)
        .await
        .unwrap();
    crate::project_service::replace_active_project(&state, project, path.clone());
    let fixture = Fixture {
        state,
        path,
        parent: scene.id,
    };
    crate::command_service::create_script_block(
        &fixture.state,
        CommandEnvelope::new(CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: scene.id,
            expected_start_ms: scene.time_range.start_ms,
            expected_end_ms: scene.time_range.end_ms,
            block_kind: ScriptBlockKind::Action,
            text: SCRIPT.into(),
        }),
    )
    .await
    .unwrap();
    node(&fixture, "Mara", "Mara", 0).await;
    node(&fixture, "Eli", "Eli", 1).await;
    field(&fixture, "Mara", "tagline", Some(BLUE)).await;
    edge(&fixture, "Mara.Eli", "Mara", "Eli", "waiting for Eli").await;
    // Initialize durable proposal storage through its real public read API,
    // so a late refused generation cannot confuse empty schema with writes.
    assert!(
        crate::projection_service::child_plan_list_projection(&fixture.state)
            .await
            .unwrap()
            .payload
            .plans
            .is_empty()
    );
    crate::projection_service::bible_reference_proposal_list_projection(&fixture.state)
        .await
        .unwrap();
    fixture
}

async fn node(fixture: &Fixture, id: &str, name: &str, order: u32) {
    crate::command_service::create_bible_graph_node(
        &fixture.state,
        serde_json::from_value(serde_json::json!({"id":Uuid::new_v4(), "payload": {
            "node_id":id, "schema_key":"character", "name":name, "sort_order":order
        }}))
        .unwrap(),
    )
    .await
    .unwrap();
}

async fn field(fixture: &Fixture, owner: &str, key: &str, value: Option<&str>) {
    crate::command_service::set_bible_graph_field(
        &fixture.state,
        CommandEnvelope::new(SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new(owner).unwrap(),
            part_id: BibleGraphPartId::new(format!("part.{owner}.profile")).unwrap(),
            part_key: BibleGraphPartKey::new("profile").unwrap(),
            part_name: "Profile".into(),
            part_sort_order: 0,
            field_id: BibleGraphFieldId::new(format!("{owner}.{key}")).unwrap(),
            field_key: BibleGraphFieldKey::new(key).unwrap(),
            value: value.map(|value| FieldValue::Text(value.into())),
            field_sort_order: 0,
        }),
    )
    .await
    .unwrap();
}

async fn rename(fixture: &Fixture, name: &str) {
    crate::command_service::set_bible_graph_node_name(
        &fixture.state,
        CommandEnvelope::new(SetBibleGraphNodeNameCommand {
            node_id: BibleGraphNodeId::new("Mara").unwrap(),
            name: name.into(),
        }),
    )
    .await
    .unwrap();
}

async fn edge(fixture: &Fixture, id: &str, from: &str, to: &str, label: &str) {
    crate::command_service::set_bible_graph_edge(
        &fixture.state,
        serde_json::from_value(serde_json::json!({"id":Uuid::new_v4(), "payload": {
            "edge_id":id, "from_node_id":from, "to_node_id":to,
            "edge_kind":"references", "label":label, "directed":true, "sort_order":0
        }}))
        .unwrap(),
    )
    .await
    .unwrap();
}

async fn snapshot_field(fixture: &Fixture, owner: &str, value: &str) {
    crate::command_service::set_bible_graph_snapshot_field(
        &fixture.state,
        serde_json::from_value(serde_json::json!({"id":Uuid::new_v4(), "payload": {
            "snapshot_id":format!("{owner}.future"), "node_id":owner,
            "at_ms":1000, "label":"Future assertion", "snapshot_sort_order":0,
            "field_id":format!("{owner}.future.tagline"), "part_key":"profile",
            "part_name":"Profile", "field_key":"tagline",
            "value":{"type":"text","value":value}, "field_sort_order":0
        }}))
        .unwrap(),
    )
    .await
    .unwrap();
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

fn rows(fixture: &Fixture) -> BTreeMap<String, Vec<Vec<Value>>> {
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    let tables = {
        let mut statement = tx
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    tables
        .into_iter()
        .map(|name| {
            let sql = format!(
                "SELECT * FROM \"{}\" ORDER BY rowid",
                name.replace('"', "\"\"")
            );
            let mut statement = tx.prepare(&sql).unwrap();
            let columns = statement.column_count();
            let values = statement
                .query_map([], |row| {
                    (0..columns)
                        .map(|index| row.get::<_, Value>(index))
                        .collect()
                })
                .unwrap()
                .collect::<Result<Vec<Vec<Value>>, _>>()
                .unwrap();
            (name, values)
        })
        .collect()
}

fn assert_rows_unchanged(fixture: &Fixture, before: &BTreeMap<String, Vec<Vec<Value>>>) {
    let after = rows(fixture);
    let tables = before
        .keys()
        .chain(after.keys())
        .collect::<std::collections::BTreeSet<_>>();
    for table in tables {
        assert!(
            before.get(table) == after.get(table),
            "durable table {table} changed: before {} rows, after {} rows",
            before.get(table).map_or(0, Vec::len),
            after.get(table).map_or(0, Vec::len)
        );
    }
}

async fn fill_bible_cap(fixture: &Fixture) {
    for index in 0..200 {
        node(
            fixture,
            &format!("filler.{index:03}"),
            &format!("Filler {index:03}"),
            index + 10,
        )
        .await;
    }
}

fn current_bible(fixture: &Fixture) -> AiBibleContextProjection {
    let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
    crate::ai_context_projection::load_ai_bible_context_projection(&conn, fixture.parent, None)
        .unwrap()
        .payload
}

fn command(plan: &ChildPlan) -> serde_json::Value {
    serde_json::json!({"id":Uuid::new_v4(), "payload": {
        "parent_id":plan.parent_node_id, "child_plan_id":plan.id,
        "children":plan.children.iter().map(|child| serde_json::json!({
            "name":child.name, "outline":child.outline, "weight":child.weight,
            "beat_type":child.beat_type, "characters":child.characters,
            "props":child.props, "location":child.location
        })).collect::<Vec<_>>()
    }})
}

async fn apply(
    fixture: &Fixture,
    command: serde_json::Value,
) -> Result<serde_json::Value, BackendError> {
    crate::command_service::apply_timeline_children(
        &fixture.state,
        serde_json::from_value(command).unwrap(),
    )
    .await
    .map(|response| serde_json::to_value(response).unwrap())
}

struct Planning {
    task: tokio::task::JoinHandle<Result<ChildPlan, BackendError>>,
    request: serde_json::Value,
    release: std::sync::mpsc::Sender<()>,
    provider: std::thread::JoinHandle<()>,
}

async fn planning(fixture: &Fixture) -> Planning {
    planning_at(fixture, None).await
}

async fn planning_at(fixture: &Fixture, story_time_ms: Option<u64>) -> Planning {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (seen, request) = tokio::sync::oneshot::channel();
    let (release, wait) = std::sync::mpsc::channel();
    let provider = std::thread::spawn(move || {
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
            if let Some(end) = bytes.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
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
        wait.recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let children = serde_json::json!([{"name":" Reviewed departure ", "outline":" Mara leaves for the train. ", "weight":1.0, "characters":[" Mara "], "props":[" Umbrella "]}]);
        let body = serde_json::json!({"choices":[{"message":{"content":children.to_string()}}]})
            .to_string();
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    *fixture.state.ai_config.lock() = AiConfig {
        backend_type: BackendType::LlamaCpp,
        model: "synthetic-bible-child-plan".into(),
        base_url,
        ..AiConfig::default()
    };
    let state = fixture.state.clone();
    let parent = fixture.parent;
    let task = tokio::spawn(async move {
        generate_children(
            &state,
            AiGenerateChildrenRequest {
                node_id: parent.0,
                story_time_ms,
            },
        )
        .await
    });
    Planning {
        task,
        request: request.await.unwrap(),
        release,
        provider,
    }
}

async fn finish(planning: Planning) -> Result<ChildPlan, BackendError> {
    planning.release.send(()).unwrap();
    let result = planning.task.await.unwrap();
    planning.provider.join().unwrap();
    result
}

async fn pending(fixture: &Fixture) -> ChildPlan {
    let request = planning(fixture).await;
    assert!(
        request.request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains(SCRIPT)
    );
    finish(request).await.unwrap()
}

async fn refuse_unchanged(fixture: &Fixture, plan: &ChildPlan) {
    let before = rows(fixture);
    let saved = script(fixture);
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    let timeline = serde_json::to_value(project.timeline).unwrap();
    let error = apply(fixture, command(plan)).await.unwrap_err();
    assert_eq!(error, BackendError::Conflict(REFUSAL.into()));
    assert_rows_unchanged(fixture, &before);
    assert_eq!(script(fixture), saved);
    let (project, _) = active_sqlite_project(&fixture.state).await.unwrap();
    assert_eq!(serde_json::to_value(project.timeline).unwrap(), timeline);
}

#[tokio::test]
async fn fact_only_drift_refuses_old_plan_and_fresh_acceptance_preserves_saved_screenplay_and_replay()
 {
    let fixture = fixture().await;
    let saved = script(&fixture);
    let initial_request = planning(&fixture).await;
    let user = initial_request.request["messages"][1]["content"]
        .as_str()
        .unwrap();
    assert!(user.contains(SCRIPT));
    assert!(user.contains(BLUE));
    let old = finish(initial_request).await.unwrap();
    let evidence = old.bible_context.as_ref().unwrap();
    assert_eq!(evidence.inputs.len(), 1);
    assert_eq!(evidence.inputs[0].value, FieldValue::Text(BLUE.into()));
    assert_eq!(evidence.inputs[0].field_id.as_str(), "Mara.tagline");
    field(&fixture, "Mara", "tagline", Some(GOLD)).await;
    assert_eq!(
        script(&fixture),
        saved,
        "fact-only change preserves every screenplay revision"
    );
    refuse_unchanged(&fixture, &old).await;

    let fresh_request = planning(&fixture).await;
    assert!(
        fresh_request.request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains(GOLD)
    );
    let fresh = finish(fresh_request).await.unwrap();
    assert_eq!(
        fresh.bible_context.as_ref().unwrap().inputs[0].value,
        FieldValue::Text(GOLD.into())
    );
    let accepted = command(&fresh);
    assert_eq!(
        apply(&fixture, accepted.clone()).await.unwrap()["outcome"],
        "recorded"
    );
    assert_eq!(script(&fixture), saved);
    let durable = crate::projection_service::child_plan_list_projection(&fixture.state)
        .await
        .unwrap();
    assert_eq!(
        durable
            .payload
            .plans
            .iter()
            .find(|record| record.plan.id == old.id)
            .unwrap()
            .status,
        ChildPlanStatus::Pending
    );
    assert_eq!(
        durable
            .payload
            .plans
            .iter()
            .find(|record| record.plan.id == fresh.id)
            .unwrap()
            .status,
        ChildPlanStatus::Applied
    );
    field(&fixture, "Mara", "tagline", Some(BLUE)).await;
    let before_replay = rows(&fixture);
    assert_eq!(
        apply(&fixture, accepted).await.unwrap()["outcome"],
        "already_recorded"
    );
    assert_rows_unchanged(&fixture, &before_replay);
    assert_eq!(script(&fixture), saved);
}

#[tokio::test]
async fn bible_fact_aba_refuses_even_when_context_value_matches_original() {
    let fixture = fixture().await;
    let plan = pending(&fixture).await;
    field(&fixture, "Mara", "tagline", Some(GOLD)).await;
    field(&fixture, "Mara", "tagline", Some(BLUE)).await;
    refuse_unchanged(&fixture, &plan).await;
}

#[tokio::test]
async fn bible_change_during_actual_http_refuses_proposal_persistence_and_all_history_writes() {
    let fixture = fixture().await;
    let planning = planning(&fixture).await;
    assert!(
        planning.request["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains(BLUE)
    );
    field(&fixture, "Mara", "tagline", Some(GOLD)).await;
    let before = rows(&fixture);
    let saved = script(&fixture);
    let error = finish(planning).await.unwrap_err();
    assert!(error.message().contains(REFUSAL));
    assert_rows_unchanged(&fixture, &before);
    assert_eq!(script(&fixture), saved);
    assert!(
        crate::projection_service::child_plan_list_projection(&fixture.state)
            .await
            .unwrap()
            .payload
            .plans
            .is_empty()
    );
}

#[tokio::test]
async fn recovered_pending_plan_exposes_original_bible_and_screenplay_evidence_after_manual_fact_edit()
 {
    let mut fixture = fixture().await;
    let original = pending(&fixture).await;
    field(&fixture, "Mara", "tagline", Some(GOLD)).await;
    let before = rows(&fixture);
    fixture.state.shutdown_tasks();
    let (project, _) = crate::persistence::load_project(&fixture.path)
        .await
        .unwrap();
    fixture.state = AppState::new().await;
    crate::project_service::replace_active_project(&fixture.state, project, fixture.path.clone());
    let recovered = crate::projection_service::child_plan_list_projection(&fixture.state)
        .await
        .unwrap();
    let record = recovered
        .payload
        .plans
        .iter()
        .find(|record| record.plan.id == original.id)
        .unwrap();
    assert_eq!(record.status, ChildPlanStatus::Pending);
    assert_eq!(
        serde_json::to_value(&record.plan).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    assert_eq!(
        record.plan.bible_context.as_ref().unwrap().inputs[0].value,
        FieldValue::Text(BLUE.into())
    );
    assert_eq!(record.plan.script_context.as_ref().unwrap()[0].text, SCRIPT);
    assert_rows_unchanged(&fixture, &before);
    refuse_unchanged(&fixture, &record.plan).await;
}

#[tokio::test]
async fn selected_bible_node_edge_and_snapshot_edits_and_aba_refuse_pending_acceptance() {
    for mutation in [
        "rename",
        "rename_aba",
        "edge",
        "edge_aba",
        "edge_delete_recreate",
        "snapshot",
        "snapshot_aba",
        "null_field",
    ] {
        let fixture = fixture().await;
        if mutation.starts_with("snapshot") {
            snapshot_field(&fixture, "Mara", "waiting").await;
        }
        if mutation == "null_field" {
            field(&fixture, "Mara", "motivation", None).await;
        }
        let plan = pending(&fixture).await;
        match mutation {
            "rename" => rename(&fixture, "Mara changed").await,
            "rename_aba" => {
                rename(&fixture, "Mara changed").await;
                rename(&fixture, "Mara").await;
            }
            "edge" => edge(&fixture, "Mara.Eli", "Mara", "Eli", "leaves Eli").await,
            "edge_aba" => {
                edge(&fixture, "Mara.Eli", "Mara", "Eli", "leaves Eli").await;
                edge(&fixture, "Mara.Eli", "Mara", "Eli", "waiting for Eli").await;
            }
            "edge_delete_recreate" => {
                crate::command_service::delete_bible_graph_edge(
                    &fixture.state,
                    CommandEnvelope::new(DeleteBibleGraphEdgeCommand {
                        edge_id: BibleGraphEdgeId::new("Mara.Eli").unwrap(),
                    }),
                )
                .await
                .unwrap();
                edge(&fixture, "Mara.Eli", "Mara", "Eli", "waiting for Eli").await;
            }
            "snapshot" => snapshot_field(&fixture, "Mara", "departed").await,
            "snapshot_aba" => {
                snapshot_field(&fixture, "Mara", "departed").await;
                snapshot_field(&fixture, "Mara", "waiting").await;
            }
            "null_field" => {
                field(&fixture, "Mara", "motivation", Some("Reach the last train")).await
            }
            _ => unreachable!(),
        }
        refuse_unchanged(&fixture, &plan).await;
    }
}

#[tokio::test]
async fn unselected_bible_field_snapshot_and_edge_changes_allow_original_reviewed_plan() {
    let fixture = fixture().await;
    fill_bible_cap(&fixture).await;
    node(&fixture, "OutsideA", "Outside A", 900).await;
    node(&fixture, "OutsideB", "Outside B", 901).await;
    field(
        &fixture,
        "OutsideA",
        "tagline",
        Some("Unselected initial fact"),
    )
    .await;
    snapshot_field(&fixture, "OutsideA", "waiting").await;
    edge(
        &fixture,
        "outside.edge",
        "OutsideA",
        "OutsideB",
        "unselected relationship",
    )
    .await;
    // A relationship that exited the prompt before generation retains its
    // historical membership custody. A later outside-only label edit is safe.
    edge(
        &fixture,
        "old.selected.edge",
        "Mara",
        "Eli",
        "previously selected",
    )
    .await;
    edge(
        &fixture,
        "old.selected.edge",
        "Mara",
        "OutsideA",
        "already outside",
    )
    .await;
    let plan = pending(&fixture).await;
    let context = &plan.bible_context.as_ref().unwrap().context.payload;
    assert!(
        context
            .nodes
            .iter()
            .all(|node| !node.node_id.as_str().starts_with("Outside"))
    );
    field(
        &fixture,
        "OutsideA",
        "tagline",
        Some("Unselected changed fact"),
    )
    .await;
    snapshot_field(&fixture, "OutsideA", "departed").await;
    edge(
        &fixture,
        "outside.edge",
        "OutsideA",
        "OutsideB",
        "unselected changed relationship",
    )
    .await;
    edge(
        &fixture,
        "old.selected.edge",
        "Mara",
        "OutsideA",
        "outside label edited",
    )
    .await;
    let saved = script(&fixture);
    assert_eq!(
        apply(&fixture, command(&plan)).await.unwrap()["outcome"],
        "recorded"
    );
    assert_eq!(script(&fixture), saved);
}

#[tokio::test]
async fn canonical_and_sparse_endpoint_history_aba_refuse_even_when_current_prompt_is_identical() {
    for sparse_history in [false, true] {
        let fixture = fixture().await;
        fill_bible_cap(&fixture).await;
        node(&fixture, "Outside", "Outside", 900).await;
        edge(
            &fixture,
            "hidden.edge",
            "Mara",
            "Outside",
            "hidden relationship",
        )
        .await;
        let plan = pending(&fixture).await;
        let original = plan.bible_context.as_ref().unwrap().context.payload.clone();
        assert!(original.nodes.iter().all(|node| {
            node.incoming_edges
                .iter()
                .chain(&node.outgoing_edges)
                .all(|edge| edge.edge_id.as_str() != "hidden.edge")
        }));
        for to in ["Eli", "Outside"] {
            if sparse_history {
                // The public generic-field command owns sparse revision
                // projection, not current Bible-edge storage. This proves
                // historical selection admission without claiming a GUI edit.
                crate::command_service::set_object_field(
                    &fixture.state,
                    CommandEnvelope::new(SetObjectFieldCommand {
                        object_kind: ObjectKind::BibleEdge,
                        object_id: "hidden.edge".into(),
                        field_key: "to_node_id".into(),
                        value: Some(FieldValue::ObjectRef {
                            kind: ObjectKind::BibleNode,
                            id: to.into(),
                        }),
                    }),
                )
                .await
                .unwrap();
            } else {
                edge(&fixture, "hidden.edge", "Mara", to, "hidden relationship").await;
            }
        }
        assert_eq!(current_bible(&fixture), original);
        refuse_unchanged(&fixture, &plan).await;
    }
}

async fn select_context(fixture: &Fixture, owner: &str) {
    let id = ContextEvaluationId::new();
    let created_at_ms = rows(fixture)["change_events"].len() as u64 + 100;
    crate::context_influence_service::record_context_evaluation(
        &fixture.state,
        CommandEnvelope::new(RecordContextEvaluationCommand {
            evaluation: ContextEvaluation {
                id,
                target_node_id: fixture.parent,
                task_kind: ContextEvaluationTaskKind::GenerateScript,
                summary: "Explicit fixture Bible selection".into(),
                distilled_context: None,
                created_at_ms,
            },
            influences: vec![ContextInfluenceRecord {
                id: ContextInfluenceId::new(),
                evaluation_id: id,
                timeline_node_id: fixture.parent,
                source_layer: StoryLevel::Scene,
                influence_kind: ContextInfluenceKind::Direct,
                confidence: 1.0,
                reason: "Manual fixture assignment".into(),
                provenance: ContextInfluenceProvenance::UserSelected,
                bible_node_id: Some(BibleGraphNodeId::new(owner).unwrap()),
                bible_edge_id: None,
                introduced_by_node_id: None,
                sort_order: 0,
            }],
        }),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn explicit_context_selection_aba_refuses_with_original_bible_payload_restored() {
    let fixture = fixture().await;
    select_context(&fixture, "Mara").await;
    let plan = pending(&fixture).await;
    select_context(&fixture, "Eli").await;
    select_context(&fixture, "Mara").await;
    assert_eq!(
        current_bible(&fixture),
        plan.bible_context.as_ref().unwrap().context.payload
    );
    refuse_unchanged(&fixture, &plan).await;
}

#[tokio::test]
async fn capped_bible_node_entry_and_removal_aba_refuse_restored_selection() {
    let fixture = fixture().await;
    fill_bible_cap(&fixture).await;
    let plan = pending(&fixture).await;
    let original = plan.bible_context.as_ref().unwrap().context.payload.clone();
    assert_eq!(original.nodes.len(), 200);
    node(&fixture, "Entry", "Entry", 0).await;
    assert!(
        current_bible(&fixture)
            .nodes
            .iter()
            .any(|node| node.node_id.as_str() == "Entry")
    );
    crate::command_service::delete_bible_graph_node(
        &fixture.state,
        CommandEnvelope::new(DeleteBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("Entry").unwrap(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(current_bible(&fixture), original);
    refuse_unchanged(&fixture, &plan).await;
}

#[tokio::test]
async fn explicit_story_time_is_recovered_and_reused_at_acceptance_and_new_equal_time_conflict_refuses()
 {
    for conflict in [false, true] {
        let mut fixture = fixture().await;
        snapshot_field(&fixture, "Mara", GOLD).await;
        let saved = script(&fixture);
        let request = planning_at(&fixture, Some(1000)).await;
        let prompt = request.request["messages"][1]["content"].as_str().unwrap();
        assert!(prompt.contains("Field values resolved at fictional story time 1000ms"));
        assert!(prompt.contains(GOLD));
        assert!(!prompt.contains(&format!("profile.tagline: {BLUE}")));
        let plan = finish(request).await.unwrap();
        let evidence = plan.bible_context.as_ref().unwrap();
        assert_eq!(evidence.context.payload.story_time_ms, Some(1000));
        assert!(
            evidence.inputs.is_empty(),
            "a resolved timed assertion must never rebind the unused baseline fact"
        );
        let mara = evidence
            .context
            .payload
            .nodes
            .iter()
            .find(|node| node.node_id.as_str() == "Mara")
            .unwrap();
        assert_eq!(mara.snapshots[0].at_ms, 1000);
        assert_eq!(
            mara.snapshots[0].fields[0].value,
            FieldValue::Text(GOLD.into())
        );

        let before = rows(&fixture);
        fixture.state.shutdown_tasks();
        let (project, _) = crate::persistence::load_project(&fixture.path)
            .await
            .unwrap();
        fixture.state = AppState::new().await;
        crate::project_service::replace_active_project(
            &fixture.state,
            project,
            fixture.path.clone(),
        );
        let recovered = crate::projection_service::child_plan_list_projection(&fixture.state)
            .await
            .unwrap();
        let restored = &recovered
            .payload
            .plans
            .iter()
            .find(|record| record.plan.id == plan.id)
            .unwrap()
            .plan;
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(&plan).unwrap()
        );
        assert_rows_unchanged(&fixture, &before);
        if conflict {
            crate::command_service::set_bible_graph_snapshot_field(&fixture.state,
                serde_json::from_value(serde_json::json!({"id":Uuid::new_v4(), "payload": {
                    "snapshot_id":"Mara.conflict", "node_id":"Mara", "at_ms":1000,
                    "label":"Conflicting same-time assertion", "snapshot_sort_order":1,
                    "field_id":"Mara.conflict.tagline", "part_key":"profile", "part_name":"Profile",
                    "field_key":"tagline", "value":{"type":"text","value":BLUE}, "field_sort_order":0
                }})).unwrap()).await.unwrap();
            refuse_unchanged(&fixture, restored).await;
        } else {
            // Recapturing at None (or narrative placement) would withhold this
            // assertion and refuse. Success proves the original coordinate.
            assert_eq!(
                apply(&fixture, command(restored)).await.unwrap()["outcome"],
                "recorded"
            );
            assert_eq!(script(&fixture), saved);
        }
    }
}

#[tokio::test]
async fn legacy_missing_bible_or_whole_memory_receipts_refuse_without_mutating_saved_state() {
    for missing in ["bible", "memory"] {
        let fixture = fixture().await;
        let plan = pending(&fixture).await;
        let conn = crate::sqlite::open_write_connection(&fixture.path).unwrap();
        // Legacy-fixture setup only: emulate exact old creation JSON while
        // preserving all canonical data, command IDs and revision identities.
        let raw: String = conn.query_row("SELECT c.payload_json FROM child_plans p JOIN change_events e ON e.id=p.created_event_id JOIN commands c ON c.id=e.command_id WHERE p.id=?1", [plan.id.as_str()], |row|row.get(0)).unwrap();
        let mut old: serde_json::Value = serde_json::from_str(&raw).unwrap();
        if missing == "bible" {
            old["memory"].as_object_mut().unwrap().remove("bible");
        } else {
            old.as_object_mut().unwrap().remove("memory");
        }
        conn.execute("UPDATE commands SET payload_json=?1 WHERE id=(SELECT e.command_id FROM child_plans p JOIN change_events e ON e.id=p.created_event_id WHERE p.id=?2)", rusqlite::params![old.to_string(),plan.id.as_str()]).unwrap();
        refuse_unchanged(&fixture, &plan).await;
    }
}
