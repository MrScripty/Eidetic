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
    let project = Template::MultiCam.build_project("Commands Test");
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path);
    router().with_state(state)
}

use eidetic_core::contracts::{
    CommandEnvelope, CommandId, CreateStoryArcCommand, FieldValue, ObjectKind,
};
use eidetic_core::story::arc::{ArcId, ArcType, Color};

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
async fn story_arc_create_command_returns_arc_list_projection() {
    let path = temp_db_path("story-arc-create");
    let app = app_with_project_path(path.clone()).await;
    let arc_id = ArcId::new();
    let body = serde_json::to_value(CommandEnvelope {
        id: CommandId::new(),
        payload: CreateStoryArcCommand {
            arc_id,
            parent_arc_id: None,
            name: "Mystery".to_string(),
            description: "Central investigation".to_string(),
            arc_type: ArcType::APlot,
            color: Color::A_PLOT,
        },
    })
    .unwrap();

    let response = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    assert_eq!(value["projection"]["version"], 2);
    assert!(
        value["projection"]["payload"]["arcs"]
            .as_array()
            .expect("arcs")
            .iter()
            .any(|arc| arc["id"] == serde_json::json!(arc_id)
                && arc["name"] == serde_json::json!("Mystery"))
    );
    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let revisions = crate::history_store::load_revisions_for_object(
        &conn,
        ObjectKind::StoryArc,
        &arc_id.0.to_string(),
    )
    .expect("story arc revisions");
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
                && field.new_value == Some(FieldValue::Text("Mystery".to_string())))
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_generates_arc_id_when_omitted() {
    let path = temp_db_path("story-arc-create-generated-id");
    let app = app_with_project_path(path.clone()).await;
    let body = json!({
        "id": CommandId::new(),
        "payload": {
            "parent_arc_id": null,
            "name": "Mystery",
            "description": "Central investigation",
            "arc_type": ArcType::APlot,
            "color": Color::A_PLOT,
        }
    });

    let response = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["outcome"], "recorded");
    let arc = value["projection"]["payload"]["arcs"]
        .as_array()
        .expect("arcs")
        .iter()
        .find(|arc| arc["name"] == "Mystery")
        .expect("generated arc");
    let generated_arc_id = arc["id"].as_str().expect("generated arc id");

    let conn = crate::sqlite::open_write_connection(&path).expect("open db");
    let persisted_count = conn
        .query_row(
            "SELECT COUNT(*) FROM arcs WHERE id = ?1",
            [generated_arc_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("persisted generated arc count");
    assert_eq!(persisted_count, 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_replays_omitted_arc_id() {
    let path = temp_db_path("story-arc-create-generated-id-replay");
    let app = app_with_project_path(path.clone()).await;
    let body = json!({
        "id": CommandId::new(),
        "payload": {
            "parent_arc_id": null,
            "name": "Mystery",
            "description": "Central investigation",
            "arc_type": ArcType::APlot,
            "color": Color::A_PLOT,
        }
    });

    let first = app
        .clone()
        .oneshot(story_arc_create_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    assert_eq!(
        value["projection"]["payload"]["arcs"]
            .as_array()
            .expect("arcs")
            .iter()
            .filter(|arc| arc["name"] == "Mystery")
            .count(),
        1
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_replays_duplicate_command() {
    let path = temp_db_path("story-arc-create-duplicate");
    let app = app_with_project_path(path.clone()).await;
    let arc_id = ArcId::new();
    let body = serde_json::to_value(CommandEnvelope {
        id: CommandId::new(),
        payload: CreateStoryArcCommand {
            arc_id,
            parent_arc_id: None,
            name: "Mystery".to_string(),
            description: "Central investigation".to_string(),
            arc_type: ArcType::APlot,
            color: Color::A_PLOT,
        },
    })
    .unwrap();

    let first = app
        .clone()
        .oneshot(story_arc_create_command_request(body.clone()))
        .await
        .expect("first route response");
    let second = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let value = response_json(second).await;
    assert_eq!(value["outcome"], "already_recorded");
    assert_eq!(
        value["projection"]["payload"]["arcs"]
            .as_array()
            .expect("arcs")
            .iter()
            .filter(|arc| arc["id"] == serde_json::json!(arc_id))
            .count(),
        1
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_rejects_existing_sqlite_arc_when_project_mirror_is_stale() {
    let path = temp_db_path("story-arc-create-db-conflict");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Commands Test");
    let existing_arc_id = project.arcs[0].id;
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed project database");
    project.arcs.clear();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);
    let body = serde_json::to_value(CommandEnvelope {
        id: CommandId::new(),
        payload: CreateStoryArcCommand {
            arc_id: existing_arc_id,
            parent_arc_id: None,
            name: "Duplicate".to_string(),
            description: String::new(),
            arc_type: ArcType::APlot,
            color: Color::A_PLOT,
        },
    })
    .unwrap();

    let response = app
        .oneshot(story_arc_create_command_request(body))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn story_arc_create_command_rejects_conflicting_duplicate_command() {
    let path = temp_db_path("story-arc-create-conflict");
    let app = app_with_project_path(path.clone()).await;
    let command_id = CommandId::new();
    let original = serde_json::to_value(CommandEnvelope {
        id: command_id,
        payload: CreateStoryArcCommand {
            arc_id: ArcId::new(),
            parent_arc_id: None,
            name: "Mystery".to_string(),
            description: "Central investigation".to_string(),
            arc_type: ArcType::APlot,
            color: Color::A_PLOT,
        },
    })
    .unwrap();
    let conflicting = serde_json::to_value(CommandEnvelope {
        id: command_id,
        payload: CreateStoryArcCommand {
            arc_id: ArcId::new(),
            parent_arc_id: None,
            name: "B Story".to_string(),
            description: "Different payload".to_string(),
            arc_type: ArcType::BPlot,
            color: Color::B_PLOT,
        },
    })
    .unwrap();

    let first = app
        .clone()
        .oneshot(story_arc_create_command_request(original))
        .await
        .expect("first route response");
    let second = app
        .oneshot(story_arc_create_command_request(conflicting))
        .await
        .expect("second route response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::CONFLICT);

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

fn command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/object-field")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn story_arc_create_command_request(body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/commands/story/create-arc")
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
