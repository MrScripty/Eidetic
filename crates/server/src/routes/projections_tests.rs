use super::*;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphNodeId, BibleGraphSchemaKey, CommandEnvelope,
    CreateBibleGraphNodeCommand, FieldValue, ObjectKind, ScriptBlockId, ScriptBlockKind,
    ScriptDocumentId, ScriptSegmentId, ScriptSegmentStatus, ScriptSpanProvenance,
    SetBibleGraphEdgeCommand, SetBibleGraphFieldCommand, SetObjectFieldCommand,
    SetScriptBlockCommand,
};
use tower::util::ServiceExt;

async fn app_with_project_path(path: PathBuf) -> Router {
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Projection Test"));
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

#[tokio::test]
async fn object_field_projection_requires_loaded_project() {
    let app = router().with_state(AppState::new().await);

    let response = app
        .oneshot(projection_request("bible_part_field", "field-weather"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn object_field_projection_rejects_empty_object_id() {
    let path = temp_db_path("rejects-empty-object-id");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(projection_request("bible_part_field", ""))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_projection_rejects_invalid_object_kind() {
    let path = temp_db_path("rejects-invalid-object-kind");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(projection_request("not_a_kind", "field-weather"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_projection_returns_initial_projection_when_absent() {
    let path = temp_db_path("returns-initial");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(projection_request("bible_part_field", "field-weather"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert!(value.get("change_event_id").is_none());
    assert_eq!(value["payload"]["object_id"], "field-weather");
    assert_eq!(value["payload"]["fields"], serde_json::json!({}));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn object_field_projection_returns_persisted_fields() {
    let path = temp_db_path("returns-persisted");
    seed_weather_field(&path, "rainy");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(projection_request("bible_part_field", "field-weather"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["fields"]["weather"]["value"],
        serde_json::json!("rainy")
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_projection_returns_not_found_when_absent() {
    let path = temp_db_path("bible-node-absent");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_projection_request("node.character.ada"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_projection_rejects_empty_node_id() {
    let path = temp_db_path("bible-node-empty-id");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_projection_request(""))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_projection_returns_persisted_node() {
    let path = temp_db_path("bible-node-persisted");
    seed_bible_graph_node(&path, "node.character.ada", "Ada");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_projection_request("node.character.ada"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(value["payload"]["node"]["id"], "node.character.ada");
    assert_eq!(value["payload"]["node"]["name"], "Ada");
    assert_eq!(value["payload"]["parts"][0]["part"]["part_key"], "profile");
    assert_eq!(
        value["payload"]["parts"][0]["fields"][1]["field_key"],
        "tagline"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_projection_returns_persisted_parts_and_fields() {
    let path = temp_db_path("bible-node-fields-persisted");
    seed_bible_graph_node(&path, "node.character.ada", "Ada");
    seed_bible_graph_field(&path, "Reluctant detective");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_projection_request("node.character.ada"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 3);
    assert_eq!(value["payload"]["parts"][0]["part"]["name"], "Profile");
    assert_eq!(
        value["payload"]["parts"][0]["fields"][0]["value"]["value"],
        "Reluctant detective"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_projection_returns_persisted_edges() {
    let path = temp_db_path("bible-node-edges-persisted");
    seed_bible_graph_node(&path, "node.character.ada", "Ada");
    seed_bible_graph_node(&path, "node.place.beach", "Beach");
    seed_bible_graph_edge(&path);
    let app = app_with_project_path(path.clone()).await;

    let source_response = app
        .clone()
        .oneshot(bible_node_projection_request("node.character.ada"))
        .await
        .expect("source route response");
    let target_response = app
        .oneshot(bible_node_projection_request("node.place.beach"))
        .await
        .expect("target route response");

    assert_eq!(source_response.status(), StatusCode::OK);
    assert_eq!(target_response.status(), StatusCode::OK);
    let source = response_json(source_response).await;
    let target = response_json(target_response).await;
    assert_eq!(
        source["payload"]["outgoing_edges"][0]["id"],
        "edge.ada.beach"
    );
    assert_eq!(
        target["payload"]["incoming_edges"][0]["id"],
        "edge.ada.beach"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_list_projection_returns_empty_list_when_absent() {
    let path = temp_db_path("bible-node-list-empty");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_list_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(value["payload"]["nodes"], serde_json::json!([]));

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_node_list_projection_returns_persisted_nodes() {
    let path = temp_db_path("bible-node-list-persisted");
    seed_bible_graph_node(&path, "node.character.ada", "Ada");
    seed_bible_graph_node(&path, "node.place.beach", "Beach");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_node_list_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 3);
    assert_eq!(value["payload"]["nodes"][0]["id"], "node.character.ada");
    assert_eq!(value["payload"]["nodes"][1]["id"], "node.place.beach");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn bible_graph_schema_list_projection_returns_builtin_schemas() {
    let path = temp_db_path("bible-schema-list");
    seed_bible_graph_node(&path, "node.character.ada", "Ada");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(bible_schema_list_projection_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 1);
    assert_eq!(value["payload"]["schemas"][0]["schema_key"], "character");
    assert_eq!(
        value["payload"]["schemas"][0]["parts"][0]["fields"][1]["field_key"],
        "tagline"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_document_projection_returns_not_found_when_absent() {
    let path = temp_db_path("script-document-absent");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(script_document_projection_request("script.document.main"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn script_document_projection_returns_persisted_script_blocks() {
    let path = temp_db_path("script-document-persisted");
    seed_script_block(&path, "Ada enters with a wet umbrella.");
    let app = app_with_project_path(path.clone()).await;

    let response = app
        .oneshot(script_document_projection_request("script.document.main"))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 5);
    assert_eq!(value["payload"]["document"]["title"], "Pilot");
    assert_eq!(
        value["payload"]["segments"][0]["segment"]["source_node_id"],
        "node.beat.opening"
    );
    assert_eq!(
        value["payload"]["segments"][0]["blocks"][0]["block"]["text"],
        "Ada enters with a wet umbrella."
    );

    let _ = std::fs::remove_file(path);
}

fn seed_weather_field(path: &PathBuf, weather: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    history_store::create_schema(&conn).unwrap();
    let command = CommandEnvelope::new(SetObjectFieldCommand::new(
        ObjectKind::BiblePartField,
        "field-weather",
        "weather",
        Some(FieldValue::Text(weather.to_string())),
    ));
    crate::object_field_command::apply_set_object_field(&mut conn, &command, 100).unwrap();
}

fn seed_bible_graph_node(path: &PathBuf, node_id: &str, name: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(CreateBibleGraphNodeCommand {
        node_id: BibleGraphNodeId::new(node_id).unwrap(),
        parent_id: None,
        schema_key: BibleGraphSchemaKey::new("character").unwrap(),
        name: name.to_string(),
        sort_order: 3,
    });
    crate::bible_graph_command::apply_create_bible_graph_node(&mut conn, &command, 100).unwrap();
}

fn seed_bible_graph_field(path: &PathBuf, value: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(SetBibleGraphFieldCommand {
        node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        part_id: eidetic_core::contracts::BibleGraphPartId::new("part.character.profile").unwrap(),
        part_key: eidetic_core::contracts::BibleGraphPartKey::new("profile").unwrap(),
        part_name: "Profile".to_string(),
        part_sort_order: 1,
        field_id: eidetic_core::contracts::BibleGraphFieldId::new("field.character.tagline")
            .unwrap(),
        field_key: eidetic_core::contracts::BibleGraphFieldKey::new("tagline").unwrap(),
        value: Some(FieldValue::Text(value.to_string())),
        field_sort_order: 2,
    });
    crate::bible_graph_command::apply_set_bible_graph_field(&mut conn, &command, 200).unwrap();
}

fn seed_bible_graph_edge(path: &PathBuf) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(SetBibleGraphEdgeCommand {
        edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
        from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        to_node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
        edge_kind: BibleGraphEdgeKind::LocatedIn,
        label: "located in".to_string(),
        directed: true,
        sort_order: 1,
    });
    crate::bible_graph_command::apply_set_bible_graph_edge(&mut conn, &command, 300).unwrap();
}

fn seed_script_block(path: &PathBuf, text: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    let command = CommandEnvelope::new(SetScriptBlockCommand {
        document_id: ScriptDocumentId::new("script.document.main").unwrap(),
        document_title: "Pilot".to_string(),
        document_sort_order: 0,
        segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
        source_node_id: Some("node.beat.opening".to_string()),
        segment_start_ms: 1_000,
        segment_end_ms: 5_000,
        segment_status: ScriptSegmentStatus::Current,
        segment_sort_order: 1,
        block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
        block_kind: ScriptBlockKind::Action,
        text: text.to_string(),
        span_provenance: ScriptSpanProvenance::UserEdited,
        sort_order: 2,
    });
    crate::script_document_command::apply_set_script_block(&mut conn, &command, 400).unwrap();
}

fn projection_request(object_kind: &str, object_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/projections/object-field?object_kind={object_kind}&object_id={object_id}"
        ))
        .body(Body::empty())
        .unwrap()
}

fn bible_node_projection_request(node_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/projections/bible-graph/node?node_id={node_id}"))
        .body(Body::empty())
        .unwrap()
}

fn bible_node_list_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/bible-graph/nodes")
        .body(Body::empty())
        .unwrap()
}

fn bible_schema_list_projection_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/projections/bible-graph/schemas")
        .body(Body::empty())
        .unwrap()
}

fn script_document_projection_request(document_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/projections/script/document?document_id={document_id}"
        ))
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "eidetic-projection-route-{label}-{}.db",
        uuid::Uuid::new_v4()
    ))
}
