use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackendError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Internal(String),
}

impl BackendError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn no_project() -> Self {
        Self::not_found("no project loaded")
    }

    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(message)
            | Self::BadRequest(message)
            | Self::Conflict(message)
            | Self::Internal(message) => message,
        }
    }
}
