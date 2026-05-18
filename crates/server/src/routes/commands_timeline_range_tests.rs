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
async fn timeline_node_range_command_returns_timeline_render_projection() {
    let path = temp_db_path("sets-timeline-node-range");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_range = conn
        .query_row(
            "SELECT start_ms, end_ms FROM nodes WHERE id = ?1",
            [node_id.0.to_string()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .expect("persisted node range");
    assert_eq!(persisted_range, (1_000, 2_000));

    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node_id.0.to_string(),
    )
    .expect("timeline node revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Update
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "start_ms"
                && field.old_value == Some(FieldValue::Integer(0))
                && field.new_value == Some(FieldValue::Integer(1_000)))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "end_ms"
                && field.new_value == Some(FieldValue::Integer(2_000)))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_range_command_replays_duplicate_command() {
    let path = temp_db_path("timeline-node-range-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let command_id = uuid::Uuid::new_v4();
    let body = json!({
        "id": command_id,
        "payload": {
            "node_id": node_id,
            "start_ms": 1_000,
            "end_ms": 2_000,
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_range_command_request(body.clone()))
        .await
        .expect("first route response");
    state
        .project
        .lock()
        .as_mut()
        .expect("project")
        .timeline
        .resize_node(
            node_id,
            eidetic_core::timeline::timing::TimeRange::new(3_000, 4_000).unwrap(),
        )
        .expect("make mirror stale");
    let second = app
        .oneshot(timeline_node_range_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    let clips = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips");
    let clip = clips
        .iter()
        .find(|clip| clip["node_id"] == node_id.0.to_string())
        .expect("updated node clip");
    assert_eq!(clip["start_ms"], 1_000);
    assert_eq!(clip["end_ms"], 2_000);

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
async fn timeline_node_range_command_validates_against_sqlite_when_project_mirror_is_stale() {
    let path = temp_db_path("timeline-node-range-stale-mirror");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed timeline database");
    let mut stale_project = project;
    stale_project
        .timeline
        .remove_node(node_id)
        .expect("make mirror stale");
    *state.project.lock() = Some(stale_project);
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
    let clip = value["projection"]["payload"]["clips"]
        .as_array()
        .expect("timeline clips")
        .iter()
        .find(|clip| clip["node_id"] == node_id.0.to_string())
        .expect("updated node clip");
    assert_eq!(clip["start_ms"], 1_000);
    assert_eq!(clip["end_ms"], 2_000);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_range_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("timeline-node-range-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node_id = project.timeline.nodes[0].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = json!({
        "id": command_id,
        "payload": {
            "node_id": node_id,
            "start_ms": 1_000,
            "end_ms": 2_000,
        }
    });
    let conflicting = json!({
        "id": command_id,
        "payload": {
            "node_id": node_id,
            "start_ms": 2_000,
            "end_ms": 3_000,
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_range_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(timeline_node_range_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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

fn timeline_node_range_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/node-range")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
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
