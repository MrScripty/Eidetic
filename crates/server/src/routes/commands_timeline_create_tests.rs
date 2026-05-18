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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_node = conn
        .query_row(
            "SELECT parent_id, level, start_ms, end_ms, name, locked FROM nodes WHERE id = ?1",
            [node_id.0.to_string()],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .expect("persisted node");
    assert_eq!(
        persisted_node,
        (
            Some(parent.id.0.to_string()),
            "Act".to_string(),
            parent.time_range.start_ms as i64,
            (parent.time_range.start_ms + 1_000) as i64,
            "Inserted act".to_string(),
            0
        )
    );

    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node_id.0.to_string(),
    )
    .expect("timeline node revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Create
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "name"
                && field.new_value == Some(FieldValue::Text("Inserted act".to_string())))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "parent_id"
                && field.new_value == Some(FieldValue::Text(parent.id.0.to_string())))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "level"
                && field.new_value == Some(FieldValue::Text("Act".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_node_command_replays_duplicate_command() {
    let path = temp_db_path("creates-timeline-node-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let node_id = eidetic_core::timeline::node::NodeId::new();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_node_command_body_with_id(
        uuid::Uuid::new_v4(),
        node_id,
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Inserted act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
    );

    let first = app
        .clone()
        .oneshot(create_timeline_node_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(create_timeline_node_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    assert_eq!(
        clips
            .iter()
            .filter(|clip| clip["node_id"] == node_id.0.to_string())
            .count(),
        1
    );

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node_id.0.to_string(),
    )
    .expect("timeline node revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_node_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("creates-timeline-node-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let command_id = uuid::Uuid::new_v4();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let original = create_timeline_node_command_body_with_id(
        command_id,
        eidetic_core::timeline::node::NodeId::new(),
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Inserted act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
    );
    let conflicting = create_timeline_node_command_body_with_id(
        command_id,
        eidetic_core::timeline::node::NodeId::new(),
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Different act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
    );

    let first = app
        .clone()
        .oneshot(create_timeline_node_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(create_timeline_node_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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

fn create_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/create-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
    parent_id: Option<eidetic_core::timeline::node::NodeId>,
    level: eidetic_core::timeline::node::StoryLevel,
    name: &str,
    start_ms: u64,
    end_ms: u64,
) -> serde_json::Value {
    create_timeline_node_command_body_with_id(
        uuid::Uuid::new_v4(),
        node_id,
        parent_id,
        level,
        name,
        start_ms,
        end_ms,
    )
}

fn create_timeline_node_command_body_with_id(
    command_id: uuid::Uuid,
    node_id: eidetic_core::timeline::node::NodeId,
    parent_id: Option<eidetic_core::timeline::node::NodeId>,
    level: eidetic_core::timeline::node::StoryLevel,
    name: &str,
    start_ms: u64,
    end_ms: u64,
) -> serde_json::Value {
    json!({
        "id": command_id,
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
