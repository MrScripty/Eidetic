use eidetic_server::backend_error::BackendError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    kind: &'static str,
    message: String,
}

impl From<BackendError> for CommandError {
    fn from(error: BackendError) -> Self {
        let kind = match error {
            BackendError::NotFound(_) => "not_found",
            BackendError::BadRequest(_) => "bad_request",
            BackendError::Conflict(_) => "conflict",
            BackendError::Internal(_) => "internal",
        };

        Self {
            kind,
            message: error.message().to_string(),
        }
    }
}

impl CommandError {
    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: "internal",
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandError;

    #[test]
    fn internal_error_serializes_transport_shape() {
        let error = CommandError::internal("renderer unavailable");

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({
                "kind": "internal",
                "message": "renderer unavailable",
            })
        );
    }
}
