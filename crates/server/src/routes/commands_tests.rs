use super::*;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleGraphNodeId, BibleGraphSchemaKey, EnsureCanonicalBibleRootsCommand, FieldValue, ObjectKind,
};
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
        value["projection"]["payload"]["parts"],
        serde_json::json!([])
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

fn command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/object-field")
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
