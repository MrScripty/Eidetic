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
