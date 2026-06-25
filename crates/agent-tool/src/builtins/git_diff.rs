use agent_sandbox::WorkspaceSandbox;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::builtins::git_status::{resolve_cwd, run_git, validate_git_path};
use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct GitDiffTool {
    sandbox: WorkspaceSandbox,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitDiffResult {
    pub diff: String,
}

impl GitDiffTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for GitDiffTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.git_diff").expect("static tool name is valid"),
            description: "Inspect read-only Git diff for a workspace repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cwd": { "type": "string" },
                    "path": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let cwd = resolve_cwd(&self.sandbox, &input)?;
        let path = input
            .value
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(".");
        validate_git_path(path)?;

        let diff = run_git(&cwd, "diff", &["diff", "--no-ext-diff", "--", path])?;
        Ok(ToolOutput {
            value: serde_json::to_value(GitDiffResult { diff })
                .map_err(|error| ToolError::Execution(error.to_string()))?,
        })
    }
}
