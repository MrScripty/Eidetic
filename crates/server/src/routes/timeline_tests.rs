use super::*;
use std::path::PathBuf;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use eidetic_core::Template;
use tower::util::ServiceExt;

#[tokio::test]
async fn timeline_route_reads_sqlite_not_stale_project_mirror() {
    let path = temp_db_path("timeline-route-sqlite");
    let state = AppState::new().await;
    let mut project = Template::MultiCam.build_project("Timeline Route Test");
    let node_id = project.timeline.nodes[0].id;
    project.timeline.node_mut(node_id).unwrap().name = "SQLite timeline node".to_string();
    crate::persistence::save_project(&project, &path, None)
        .await
        .expect("seed timeline database");

    project.timeline.node_mut(node_id).unwrap().name = "Stale mirror node".to_string();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(path.clone());
    let app = router().with_state(state);

    let response = app
        .oneshot(timeline_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let value = response_json(response).await;
    let node = value["nodes"]
        .as_array()
        .expect("timeline nodes")
        .iter()
        .find(|node| node["id"] == node_id.0.to_string())
        .expect("persisted node");
    assert_eq!(node["name"], "SQLite timeline node");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn timeline_route_requires_loaded_project_path() {
    let app = router().with_state(AppState::new().await);

    let response = app
        .oneshot(timeline_request())
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

fn timeline_request() -> Request<Body> {
    Request::builder()
        .uri("/timeline")
        .body(Body::empty())
        .expect("timeline request")
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response bytes");
    serde_json::from_slice(&bytes).expect("json response")
}

fn temp_db_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("eidetic-{label}-{}.db", uuid::Uuid::new_v4()))
}
