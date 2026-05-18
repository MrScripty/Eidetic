use super::*;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{CommandEnvelope, FieldValue, ObjectKind, SetObjectFieldCommand};
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

fn projection_request(object_kind: &str, object_id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/projections/object-field?object_kind={object_kind}&object_id={object_id}"
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
