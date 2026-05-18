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
async fn delete_timeline_node_command_returns_timeline_render_projection() {
    let path = temp_db_path("deletes-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    let descendant_ids: Vec<_> = project
        .timeline
        .descendants_of(node.id)
        .iter()
        .map(|descendant| descendant.id)
        .collect();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed node current state");
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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    for removed_node_id in std::iter::once(node.id).chain(descendant_ids.iter().copied()) {
        let persisted_count = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE id = ?1",
                [removed_node_id.0.to_string()],
                |row| row.get::<_, i64>(0),
            )
            .expect("persisted removed node count");
        assert_eq!(persisted_count, 0);
    }

    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
    )
    .expect("timeline node revisions");
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        revisions[0].operation,
        eidetic_core::contracts::RevisionOperation::Delete
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "name"
                && field.old_value == Some(FieldValue::Text(node.name.clone()))
                && field.new_value.is_none())
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_node_command_replays_duplicate_command() {
    let path = temp_db_path("deletes-timeline-node-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = delete_timeline_node_command_body_with_id(uuid::Uuid::new_v4(), node.id);

    let first = app
        .clone()
        .oneshot(delete_timeline_node_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(delete_timeline_node_command_request(body))
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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
    )
    .expect("timeline node revisions");
    assert_eq!(revisions.len(), 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_timeline_node_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("deletes-timeline-node-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let first_node_id = project.timeline.nodes[0].id;
    let second_node_id = project.timeline.nodes[1].id;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = delete_timeline_node_command_body_with_id(command_id, first_node_id);
    let conflicting = delete_timeline_node_command_body_with_id(command_id, second_node_id);

    let first = app
        .clone()
        .oneshot(delete_timeline_node_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(delete_timeline_node_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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

fn delete_timeline_node_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/timeline/delete-node")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn delete_timeline_node_command_body(
    node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    delete_timeline_node_command_body_with_id(uuid::Uuid::new_v4(), node_id)
}

fn delete_timeline_node_command_body_with_id(
    command_id: uuid::Uuid,
    node_id: eidetic_core::timeline::node::NodeId,
) -> serde_json::Value {
    json!({
        "id": command_id,
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
