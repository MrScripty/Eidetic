use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSchemaKey,
    BibleGraphSnapshotFieldId, BibleGraphSnapshotId, FieldValue,
};
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
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
async fn bible_graph_snapshot_field_command_generates_ids_when_omitted() {
    let path = temp_db_path("sets-bible-graph-snapshot-field-generated-ids");
    let app = app_with_project_path(path.clone()).await;
    let node = bible_graph_node_command_body("node.character.ada", "Ada");
    let mut snapshot = bible_graph_snapshot_field_command_body(Some(json_text("Rain-soaked")));
    let payload = snapshot["payload"].as_object_mut().expect("payload object");
    payload.remove("snapshot_id");
    payload.remove("field_id");

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
    let snapshot_id = value["projection"]["payload"]["snapshots"][0]["snapshot"]["id"]
        .as_str()
        .expect("generated snapshot id");
    let field_id = value["projection"]["payload"]["snapshots"][0]["fields"][0]["id"]
        .as_str()
        .expect("generated snapshot field id");
    assert!(snapshot_id.starts_with("snapshot."));
    assert!(field_id.starts_with("snapshot-field."));

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

fn bible_graph_snapshot_field_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/bible-graph/snapshot-field")
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

fn json_text(value: &str) -> serde_json::Value {
    serde_json::to_value(FieldValue::Text(value.to_string())).unwrap()
}

fn bible_graph_node_command_body(node_id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4(),
        "payload": {
            "node_id": BibleGraphNodeId::new(node_id).unwrap(),
            "parent_id": null,
            "schema_key": BibleGraphSchemaKey::new("character").unwrap(),
            "name": name,
            "sort_order": 3,
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
