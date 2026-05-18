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
async fn timeline_node_lock_command_returns_timeline_render_projection() {
    let path = temp_db_path("locks-timeline-node");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_locked = conn
        .query_row(
            "SELECT locked FROM nodes WHERE id = ?1",
            [node.id.0.to_string()],
            |row| row.get::<_, i64>(0),
        )
        .expect("persisted node lock");
    assert_eq!(persisted_locked, 1);

    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
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
            .any(|field| field.field_key == "locked"
                && field.old_value == Some(FieldValue::Bool(false))
                && field.new_value == Some(FieldValue::Bool(true)))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_lock_command_replays_duplicate_command() {
    let path = temp_db_path("timeline-node-lock-duplicate");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let command_id = uuid::Uuid::new_v4();
    let body = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "locked": true,
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_lock_command_request(body.clone()))
        .await
        .expect("first route response");
    state
        .project
        .lock()
        .as_mut()
        .expect("project")
        .timeline
        .node_mut(node.id)
        .expect("node")
        .locked = false;
    let second = app
        .oneshot(timeline_node_lock_command_request(body))
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
        .find(|clip| clip["node_id"] == node.id.0.to_string())
        .expect("locked clip");
    assert_eq!(clip["locked"], true);

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
async fn timeline_node_lock_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("timeline-node-lock-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let command_id = uuid::Uuid::new_v4();
    let original = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "locked": true,
        }
    });
    let conflicting = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "locked": false,
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_lock_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(timeline_node_lock_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_lock_command_rejects_unknown_node() {
    let path = temp_db_path("rejects-unknown-timeline-lock");
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
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

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_content = conn
        .query_row(
            "SELECT content_json FROM nodes WHERE id = ?1",
            [node.id.0.to_string()],
            |row| row.get::<_, String>(0),
        )
        .expect("persisted node content");
    let persisted_content: serde_json::Value =
        serde_json::from_str(&persisted_content).expect("content json");
    assert_eq!(persisted_content["notes"], "New outline");
    assert_eq!(persisted_content["status"], "NotesOnly");

    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::TimelineNode,
        &node.id.0.to_string(),
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
            .any(|field| field.field_key == "notes"
                && field.old_value == Some(FieldValue::Text(String::new()))
                && field.new_value == Some(FieldValue::Text("New outline".to_string())))
    );
    assert!(
        revisions[0]
            .fields
            .iter()
            .any(|field| field.field_key == "content_status"
                && field.old_value == Some(FieldValue::Text("Empty".to_string()))
                && field.new_value == Some(FieldValue::Text("NotesOnly".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_node_notes_command_replays_duplicate_command() {
    let path = temp_db_path("timeline-node-notes-duplicate");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    project.timeline.node_mut(node.id).unwrap().content.status =
        eidetic_core::timeline::node::ContentStatus::Empty;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state.clone());
    let command_id = uuid::Uuid::new_v4();
    let body = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "notes": "New outline",
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_notes_command_request(body.clone()))
        .await
        .expect("first route response");
    let mut guard = state.project.lock();
    let stale_node = guard
        .as_mut()
        .expect("project")
        .timeline
        .node_mut(node.id)
        .expect("node");
    stale_node.content.notes.clear();
    stale_node.content.status = eidetic_core::timeline::node::ContentStatus::Empty;
    drop(guard);
    let second = app
        .oneshot(timeline_node_notes_command_request(body))
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
        .find(|clip| clip["node_id"] == node.id.0.to_string())
        .expect("notes clip");
    assert_eq!(clip["content_status"], "NotesOnly");

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
async fn timeline_node_notes_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("timeline-node-notes-conflict");
    let state = AppState::new().await;
    let project = Template::MultiCam.build_project("Commands Test");
    let node = project.timeline.nodes[0].clone();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let command_id = uuid::Uuid::new_v4();
    let original = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "notes": "New outline",
        }
    });
    let conflicting = json!({
        "id": command_id,
        "payload": {
            "node_id": node.id,
            "notes": "Different outline",
        }
    });

    let first = app
        .clone()
        .oneshot(timeline_node_notes_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(timeline_node_notes_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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
