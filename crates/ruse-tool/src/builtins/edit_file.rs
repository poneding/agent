use ruse_sandbox::WorkspaceSandbox;
use serde_json::json;

use crate::builtins::read_file::input_path;
use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct FileEditTool {
    sandbox: WorkspaceSandbox,
}

impl FileEditTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for FileEditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.edit_file").expect("static tool name is valid"),
            description: "Preview or apply a workspace-confined file replacement".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "replacement"],
                "properties": {
                    "path": { "type": "string" },
                    "expected": { "type": "string" },
                    "replacement": { "type": "string" },
                    "apply": { "type": "boolean" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let path = input_path(&input)?;
        let replacement = input
            .value
            .get("replacement")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ToolError::Validation("replacement must be a string".to_string()))?;
        let expected = input
            .value
            .get("expected")
            .and_then(serde_json::Value::as_str);
        let apply = input
            .value
            .get("apply")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let resolved = self
            .sandbox
            .resolve_read_path(path)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        let current = std::fs::read_to_string(&resolved)
            .map_err(|error| ToolError::Execution(error.to_string()))?;

        if expected.is_some_and(|expected| expected != current) {
            return Err(ToolError::Validation(
                "expected content did not match current file".to_string(),
            ));
        }

        let diff = simple_diff(path, &current, replacement);
        if apply {
            std::fs::write(&resolved, replacement)
                .map_err(|error| ToolError::Execution(error.to_string()))?;
        }

        Ok(ToolOutput {
            value: json!({
                "path": path,
                "applied": apply,
                "diff": diff
            }),
        })
    }
}

fn simple_diff(path: &str, current: &str, replacement: &str) -> String {
    format!(
        "--- {path}\n+++ {path}\n@@\n{}{}",
        removed_lines(current),
        added_lines(replacement)
    )
}

fn removed_lines(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("-{line}\n"))
        .collect::<String>()
}

fn added_lines(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("+{line}\n"))
        .collect::<String>()
}
