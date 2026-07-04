#![forbid(unsafe_code)]
//! Tool contracts and registry for agent.

pub mod builtins;
pub mod definition;
pub mod error;
pub mod preview;
pub mod registry;
pub mod tool;

pub use definition::{ToolDefinition, ToolInput, ToolName, ToolOutput};
pub use error::{ToolError, ToolNameError, ToolRegistryError};
pub use preview::{
    BackgroundTaskPreview, CommandEnvPreview, CommandPreview, FileEditPreview, GenericToolPreview,
    GitPreview, McpPreview, PluginPreview, PreviewRedactionReport, ToolPreview, preview_tool_call,
};
pub use registry::ToolRegistry;
pub use tool::Tool;
