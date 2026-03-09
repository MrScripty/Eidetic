use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

/// Unified API error type that returns proper HTTP status codes.
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

pub type ApiJson = Result<Json<serde_json::Value>, ApiError>;

impl ApiError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into())
    }

    pub fn no_project() -> Self {
        Self(StatusCode::NOT_FOUND, "no project loaded".into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = axum::Json(json!({ "error": self.1 }));
        (self.0, body).into_response()
    }
}

pub fn json_value<T: Serialize>(value: T) -> ApiJson {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}
