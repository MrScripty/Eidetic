use super::*;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId,
    BibleGraphPartId, BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId,
    BibleGraphSnapshotId, CommandEnvelope, CommandId, CreateStoryArcCommand,
    EnsureCanonicalBibleRootsCommand, FieldValue, ObjectKind, ScriptBlockId, ScriptBlockKind,
    ScriptDocumentId, ScriptLockId, ScriptSegmentId, ScriptSegmentStatus, ScriptSpanId,
};
use eidetic_core::story::arc::{ArcId, ArcType, Color};
use serde_json::json;
use tower::util::ServiceExt;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

#[tokio::test]
async fn object_field_command_requires_loaded_project() {
    let app = router().with_state(AppState::new().await);
    let body = object_field_command_body("field-weather", "weather", Some(json_text("rainy")));

    let response = app
        .oneshot(command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn object_field_command_returns_projection() {
    let path = temp_db_path("returns-projection");
    let app = app_with_project_path(path.clone()).await;
    let body = object_field_command_body("field-weather", "weather", Some(json_text("rainy")));

    let response = app
        .oneshot(command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert_eq!(
        value["projection"]["payload"]["fields"]["weather"]["value"],
        "rainy"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_command_rejects_empty_field_key() {
    let path = temp_db_path("rejects-empty-field");
    let app = app_with_project_path(path.clone()).await;
    let body = object_field_command_body("field-weather", "", Some(json_text("rainy")));

    let response = app
        .oneshot(command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_command_replays_duplicate_command() {
    let path = temp_db_path("replays-duplicate");
    let app = app_with_project_path(path.clone()).await;
    let command_id = uuid::Uuid::new_v4();
    let body = object_field_command_body_with_id(
        command_id,
        "field-weather",
        "weather",
        Some(json_text("rainy")),
    );

    let first = app
        .clone()
        .oneshot(command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    assert_eq!(value["projection"]["version"], 2);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_returns_arc_list_projection() {
    let path = temp_db_path("story-arc-create");
    let app = app_with_project_path(path.clone()).await;
    let arc_id = ArcId::new();
    let body = serde_json::to_value(CommandEnvelope {
        id: CommandId::new(),
        payload: CreateStoryArcCommand {
            arc_id,
            parent_arc_id: None,
            name: "Mystery".to_string(),
            description: "Central investigation".to_string(),
            arc_type: ArcType::APlot,
            color: Color::A_PLOT,
        },
    })
    .unwrap();

    let response = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 1);
    assert!(
        value["projection"]["payload"]["arcs"]
            .as_array()
            .expect("arcs")
            .iter()
            .any(|arc| arc["id"] == serde_json::json!(arc_id)
                && arc["name"] == serde_json::json!("Mystery"))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("rejects-conflicting-duplicate");
    let app = app_with_project_path(path.clone()).await;
    let command_id = uuid::Uuid::new_v4();
    let original = object_field_command_body_with_id(
        command_id,
        "field-weather",
        "weather",
        Some(json_text("rainy")),
    );
    let conflicting = object_field_command_body_with_id(
        command_id,
        "field-weather",
        "weather",
        Some(json_text("sunny")),
    );

    let first = app
        .clone()
        .oneshot(command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_command_returns_projection() {
    let path = temp_db_path("creates-bible-graph-node");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_graph_node_command_body("node.character.ada", "Ada");

    let response = app
        .oneshot(bible_graph_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert_eq!(
        value["projection"]["payload"]["node"]["id"],
        "node.character.ada"
    );
    assert_eq!(value["projection"]["payload"]["node"]["name"], "Ada");
    assert_eq!(
        value["projection"]["payload"]["parts"][0]["part"]["part_key"],
        "profile"
    );
    assert_eq!(
        value["projection"]["payload"]["parts"][0]["fields"][1]["field_key"],
        "tagline"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_command_rejects_empty_name() {
    let path = temp_db_path("rejects-empty-bible-node-name");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_graph_node_command_body("node.character.ada", " ");

    let response = app
        .oneshot(bible_graph_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_command_rejects_missing_parent() {
    let path = temp_db_path("rejects-missing-bible-parent");
    let app = app_with_project_path(path.clone()).await;
    let body = bible_graph_node_command_body_with_parent(
        "node.character.ada",
        Some("node.group.missing"),
        "Ada",
    );

    let response = app
        .oneshot(bible_graph_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_roots_command_returns_node_list_projection() {
    let path = temp_db_path("ensures-bible-roots");
    let app = app_with_project_path(path.clone()).await;
    let body = json!({
        "id": uuid::Uuid::new_v4(),
        "payload": EnsureCanonicalBibleRootsCommand {},
    });

    let response = app
        .oneshot(bible_graph_roots_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 9);
    assert_eq!(
        value["projection"]["payload"]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(
        value["projection"]["payload"]["nodes"][0]["id"],
        "canonical.characters"
    );
    assert_eq!(
        value["projection"]["payload"]["nodes"][0]["system_owned"],
        true
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_field_command_returns_populated_node_detail_projection() {
    let path = temp_db_path("sets-bible-graph-field");
    let app = app_with_project_path(path.clone()).await;
    let node = bible_graph_node_command_body("node.character.ada", "Ada");
    let field = bible_graph_field_command_body(Some(json_text("Reluctant detective")));

    let create_response = app
        .clone()
        .oneshot(bible_graph_command_request(node))
        .await
        .expect("create route response");
    let field_response = app
        .oneshot(bible_graph_field_command_request(field))
        .await
        .expect("field route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(field_response.status(), StatusCode::OK);
    let value = response_json(field_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 3);
    assert_eq!(
        value["projection"]["payload"]["parts"][0]["fields"][0]["value"]["value"],
        "Reluctant detective"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_edge_command_returns_edge_projection() {
    let path = temp_db_path("sets-bible-graph-edge");
    let app = app_with_project_path(path.clone()).await;
    let source = bible_graph_node_command_body("node.character.ada", "Ada");
    let target = bible_graph_node_command_body("node.place.beach", "Beach");
    let edge = bible_graph_edge_command_body();

    let source_response = app
        .clone()
        .oneshot(bible_graph_command_request(source))
        .await
        .expect("source route response");
    let target_response = app
        .clone()
        .oneshot(bible_graph_command_request(target))
        .await
        .expect("target route response");
    let edge_response = app
        .oneshot(bible_graph_edge_command_request(edge))
        .await
        .expect("edge route response");

    assert_eq!(source_response.status(), StatusCode::OK);
    assert_eq!(target_response.status(), StatusCode::OK);
    assert_eq!(edge_response.status(), StatusCode::OK);
    let value = response_json(edge_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(
        value["projection"]["payload"]["outgoing_edges"][0]["id"],
        "edge.ada.beach"
    );
    assert_eq!(
        value["projection"]["payload"]["outgoing_edges"][0]["to_node_id"],
        "node.place.beach"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_edge_command_rejects_missing_target() {
    let path = temp_db_path("rejects-missing-edge-target");
    let app = app_with_project_path(path.clone()).await;
    let source = bible_graph_node_command_body("node.character.ada", "Ada");
    let edge = bible_graph_edge_command_body();

    let source_response = app
        .clone()
        .oneshot(bible_graph_command_request(source))
        .await
        .expect("source route response");
    let edge_response = app
        .oneshot(bible_graph_edge_command_request(edge))
        .await
        .expect("edge route response");

    assert_eq!(source_response.status(), StatusCode::OK);
    assert_eq!(edge_response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_snapshot_field_command_returns_snapshot_projection() {
    let path = temp_db_path("sets-bible-graph-snapshot-field");
    let app = app_with_project_path(path.clone()).await;
    let node = bible_graph_node_command_body("node.character.ada", "Ada");
    let snapshot = bible_graph_snapshot_field_command_body(Some(json_text("Rain-soaked")));

    let create_response = app
        .clone()
        .oneshot(bible_graph_command_request(node))
        .await
        .expect("create route response");
    let snapshot_response = app
        .oneshot(bible_graph_snapshot_field_command_request(snapshot))
        .await
        .expect("snapshot route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(snapshot_response.status(), StatusCode::OK);
    let value = response_json(snapshot_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 3);
    assert_eq!(
        value["projection"]["payload"]["snapshots"][0]["snapshot"]["label"],
        "Sequence 1 state"
    );
    assert_eq!(
        value["projection"]["payload"]["snapshots"][0]["fields"][0]["value"]["value"],
        "Rain-soaked"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_snapshot_field_command_rejects_blank_label() {
    let path = temp_db_path("rejects-blank-snapshot-label");
    let app = app_with_project_path(path.clone()).await;
    let node = bible_graph_node_command_body("node.character.ada", "Ada");
    let mut snapshot = bible_graph_snapshot_field_command_body(Some(json_text("Rain-soaked")));
    snapshot["payload"]["label"] = json!(" ");

    let create_response = app
        .clone()
        .oneshot(bible_graph_command_request(node))
        .await
        .expect("create route response");
    let snapshot_response = app
        .oneshot(bible_graph_snapshot_field_command_request(snapshot))
        .await
        .expect("snapshot route response");

    assert_eq!(create_response.status(), StatusCode::OK);
    assert_eq!(snapshot_response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_block_command_returns_script_document_projection() {
    let path = temp_db_path("sets-script-block");
    let app = app_with_project_path(path.clone()).await;
    let body = script_block_command_body("Ada enters with a wet umbrella.");

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 5);
    assert_eq!(value["projection"]["payload"]["document"]["title"], "Pilot");
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["block"]["text"],
        "Ada enters with a wet umbrella."
    );
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["spans"][0]["end_byte"],
        31
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_block_command_rejects_invalid_segment_range() {
    let path = temp_db_path("rejects-invalid-script-range");
    let app = app_with_project_path(path.clone()).await;
    let mut body = script_block_command_body("Ada enters with a wet umbrella.");
    body["payload"]["segment_start_ms"] = json!(5_000);
    body["payload"]["segment_end_ms"] = json!(1_000);

    let response = app
        .oneshot(script_block_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_lock_command_returns_script_document_projection() {
    let path = temp_db_path("sets-script-lock");
    let app = app_with_project_path(path.clone()).await;
    let block = script_block_command_body("Ada enters with a wet umbrella.");
    let lock = script_lock_command_body("User approved wording.");

    let block_response = app
        .clone()
        .oneshot(script_block_command_request(block))
        .await
        .expect("block route response");
    let lock_response = app
        .oneshot(script_lock_command_request(lock))
        .await
        .expect("lock route response");

    assert_eq!(block_response.status(), StatusCode::OK);
    assert_eq!(lock_response.status(), StatusCode::OK);
    let value = response_json(lock_response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 6);
    assert_eq!(
        value["projection"]["payload"]["segments"][0]["blocks"][0]["locks"][0]["reason"],
        "User approved wording."
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_lock_command_rejects_missing_span() {
    let path = temp_db_path("rejects-missing-script-span");
    let app = app_with_project_path(path.clone()).await;
    let body = script_lock_command_body("User approved wording.");

    let response = app
        .oneshot(script_lock_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_range_command_returns_timeline_render_projection() {
    let path = temp_db_path("sets-timeline-node-range");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = timeline_node_range_command_body(node_id, 1_000, 2_000);

    let response = app
        .oneshot(timeline_node_range_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 1);
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    let clip = clips
        .iter()
        .find(|clip| clip["node_id"] == node_id.0.to_string())
        .expect("updated node clip");
    assert_eq!(clip["start_ms"], 1_000);
    assert_eq!(clip["end_ms"], 2_000);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_range_command_rejects_invalid_range() {
    let path = temp_db_path("rejects-invalid-timeline-range");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = timeline_node_range_command_body(node_id, 2_000, 1_000);

    let response = app
        .oneshot(timeline_node_range_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_range_command_requires_loaded_project() {
    let app = router().with_state(AppState::new().await);
    let body = timeline_node_range_command_body(
        eidetic_core::timeline::node::NodeId(uuid::Uuid::new_v4()),
        1_000,
        2_000,
    );

    let response = app
        .oneshot(timeline_node_range_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_timeline_node_command_returns_timeline_render_projection() {
    let path = temp_db_path("creates-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let node_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_node_command_body(
        node_id,
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Inserted act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
    );

    let response = app
        .oneshot(create_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == node_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "Inserted act"
    }));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_node_command_rejects_invalid_hierarchy() {
    let path = temp_db_path("rejects-invalid-timeline-create");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_node_command_body(
        eidetic_core::timeline::node::NodeId::new(),
        None,
        eidetic_core::timeline::node::StoryLevel::Scene,
        "Parentless scene",
        1_000,
        2_000,
    );

    let response = app
        .oneshot(create_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_returns_timeline_render_projection() {
    let path = temp_db_path("applies-timeline-children");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let original_child = project
        .timeline
        .children_of(parent.id)
        .first()
        .expect("original child")
        .id;
    let first_child_id = eidetic_core::timeline::node::NodeId::new();
    let second_child_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = apply_timeline_children_command_body(parent.id, first_child_id, second_child_id);

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != original_child.0.to_string())
    );
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == first_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "First child"
    }));
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == second_child_id.0.to_string()
            && clip["parent_id"] == parent.id.0.to_string()
            && clip["name"] == "Second child"
    }));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn apply_timeline_children_command_rejects_unknown_parent() {
    let path = temp_db_path("rejects-unknown-timeline-children-parent");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = apply_timeline_children_command_body(
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );

    let response = app
        .oneshot(apply_timeline_children_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_relationship_command_returns_timeline_render_projection() {
    let path = temp_db_path("creates-timeline-relationship");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_relationship_command_body(relationship_id, from_node, to_node);

    let response = app
        .oneshot(create_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert!(relationships.iter().any(|relationship| {
        relationship["relationship_id"] == relationship_id.0.to_string()
            && relationship["from_node_id"] == from_node.0.to_string()
            && relationship["to_node_id"] == to_node.0.to_string()
    }));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_relationship_command_rejects_unknown_endpoint() {
    let path = temp_db_path("rejects-unknown-timeline-relationship-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let to_node = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_relationship_command_body(
        eidetic_core::timeline::relationship::RelationshipId::new(),
        eidetic_core::timeline::node::NodeId::new(),
        to_node,
    );

    let response = app
        .oneshot(create_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_returns_timeline_render_projection() {
    let path = temp_db_path("deletes-timeline-relationship");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let from_node = project.timeline.nodes[0].id;
    let to_node = project.timeline.nodes[1].id;
    let mut relationship = eidetic_core::timeline::relationship::Relationship::new(
        from_node,
        to_node,
        eidetic_core::timeline::relationship::RelationshipType::Thematic,
    );
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    relationship.id = relationship_id;
    project.timeline.add_relationship(relationship).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_relationship_command_body(relationship_id);

    let response = app
        .oneshot(delete_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let relationships = value["projection"]["payload"]["relationships"]
        .as_array()
        .expect("timeline relationships");
    assert!(
        relationships
            .iter()
            .all(|relationship| relationship["relationship_id"] != relationship_id.0.to_string())
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_relationship_command_rejects_unknown_relationship() {
    let path = temp_db_path("rejects-unknown-timeline-relationship-delete");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_relationship_command_body(
        eidetic_core::timeline::relationship::RelationshipId::new(),
    );

    let response = app
        .oneshot(delete_timeline_relationship_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_returns_timeline_render_projection() {
    let path = temp_db_path("splits-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body(node.id, split_ms);

    let response = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != node.id.0.to_string())
    );
    assert!(clips.iter().any(|clip| {
        clip["start_ms"] == node.time_range.start_ms && clip["end_ms"] == split_ms
    }));
    assert!(
        clips.iter().any(|clip| {
            clip["start_ms"] == split_ms && clip["end_ms"] == node.time_range.end_ms
        })
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_rejects_out_of_range_split() {
    let path = temp_db_path("rejects-invalid-timeline-split");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body(node.id, node.time_range.start_ms);

    let response = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_lock_command_returns_timeline_render_projection() {
    let path = temp_db_path("locks-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = timeline_node_lock_command_body(node.id, true);

    let response = app
        .oneshot(timeline_node_lock_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    let clip = clips
        .iter()
        .find(|clip| clip["node_id"] == node.id.0.to_string())
        .expect("locked clip");
    assert_eq!(clip["locked"], true);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_lock_command_rejects_unknown_node() {
    let path = temp_db_path("rejects-unknown-timeline-lock");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = timeline_node_lock_command_body(eidetic_core::timeline::node::NodeId::new(), true);

    let response = app
        .oneshot(timeline_node_lock_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_notes_command_returns_timeline_render_projection() {
    let path = temp_db_path("sets-timeline-node-notes");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    project.timeline.node_mut(node.id).unwrap().content.status =
        eidetic_core::timeline::node::ContentStatus::Empty;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = timeline_node_notes_command_body(node.id, "New outline");

    let response = app
        .oneshot(timeline_node_notes_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    let clip = clips
        .iter()
        .find(|clip| clip["node_id"] == node.id.0.to_string())
        .expect("notes clip");
    assert_eq!(clip["content_status"], "NotesOnly");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_notes_command_rejects_unknown_node() {
    let path = temp_db_path("rejects-unknown-timeline-notes");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body =
        timeline_node_notes_command_body(eidetic_core::timeline::node::NodeId::new(), "Notes");

    let response = app
        .oneshot(timeline_node_notes_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_node_command_returns_timeline_render_projection() {
    let path = temp_db_path("deletes-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_node_command_body(node.id);

    let response = app
        .oneshot(delete_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != node.id.0.to_string())
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_node_command_rejects_unknown_node() {
    let path = temp_db_path("rejects-unknown-timeline-delete");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_node_command_body(eidetic_core::timeline::node::NodeId(
        uuid::Uuid::new_v4(),
    ));

    let response = app
        .oneshot(delete_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

fn command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/object-field")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_graph_field_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/field")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_graph_edge_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/edge")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_graph_snapshot_field_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/snapshot-field")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_graph_roots_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/canonical-roots")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn script_block_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/script/block")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn script_lock_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/script/lock")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn story_arc_create_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/story/create-arc")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn timeline_node_range_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/node-range")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/create-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn apply_timeline_children_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/apply-children")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_timeline_relationship_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/create-relationship")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn delete_timeline_relationship_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/delete-relationship")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn split_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/split-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn timeline_node_lock_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/node-lock")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn timeline_node_notes_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/node-notes")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn delete_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/delete-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn bible_graph_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn object_field_command_body(
    object_id: &str,
    field_key: &str,
    value: Option<serde_json::Value>,
) -> serde_json::Value {
    object_field_command_body_with_id(uuid::Uuid::new_v4(), object_id, field_key, value)
}

fn object_field_command_body_with_id(
    command_id: uuid::Uuid,
    object_id: &str,
    field_key: &str,
    value: Option<serde_json::Value>,
) -> serde_json::Value {
    json!({
        "id": command_id,
        "payload": {
            "object_kind": ObjectKind::BiblePartField,
            "object_id": object_id,
            "field_key": field_key,
            "value": value,
        }
    })
}

fn json_text(value: &str) -> serde_json::Value {
    serde_json::to_value(FieldValue::Text(value.to_string())).unwrap()
}

fn bible_graph_node_command_body(node_id: &str, name: &str) -> serde_json::Value {
    bible_graph_node_command_body_with_parent(node_id, None, name)
}

fn bible_graph_node_command_body_with_parent(
    node_id: &str,
    parent_id: Option<&str>,
    name: &str,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": BibleGraphNodeId::new(node_id).unwrap(),
            "parent_id": parent_id.map(|value| BibleGraphNodeId::new(value).unwrap()),
            "schema_key": BibleGraphSchemaKey::new("character").unwrap(),
            "name": name,
            "sort_order": 3,
        }
    })
}

fn bible_graph_field_command_body(value: Option<serde_json::Value>) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": BibleGraphNodeId::new("node.character.ada").unwrap(),
            "part_id": BibleGraphPartId::new("part.character.profile").unwrap(),
            "part_key": BibleGraphPartKey::new("profile").unwrap(),
            "part_name": "Profile",
            "part_sort_order": 1,
            "field_id": BibleGraphFieldId::new("field.character.tagline").unwrap(),
            "field_key": BibleGraphFieldKey::new("tagline").unwrap(),
            "value": value,
            "field_sort_order": 2,
        }
    })
}

fn bible_graph_edge_command_body() -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "edge_id": BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
            "from_node_id": BibleGraphNodeId::new("node.character.ada").unwrap(),
            "to_node_id": BibleGraphNodeId::new("node.place.beach").unwrap(),
            "edge_kind": BibleGraphEdgeKind::LocatedIn,
            "label": "located in",
            "directed": true,
            "sort_order": 1,
        }
    })
}

fn bible_graph_snapshot_field_command_body(value: Option<serde_json::Value>) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "snapshot_id": BibleGraphSnapshotId::new("snapshot.character.ada.sequence-1").unwrap(),
            "node_id": BibleGraphNodeId::new("node.character.ada").unwrap(),
            "at_ms": 12_000,
            "label": "Sequence 1 state",
            "snapshot_sort_order": 1,
            "field_id": BibleGraphSnapshotFieldId::new("snapshot-field.character.status").unwrap(),
            "part_key": BibleGraphPartKey::new("profile").unwrap(),
            "part_name": "Profile",
            "field_key": BibleGraphFieldKey::new("tagline").unwrap(),
            "value": value,
            "field_sort_order": 2,
        }
    })
}

fn script_block_command_body(text: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "document_id": ScriptDocumentId::new("script.document.main").unwrap(),
            "document_title": "Pilot",
            "document_sort_order": 0,
            "segment_id": ScriptSegmentId::new("script.segment.beat-1").unwrap(),
            "source_node_id": "node.beat.opening",
            "segment_start_ms": 1_000,
            "segment_end_ms": 5_000,
            "segment_status": ScriptSegmentStatus::Current,
            "segment_sort_order": 1,
            "block_id": ScriptBlockId::new("script.block.action-1").unwrap(),
            "block_kind": ScriptBlockKind::Action,
            "text": text,
            "span_provenance": "user_edited",
            "sort_order": 2,
        }
    })
}

fn script_lock_command_body(reason: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "lock_id": ScriptLockId::new("script.lock.action-1").unwrap(),
            "span_id": ScriptSpanId::new("script.block.action-1.span.main").unwrap(),
            "reason": reason,
        }
    })
}

fn timeline_node_range_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    start_ms: u64,
    end_ms: u64,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
            "start_ms": start_ms,
            "end_ms": end_ms,
        }
    })
}

fn create_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    parent_id: Option<eidetic_core::timeline::node::NodeId>,
    level: eidetic_core::timeline::node::StoryLevel,
    name: &str,
    start_ms: u64,
    end_ms: u64,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
            "parent_id": parent_id,
            "level": level,
            "name": name,
            "start_ms": start_ms,
            "end_ms": end_ms,
            "beat_type": null,
        }
    })
}

fn apply_timeline_children_command_body(
    parent_id: eidetic_core::timeline::node::NodeId,
    first_child_id: eidetic_core::timeline::node::NodeId,
    second_child_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "parent_id": parent_id,
            "children": [
                {
                    "node_id": first_child_id,
                    "name": "First child",
                    "outline": "First outline",
                    "weight": 1.0,
                    "beat_type": null,
                },
                {
                    "node_id": second_child_id,
                    "name": "Second child",
                    "outline": "Second outline",
                    "weight": 1.0,
                    "beat_type": null,
                }
            ]
        }
    })
}

fn create_timeline_relationship_command_body(
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
    from_node_id: eidetic_core::timeline::node::NodeId,
    to_node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "relationship_id": relationship_id,
            "from_node_id": from_node_id,
            "to_node_id": to_node_id,
            "relationship_type": eidetic_core::timeline::relationship::RelationshipType::Thematic,
        }
    })
}

fn delete_timeline_relationship_command_body(
    relationship_id: eidetic_core::timeline::relationship::RelationshipId,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "relationship_id": relationship_id,
        }
    })
}

fn split_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    at_ms: u64,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
            "at_ms": at_ms,
        }
    })
}

fn timeline_node_lock_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    locked: bool,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
            "locked": locked,
        }
    })
}

fn timeline_node_notes_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    notes: &str,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
            "notes": notes,
        }
    })
}

fn delete_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node_id,
        }
    })
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}
