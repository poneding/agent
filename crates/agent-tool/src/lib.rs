#![forbid(unsafe_code)]
//! Tool contracts and registry for agent.

pub mod definition;
pub mod error;
pub mod registry;
pub mod tool;

pub use definition::{ToolDefinition, ToolInput, ToolName, ToolOutput};
pub use error::{ToolError, ToolNameError, ToolRegistryError};
pub use registry::ToolRegistry;
pub use tool::Tool;
