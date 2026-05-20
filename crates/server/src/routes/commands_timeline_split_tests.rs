use super::router;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{FieldValue, ObjectKind};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

#[tokio::test]
async fn split_timeline_node_command_returns_timeline_render_projection() {
    let path = temp_db_path("splits-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    let left_node_id = eidetic_core::timeline::node::NodeId::new();
    let right_node_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed split current state");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node.id,
        split_ms,
        left_node_id,
        right_node_id,
    );

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
        clip["node_id"] == left_node_id.0.to_string()
            && clip["start_ms"] == node.time_range.start_ms
            && clip["end_ms"] == split_ms
    }));
    assert!(clips.iter().any(|clip| {
        clip["node_id"] == right_node_id.0.to_string()
            && clip["start_ms"] == split_ms
            && clip["end_ms"] == node.time_range.end_ms
    }));

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let original_count = conn
        .query_row(
            "SELECT COUNT(*) FROM nodes WHERE id = ?1",
            [node.id.0.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .expect("original node count");
    assert_eq!(original_count, 0);
    let left_range = conn
        .query_row(
            "SELECT start_ms, end_ms FROM nodes WHERE id = ?1",
            [left_node_id.0.to_string()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .expect("left split node");
    assert_eq!(
        left_range,
        (node.time_range.start_ms as i64, split_ms as i64)
    );
    let right_range = conn
        .query_row(
            "SELECT start_ms, end_ms FROM nodes WHERE id = ?1",
            [right_node_id.0.to_string()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .expect("right split node");
    assert_eq!(
        right_range,
        (split_ms as i64, node.time_range.end_ms as i64)
    );

    let original_revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
    )
    .expect("original timeline node revisions");
    assert_eq!(original_revisions.len(), 1);
    assert_eq!(
        original_revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Delete
    );
    let left_revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &left_node_id.0.to_string(),
    )
    .expect("left timeline node revisions");
    assert_eq!(left_revisions.len(), 1);
    assert_eq!(
        left_revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Create
    );
    assert!(
        left_revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "start_ms"
                && field.old_value.is_none()
                && field.new_value == Some(FieldValue::Integer(node.time_range.start_ms as i64)))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_generates_result_ids_when_omitted() {
    let path = temp_db_path("splits-timeline-node-generated-ids");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed split current state");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": node.id,
            "at_ms": split_ms,
        }
    });

    let response = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let split_clips: Vec<_> = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips")
        .iter()
        .filter(|clip| {
            clip["name"].as_str().is_some_and(|name| {
                name == format!("{} (L)", node.name) || name == format!("{} (R)", node.name)
            })
        })
        .collect();
    assert_eq!(split_clips.len(), 2);
    assert_ne!(split_clips[0]["node_id"], split_clips[1]["node_id"]);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_replays_duplicate_command() {
    let path = temp_db_path("splits-timeline-node-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let stale_project = project.clone();
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    let left_node_id = eidetic_core::timeline::node::NodeId::new();
    let right_node_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let body = split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node.id,
        split_ms,
        left_node_id,
        right_node_id,
    );

    let first = app
        .clone()
        .oneshot(split_timeline_node_command_request(body.clone()))
        .await
        .expect("first route response");
    *state.project.lock() = Some(stale_project);
    let second = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert!(
        clips
            .iter()
            .all(|clip| clip["node_id"] != node.id.0.to_string())
    );
    assert!(
        clips
            .iter()
            .any(|clip| clip["node_id"] == left_node_id.0.to_string())
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
    )
    .expect("original timeline node revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_validates_against_sqlite_when_project_mirror_is_stale() {
    let path = temp_db_path("splits-timeline-node-stale-mirror");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    let left_node_id = eidetic_core::timeline::node::NodeId::new();
    let right_node_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed split current state");
    let mut stale_project = project;
    stale_project
        .timeline
        .remove_node(node.id)
        .expect("make mirror stale");
    *state.project.lock() = Some(stale_project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node.id,
        split_ms,
        left_node_id,
        right_node_id,
    );

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
    assert!(
        clips
            .iter()
            .any(|clip| clip["node_id"] == left_node_id.0.to_string())
    );
    assert!(
        clips
            .iter()
            .any(|clip| clip["node_id"] == right_node_id.0.to_string())
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("splits-timeline-node-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = split_timeline_node_command_body_with_result_ids(
        command_id,
        node.id,
        split_ms,
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );
    let conflicting = split_timeline_node_command_body_with_result_ids(
        command_id,
        node.id,
        split_ms + 1,
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    );

    let first = app
        .clone()
        .oneshot(split_timeline_node_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(split_timeline_node_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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
async fn split_timeline_node_command_rejects_equal_result_ids() {
    let path = temp_db_path("rejects-equal-timeline-split-result-ids");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    let result_node_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node.id,
        split_ms,
        result_node_id,
        result_node_id,
    );

    let response = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let value = response_json(response).await;
    assert_eq!(
        value["error"],
        "invalid operation: split node ids must be distinct"
    );
    assert_no_recorded_commands(&path);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn split_timeline_node_command_rejects_existing_result_id() {
    let path = temp_db_path("rejects-existing-timeline-split-result-id");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let split_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
    let existing_node_id = project.timeline.nodes[1].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node.id,
        split_ms,
        existing_node_id,
        eidetic_core::timeline::node::NodeId::new(),
    );

    let response = app
        .oneshot(split_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let value = response_json(response).await;
    assert_eq!(
        value["error"],
        "invalid operation: split node ids already exist"
    );
    assert_no_recorded_commands(&path);

    let _ = std::fs::remove_file(path);
}

fn split_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/split-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn split_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    at_ms: u64,
) -> serde_json::Value {
    split_timeline_node_command_body_with_result_ids(
        uuid::Uuid::new_v4(),
        node_id,
        at_ms,
        eidetic_core::timeline::node::NodeId::new(),
        eidetic_core::timeline::node::NodeId::new(),
    )
}

fn split_timeline_node_command_body_with_result_ids(
    command_id: uuid::Uuid,
    node_id: eidetic_core::timeline::node::NodeId,
    at_ms: u64,
    left_node_id: eidetic_core::timeline::node::NodeId,
    right_node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": command_id,
        "payload": {
            "node_id": node_id,
            "at_ms": at_ms,
            "left_node_id": left_node_id,
            "right_node_id": right_node_id,
        }
    })
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn assert_no_recorded_commands(path: &std::path::Path) {
    let conn = crate::sqlite::open_write_connection(path).expect("open db");
    let command_count = conn
        .query_row("SELECT COUNT(*) FROM commands", [], |row| {
            row.get::<_, i64>(0)
        })
        .expect("command count");
    assert_eq!(command_count, 0);
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-command-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}
