use super::*;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    CommandEnvelope, ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanProvenance, SetScriptBlockCommand,
};
use tower::util::ServiceExt;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Projection Test");
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

#[tokio::test]
async fn script_document_projection_returns_not_found_when_absent() {
    let path = temp_db_path("script-document-absent");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(script_document_projection_request("script.document.main"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_document_projection_returns_persisted_script_blocks() {
    let path = temp_db_path("script-document-persisted");
    let app = app_with_project_path(path.clone()).await;
    seed_script_block(&path, "Ada enters with a wet umbrella.");

    let response = app
        .oneshot(script_document_projection_request("script.document.main"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 5);
    assert_eq!(value["payload"]["document"]["title"], "Pilot");
    assert_eq!(
        value["payload"]["segments"][0]["segment"]["source_node_id"],
        "node.beat.opening"
    );
    assert_eq!(
        value["payload"]["segments"][0]["blocks"][0]["block"]["text"],
        "Ada enters with a wet umbrella."
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_render_projection_requires_loaded_project() {
    let app = router().with_state(AppState::new().await);

    let response = app
        .oneshot(timeline_render_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn timeline_render_projection_returns_backend_timeline_read_model() {
    let path = temp_db_path("timeline-render");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Projection Test");
    project.timeline.nodes.clear();
    project.timeline.node_arcs.clear();
    project.timeline.relationships.clear();
    let scene = eidetic_core::timeline::node::StoryNode::new(
        "SQLite beach argument",
        eidetic_core::timeline::node::StoryLevel::Scene,
        eidetic_core::timeline::timing::TimeRange::new(1_000, 4_000).unwrap(),
    );
    project.timeline.nodes.push(scene);
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed timeline projection");
    project.timeline.nodes[0].name = "Stale mirror argument".to_string();
    project.timeline.nodes[0].time_range =
        eidetic_core::timeline::timing::TimeRange::new(5_000, 6_000).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);

    let response = app
        .oneshot(timeline_render_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(value["payload"]["tracks"][0]["level"], "Premise");
    assert_eq!(
        value["payload"]["clips"][0]["name"],
        "SQLite beach argument"
    );
    assert_eq!(value["payload"]["clips"][0]["start_ms"], 1_000);
    assert_eq!(value["payload"]["clips"][0]["end_ms"], 4_000);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn selected_node_editor_projection_returns_empty_when_unselected() {
    let path = temp_db_path("selected-node-empty");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(selected_node_editor_projection_request(None))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert!(value["payload"]["node"].is_null());
    assert_eq!(value["payload"]["has_children"], false);
    assert_eq!(value["payload"]["children"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn selected_node_editor_projection_returns_backend_owned_context() {
    let path = temp_db_path("selected-node-context");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Projection Test");
    project.timeline.nodes.clear();
    project.timeline.node_arcs.clear();
    project.timeline.relationships.clear();

    let mut act_one = eidetic_core::timeline::node::StoryNode::new(
        "Act one",
        eidetic_core::timeline::node::StoryLevel::Act,
        eidetic_core::timeline::timing::TimeRange::new(0, 20_000).unwrap(),
    );
    act_one.sort_order = 1;
    let act_one_id = act_one.id;
    let mut act_two = eidetic_core::timeline::node::StoryNode::new(
        "Act two",
        eidetic_core::timeline::node::StoryLevel::Act,
        eidetic_core::timeline::timing::TimeRange::new(20_000, 40_000).unwrap(),
    );
    act_two.sort_order = 2;
    let mut sequence = eidetic_core::timeline::node::StoryNode::new(
        "SQLite selected sequence",
        eidetic_core::timeline::node::StoryLevel::Sequence,
        eidetic_core::timeline::timing::TimeRange::new(1_000, 10_000).unwrap(),
    );
    sequence.parent_id = Some(act_one_id);
    sequence.content.notes = "Persistent notes".to_string();
    sequence.content.status = eidetic_core::timeline::node::ContentStatus::NotesOnly;
    let sequence_id = sequence.id;
    let mut scene = eidetic_core::timeline::node::StoryNode::new(
        "Child scene",
        eidetic_core::timeline::node::StoryLevel::Scene,
        eidetic_core::timeline::timing::TimeRange::new(2_000, 4_000).unwrap(),
    );
    scene.parent_id = Some(sequence_id);
    project
        .timeline
        .nodes
        .extend([act_one, act_two, sequence, scene]);

    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed selected node projection");
    project.timeline.nodes[2].name = "Stale selected sequence".to_string();
    project.timeline.nodes[2].content.notes = "Stale notes".to_string();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);

    let response = app
        .oneshot(selected_node_editor_projection_request(Some(sequence_id)))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["payload"]["node"]["name"], "SQLite selected sequence");
    assert_eq!(value["payload"]["node"]["notes"], "Persistent notes");
    assert_eq!(value["payload"]["node"]["content_status"], "NotesOnly");
    assert_eq!(value["payload"]["child_level"], "Scene");
    assert_eq!(value["payload"]["has_children"], true);
    assert_eq!(value["payload"]["parent"]["name"], "Act one");
    assert_eq!(value["payload"]["current_sibling_index"], 0);
    assert_eq!(value["payload"]["children"][0]["name"], "Child scene");
    assert_eq!(
        value["payload"]["adjacent_parents"]["after"]["name"],
        "Act two"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn selected_node_editor_projection_returns_not_found_for_unknown_node() {
    let path = temp_db_path("selected-node-unknown");
    let app = app_with_project_path(path.clone()).await;
    let unknown_id = eidetic_core::timeline::node::NodeId(uuid::Uuid::new_v4());

    let response = app
        .oneshot(selected_node_editor_projection_request(Some(unknown_id)))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_list_projection_returns_project_arcs() {
    let path = temp_db_path("story-arc-list");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(story_arc_list_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(value["payload"]["arcs"].as_array().expect("arcs").len(), 3);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_progression_projection_returns_arc_analysis() {
    let path = temp_db_path("story-arc-progression");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(story_arc_progression_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(
        value["payload"]["progressions"]
            .as_array()
            .expect("progressions")
            .len(),
        3
    );
    assert!(
        value["payload"]["progressions"]
            .as_array()
            .expect("progressions")
            .iter()
            .all(|progression| progression.get("node_count").is_some())
    );

    let _ = std::fs::remove_file(path);
}

fn seed_script_block(path: &PathBuf, text: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: "Pilot".to_string(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        source_node_id: Some("node.beat.opening".to_string()),
        segment_start_ms: 1_000,
        segment_end_ms: 5_000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 1,
        block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: text.to_string(),
        span_provenance: ScriptSpanProvenance::UserEdited,
        sort_order: 2,
    });
    crate::script_document_command::apply_set_script_block(&mut conn, &command, 400).unwrap();
}

fn script_document_projection_request(document_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/projections/script/document?document_id={document_id}"
        ))
        .body(Body::empty())
        .unwrap()
}

fn timeline_render_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/timeline/render")
        .body(Body::empty())
        .unwrap()
}

fn selected_node_editor_projection_request(
    node_id: Option<eidetic_core::timeline::node::NodeId>,
) -> Request<Body> {
    let uri = node_id
        .map(|node_id| format!("/projections/timeline/selected-node?node_id={}", node_id.0))
        .unwrap_or_else(|| "/projections/timeline/selected-node".to_string());
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn story_arc_list_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/story/arcs")
        .body(Body::empty())
        .unwrap()
}

fn story_arc_progression_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/story/arc-progression")
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-projection-story-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}
