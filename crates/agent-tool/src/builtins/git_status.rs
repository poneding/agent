use std::path::{Component, Path, PathBuf};
use std::process::Command;

use agent_sandbox::WorkspaceSandbox;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{Tool, ToolDefinition, ToolError, ToolInput, ToolName, ToolOutput};

#[derive(Clone, Debug)]
pub struct GitStatusTool {
    sandbox: WorkspaceSandbox,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitStatusResult {
    pub files: Vec<GitStatusFile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitStatusFile {
    pub path: String,
    pub status: String,
}

impl GitStatusTool {
    #[must_use]
    pub fn new(sandbox: WorkspaceSandbox) -> Self {
        Self { sandbox }
    }
}

impl Tool for GitStatusTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("workspace.git_status").expect("static tool name is valid"),
            description: "Inspect read-only Git status for a workspace repository".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cwd": { "type": "string" }
                },
                "additionalProperties": false
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let cwd = resolve_cwd(&self.sandbox, &input)?;
        let stdout = run_git(&cwd, "status", &["status", "--porcelain=v1"])?;
        let output = GitStatusResult {
            files: parse_status(&stdout),
        };

        Ok(ToolOutput {
            value: serde_json::to_value(output)
                .map_err(|error| ToolError::Execution(error.to_string()))?,
        })
    }
}

pub(crate) fn resolve_cwd(
    sandbox: &WorkspaceSandbox,
    input: &ToolInput,
) -> Result<PathBuf, ToolError> {
    let cwd = input
        .value
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".");
    let resolved = sandbox
        .resolve_read_path(cwd)
        .map_err(|error| ToolError::Validation(error.to_string()))?;
    if !resolved.is_dir() {
        return Err(ToolError::Validation(format!(
            "cwd is not a directory: {cwd}"
        )));
    }
    Ok(resolved)
}

pub(crate) fn validate_git_path(path: &str) -> Result<(), ToolError> {
    if path.trim().is_empty() {
        return Err(ToolError::Validation(
            "path must be a non-empty string".to_string(),
        ));
    }
    let path = Path::new(path);
    if path.is_absolute() {
        return Err(ToolError::Validation(
            "path must be relative to the repository".to_string(),
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(ToolError::Validation(
            "path must not escape the repository".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn run_git(cwd: &Path, action: &str, args: &[&str]) -> Result<String, ToolError> {
    let output = Command::new(resolve_program("git"))
        .args(["-c", "color.ui=false", "-c", "core.quotePath=false"])
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .output()
        .map_err(|error| ToolError::Execution(error.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Execution(format!(
            "git {action} failed: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_status(stdout: &str) -> Vec<GitStatusFile> {
    stdout
        .lines()
        .filter(|line| line.len() >= 4)
        .map(|line| GitStatusFile {
            status: status_label(&line[..2]).to_string(),
            path: line[3..].to_string(),
        })
        .collect()
}

fn status_label(status: &str) -> &'static str {
    let mut chars = status.chars();
    let index = chars.next().unwrap_or(' ');
    let worktree = chars.next().unwrap_or(' ');

    if index == '?' && worktree == '?' {
        "untracked"
    } else if index == 'D' || worktree == 'D' {
        "deleted"
    } else if index == 'A' || worktree == 'A' {
        "added"
    } else if index == 'R' || worktree == 'R' {
        "renamed"
    } else if index == 'C' || worktree == 'C' {
        "copied"
    } else if index == 'M' || worktree == 'M' {
        "modified"
    } else if index == 'U' || worktree == 'U' {
        "unmerged"
    } else {
        "changed"
    }
}

fn resolve_program(program: &str) -> PathBuf {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .flat_map(|path| executable_candidates(&path, program))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(program))
}

fn executable_candidates(path: &Path, program: &str) -> Vec<PathBuf> {
    let candidate = path.join(program);
    #[cfg(windows)]
    {
        let mut candidates = vec![candidate.clone()];
        if candidate.extension().is_none() {
            candidates.push(path.join(format!("{program}.exe")));
        }
        candidates
    }
    #[cfg(not(windows))]
    {
        vec![candidate]
    }
}
