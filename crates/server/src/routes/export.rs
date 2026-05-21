use axum::Router;
use axum::extract::State;
use axum::http::{HeaderName, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::post;

use crate::backend_error::BackendError;
use crate::export_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/export/pdf", post(export_pdf))
}

async fn export_pdf(State(state): State<AppState>) -> impl IntoResponse {
    match export_service::export_pdf(&state).await {
        Ok(pdf_bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        ),
        Err(error) => error_response(error_status(&error), error.message().to_string()),
    }
}

fn error_status(error: &BackendError) -> StatusCode {
    match error {
        BackendError::BadRequest(_) => StatusCode::BAD_REQUEST,
        BackendError::NotFound(_) => StatusCode::NOT_FOUND,
        BackendError::Conflict(_) => StatusCode::CONFLICT,
        BackendError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(
    status: StatusCode,
    message: String,
) -> (StatusCode, [(HeaderName, &'static str); 1], Vec<u8>) {
    let body = serde_json::json!({ "error": message }).to_string();
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.into_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use eidetic_core::Template;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn export_pdf_requires_loaded_project() {
        let app = router().with_state(AppState::new().await);

        let response = app.oneshot(export_request()).await.expect("route response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn export_pdf_requires_script_document_projection() {
        let path = temp_db_path("missing-script-document");
        let state = AppState::new().await;
        *state.project.lock() = Some(Template::MultiCam.build_project("Export Test"));
        *state.project_path.lock() = Some(path.clone());
        let app = router().with_state(state);

        let response = app.oneshot(export_request()).await.expect("route response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let _ = std::fs::remove_file(path);
    }

    fn export_request() -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/export/pdf")
            .body(Body::empty())
            .unwrap()
    }

    fn temp_db_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "eidetic-export-route-{label}-{}.db",
            uuid::Uuid::new_v4()
        ))
    }
}
