use std::path::{Path, PathBuf};

use ruse_sandbox::WorkspaceSandbox;
use serde_json::json;

use crate::builtins::read_file::input_path;
use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct SearchTextTool {
    sandbox: WorkspaceSandbox,
}

impl SearchTextTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for SearchTextTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.search_text").expect("static tool name is valid"),
            description: "Search UTF-8 workspace files for a text query".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "query"],
                "properties": {
                    "path": { "type": "string" },
                    "query": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let path = input_path(&input)?;
        let query = input
            .value
            .get("query")
            .and_then(serde_json::Value::as_str)
            .filter(|query| !query.is_empty())
            .ok_or_else(|| ToolError::Validation("query must be a non-empty string".to_string()))?;
        let resolved = self
            .sandbox
            .resolve_read_path(path)
            .map_err(|error| ToolError::Validation(error.to_string()))?;

        let mut files = Vec::new();
        collect_files(&resolved, &mut files)?;
        files.sort();

        let mut matches = Vec::new();
        for file in files {
            let content = std::fs::read_to_string(&file)
                .map_err(|error| ToolError::Execution(error.to_string()))?;
            for (index, line) in content.lines().enumerate() {
                if line.contains(query) {
                    matches.push(json!({
                        "path": relative_path(self.sandbox.root(), &file)?,
                        "line": index + 1,
                        "text": line
                    }));
                }
            }
        }

        Ok(ToolOutput {
            value: json!({ "matches": matches }),
        })
    }
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), ToolError> {
    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }

    let mut entries = std::fs::read_dir(path)
        .map_err(|error| ToolError::Execution(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ToolError::Execution(error.to_string()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }

    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, ToolError> {
    path.strip_prefix(root)
        .map_err(|error| ToolError::Execution(error.to_string()))
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}
