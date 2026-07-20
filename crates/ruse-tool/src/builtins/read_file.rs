use ruse_sandbox::WorkspaceSandbox;
use serde_json::json;

use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct ReadFileTool {
    sandbox: WorkspaceSandbox,
}

impl ReadFileTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.read_file").expect("static tool name is valid"),
            description: "Read a UTF-8 file from the workspace".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let path = input_path(&input)?;
        let resolved = self
            .sandbox
            .resolve_read_path(path)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        let content = std::fs::read_to_string(&resolved)
            .map_err(|error| ToolError::Execution(error.to_string()))?;

        Ok(ToolOutput {
            value: json!({
                "path": path,
                "content": content
            }),
        })
    }
}

pub(crate) fn input_path(input: &ToolInput) -> Result<&str, ToolError> {
    input
        .value
        .get("path")
        .and_then(serde_json::Value::as_str)
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| ToolError::Validation("path must be a non-empty string".to_string()))
}
