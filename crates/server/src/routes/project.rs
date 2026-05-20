use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};

use crate::error::ApiError;
use crate::project_service::{
    CreateProjectRequest, LoadProjectRequest, SaveProjectRequest, UpdateProjectRequest,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/project", post(create_project))
        .route("/project", get(get_project))
        .route("/project", put(update_project))
        .route("/project/save", post(save_project))
        .route("/project/load", post(load_project))
        .route("/project/list", get(list_projects))
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::project_service::create_project(&state, body)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn get_project(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    crate::project_service::get_project(&state)
        .map(Json)
        .map_err(ApiError::from)
}

async fn update_project(
    State(state): State<AppState>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::project_service::update_project(&state, body)
        .map(Json)
        .map_err(ApiError::from)
}

async fn save_project(
    State(state): State<AppState>,
    Json(body): Json<SaveProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::project_service::save_project(&state, body)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn load_project(
    State(state): State<AppState>,
    Json(body): Json<LoadProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::project_service::load_project(&state, body)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn list_projects() -> Json<serde_json::Value> {
    Json(crate::project_service::list_projects().await)
}

#[cfg(test)]
mod tests {
    use super::router;
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn create_project_rejects_invalid_name() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"bad/name","template":"multi_cam"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn save_project_without_loaded_project_returns_not_found() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn load_project_rejects_path_outside_root() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project/load")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"path":"/tmp/outside/project.db"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
