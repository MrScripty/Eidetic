use super::router;
use std::path::PathBuf;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use serde_json::json;
use tower::util::ServiceExt;

use crate::state::AppState;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Commands Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId,
    BibleGraphPartId, BibleGraphPartKey, BibleGraphSchemaKey, EnsureCanonicalBibleRootsCommand,
    FieldValue,
};

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
async fn bible_graph_node_command_generates_node_id_when_omitted() {
    let path = temp_db_path("creates-bible-graph-node-generated-id");
    let app = app_with_project_path(path.clone()).await;
    let mut body = bible_graph_node_command_body("node.character.ada", "Ada");
    body["payload"]
        .as_object_mut()
        .expect("payload object")
        .remove("node_id");

    let response = app
        .oneshot(bible_graph_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let node_id = value["projection"]["payload"]["node"]["id"]
        .as_str()
        .expect("generated node id");
    assert!(node_id.starts_with("node.character."));
    assert_eq!(value["projection"]["payload"]["node"]["name"], "Ada");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_command_replays_omitted_node_id() {
    let path = temp_db_path("creates-bible-graph-node-generated-id-replay");
    let app = app_with_project_path(path.clone()).await;
    let mut body = bible_graph_node_command_body("node.character.ada", "Ada");
    body["payload"]
        .as_object_mut()
        .expect("payload object")
        .remove("node_id");

    let first = app
        .clone()
        .oneshot(bible_graph_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(bible_graph_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    assert_eq!(value["projection"]["payload"]["node"]["name"], "Ada");

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
async fn bible_graph_edge_command_generates_edge_id_when_omitted() {
    let path = temp_db_path("sets-bible-graph-edge-generated-id");
    let app = app_with_project_path(path.clone()).await;
    let source = bible_graph_node_command_body("node.character.ada", "Ada");
    let target = bible_graph_node_command_body("node.place.beach", "Beach");
    let mut edge = bible_graph_edge_command_body();
    edge["payload"]
        .as_object_mut()
        .expect("payload object")
        .remove("edge_id");

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
    let edge_id = value["projection"]["payload"]["outgoing_edges"][0]["id"]
        .as_str()
        .expect("generated edge id");
    assert!(edge_id.starts_with("edge."));

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
