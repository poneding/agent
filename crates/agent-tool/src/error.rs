use thiserror::Error;

use crate::definition::ToolName;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ToolNameError {
    #[error("tool name cannot be empty")]
    Empty,
    #[error("tool name contains invalid characters: {0}")]
    InvalidCharacters(String),
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ToolError {
    #[error("tool input validation failed: {0}")]
    Validation(String),
    #[error("tool execution failed: {0}")]
    Execution(String),
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ToolRegistryError {
    #[error("duplicate tool: {0}")]
    DuplicateTool(ToolName),
}
