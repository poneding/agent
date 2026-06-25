use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum McpError {
    #[error("invalid MCP config: {0}")]
    InvalidConfig(String),
    #[error("invalid MCP tool descriptor: {0}")]
    InvalidToolDescriptor(String),
    #[error("MCP server not found: {0}")]
    ServerNotFound(String),
    #[error("MCP client error: {0}")]
    Client(String),
}
