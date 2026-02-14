use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;

use crate::export::generate_screenplay_pdf;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/export/pdf", post(export_pdf))
}

async fn export_pdf(State(state): State<AppState>) -> impl IntoResponse {
    let project = {
        let guard = state.project.lock();
        match guard.as_ref() {
            Some(p) => p.clone(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    [(header::CONTENT_TYPE, "application/json")],
                    b"{\"error\":\"no project loaded\"}".to_vec(),
                );
            }
        }
    };

    // PDF generation is CPU-bound; run on blocking thread pool.
    let result = tokio::task::spawn_blocking(move || generate_screenplay_pdf(&project)).await;

    match result {
        Ok(Ok(pdf_bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf_bytes,
        ),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!("{{\"error\":\"{e}\"}}").into_bytes(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!("{{\"error\":\"task join error: {e}\"}}").into_bytes(),
        ),
    }
}
