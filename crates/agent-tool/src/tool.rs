use crate::definition::{ToolDefinition, ToolInput, ToolOutput};
use crate::error::ToolError;

pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError>;
}
