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
    let app = router().with_state(state.clone());
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
async fn create_timeline_node_command_generates_node_id_when_omitted() {
    let path = temp_db_path("creates-timeline-node-generated-id");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let existing_node_ids: Vec<_> = project
        .timeline
        .nodes
        .iter()
        .map(|node| node.id.0.to_string())
        .collect();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let mut body = create_timeline_node_command_body(
        eidetic_core::timeline::node::NodeId::new(),
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Backend id act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
    );
    body["payload"]
        .as_object_mut()
        .expect("payload object")
        .remove("node_id");

    let response = app
        .oneshot(create_timeline_node_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let clip = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips")
        .iter()
        .find(|clip| clip["name"] == "Backend id act")
        .expect("generated node clip");
    let generated_node_id = clip["node_id"].as_str().expect("generated node id");
    assert!(!existing_node_ids.iter().any(|id| id == generated_node_id));

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_count = conn
        .query_row(
            "SELECT COUNT(*) FROM nodes WHERE id = ?1",
            [generated_node_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("persisted generated node count");
    assert_eq!(persisted_count, 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn create_timeline_node_command_replays_omitted_node_id() {
    let path = temp_db_path("creates-timeline-node-generated-id-replay");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "parent_id": parent.id,
            "level": parent.level.child_level().expect("child level"),
            "name": "Backend id replay act",
            "start_ms": parent.time_range.start_ms,
            "end_ms": parent.time_range.start_ms + 1_000,
            "beat_type": null,
        }
    });

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
    let clips: Vec<_> = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips")
        .iter()
        .filter(|clip| clip["name"] == "Backend id replay act")
        .collect();
    assert_eq!(clips.len(), 1);

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
    let app = router().with_state(state.clone());
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
async fn create_timeline_node_command_validates_against_sqlite_when_project_mirror_is_stale() {
    let path = temp_db_path("creates-timeline-node-stale-mirror");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let node_id = eidetic_core::timeline::node::NodeId::new();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed node current state");
    let mut stale_project = project;
    stale_project
        .timeline
        .remove_node(parent.id)
        .expect("make mirror stale");
    *state.project.lock() = Some(stale_project);
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

#[tokio::test]
async fn create_timeline_node_command_rejects_existing_node_id() {
    let path = temp_db_path("rejects-existing-timeline-node-id");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let parent = project.timeline.nodes[0].clone();
    let existing_node_id = project.timeline.nodes[1].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = create_timeline_node_command_body(
        existing_node_id,
        Some(parent.id),
        parent.level.child_level().expect("child level"),
        "Duplicate id act",
        parent.time_range.start_ms,
        parent.time_range.start_ms + 1_000,
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
