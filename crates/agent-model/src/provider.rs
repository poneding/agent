use async_trait::async_trait;
use thiserror::Error;

use crate::request::ModelRequest;
use crate::stream::ModelEventStream;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ModelError {
    #[error("model script failed: {0}")]
    Script(String),
    #[error("model provider failed [{code}]: {message}")]
    Provider {
        code: String,
        message: String,
        retryable: bool,
    },
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError>;
}
