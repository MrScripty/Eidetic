use axum::Router;
use axum::extract::State;
use axum::http::{HeaderName, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::post;
use eidetic_core::contracts::ScriptDocumentId;

use crate::export::generate_screenplay_pdf;
use crate::history_store::HistoryStoreError;
use crate::script_store;
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

const MAIN_SCRIPT_DOCUMENT_ID: &str = "script.document.main";

pub fn router() -> Router<AppState> {
    Router::new().route("/export/pdf", post(export_pdf))
}

async fn export_pdf(State(state): State<AppState>) -> impl IntoResponse {
    let project_name = {
        let guard = state.project.lock();
        match guard.as_ref() {
            Some(p) => p.name.clone(),
            None => {
                return error_response(StatusCode::BAD_REQUEST, "no project loaded".to_string());
            }
        }
    };
    let path = match active_project_path(&state) {
        Ok(path) => path,
        Err(error) => return error_response(error.0, error.1),
    };

    // PDF generation is CPU-bound; run on blocking thread pool.
    let result = tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ExportPdfError::internal(e.to_string()))?;
        script_store::create_schema(&conn).map_err(map_history_error_status)?;
        let document_id = ScriptDocumentId::new(MAIN_SCRIPT_DOCUMENT_ID)
            .map_err(|e| ExportPdfError::bad_request(e.to_string()))?;
        let projection = script_store::load_document_projection(&conn, &document_id)
            .map_err(map_history_error_status)?
            .ok_or_else(|| ExportPdfError::not_found("script document not found"))?;
        generate_screenplay_pdf(&project_name, &projection).map_err(ExportPdfError::internal)
    })
    .await;

    match result {
        Ok(Ok(pdf_bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        ),
        Ok(Err(error)) => error_response(error.status, error.message),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!("{{\"error\":\"task join error: {e}\"}}").into_bytes(),
        ),
    }
}

#[derive(Debug)]
struct ExportPdfError {
    status: StatusCode,
    message: String,
}

impl ExportPdfError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

fn map_history_error_status(error: HistoryStoreError) -> ExportPdfError {
    let error = map_history_error(error);
    ExportPdfError {
        status: error.0,
        message: error.1,
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
