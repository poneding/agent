use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum UiBridgeError {
    #[error("json serialization failed: {0}")]
    Json(String),
    #[error("ui bridge validation failed: {0}")]
    Validation(String),
}

impl From<serde_json::Error> for UiBridgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}
