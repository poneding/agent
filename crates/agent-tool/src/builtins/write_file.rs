use agent_sandbox::WorkspaceSandbox;
use serde_json::json;

use crate::builtins::read_file::input_path;
use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct WriteFileTool {
    sandbox: WorkspaceSandbox,
}

impl WriteFileTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for WriteFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.write_file").expect("static tool name is valid"),
            description: "Create or overwrite a UTF-8 file inside the workspace".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let path = input_path(&input)?;
        let content = input
            .value
            .get("content")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ToolError::Validation("content must be a string".to_string()))?;
        let resolved = self
            .sandbox
            .resolve_write_path(path)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ToolError::Execution(error.to_string()))?;
        }
        std::fs::write(&resolved, content)
            .map_err(|error| ToolError::Execution(error.to_string()))?;

        Ok(ToolOutput {
            value: json!({
                "path": path,
                "bytes": content.len()
            }),
        })
    }
}
