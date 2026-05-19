use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use eidetic_core::contracts::{CommandEnvelope, FieldValue, ObjectKind, SetObjectFieldCommand};
use tower::util::ServiceExt;
use uuid::Uuid;

use super::router;
use crate::state::AppState;

#[tokio::test]
async fn change_review_projection_returns_recorded_field_deltas() {
    let path = std::env::temp_dir().join(format!("eidetic-change-review-{}.db", Uuid::new_v4()));
    let state = AppState::new().await;
    *state.project.lock() = Some(Template::MultiCam.build_project("Change Review Test"));
    *state.project_path.lock() = Some(path.clone());
    seed_weather_change(&path, "rainy");
    let app = router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/projections/history/changes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["payload"]["changes"][0]["event"]["summary"],
        "set weather"
    );
    assert_eq!(
        value["payload"]["changes"][0]["revisions"][0]["fields"][0]["new_value"]["value"],
        "rainy"
    );

    let _ = std::fs::remove_file(path);
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json response")
}

fn seed_weather_change(path: &std::path::Path, weather: &str) {
    let mut conn = crate::sqlite::open_write_connection(path).unwrap();
    crate::history_store::create_schema(&conn).unwrap();
    let command = CommandEnvelope::new(SetObjectFieldCommand::new(
        ObjectKind::BiblePartField,
        "field-weather",
        "weather",
        Some(FieldValue::Text(weather.to_string())),
    ));
    crate::object_field_command::apply_set_object_field(&mut conn, &command, 100).unwrap();
}
