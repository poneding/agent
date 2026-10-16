use std::collections::BTreeMap;
use std::path::{Component, Path};

use ruse_policy::ApprovalPreviewSummary;
use ruse_sandbox::WorkspaceSandbox;
use serde::{Deserialize, Serialize};

use crate::{ToolError, ToolInput, ToolName};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const SUMMARY_LIMIT: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ToolPreview {
    FileEdit(FileEditPreview),
    Command(CommandPreview),
    Git(GitPreview),
    Mcp(McpPreview),
    Plugin(PluginPreview),
    BackgroundTask(BackgroundTaskPreview),
    Generic(GenericToolPreview),
}

impl ToolPreview {
    pub fn to_approval_preview_summary(&self) -> Result<ApprovalPreviewSummary, ToolError> {
        let (kind, resource, summary, content_hash, truncated, redacted) = match self {
            Self::FileEdit(preview) => (
                "file_edit",
                preview.path.clone(),
                format!(
                    "Edit {}: +{} -{} lines",
                    preview.path, preview.added_lines, preview.removed_lines
                ),
                Some(preview.content_hash.clone()),
                preview.truncated,
                false,
            ),
            Self::Command(preview) => (
                "command",
                preview.command_line(),
                format!(
                    "Run {} in {} with timeout {}ms",
                    preview.command_line(),
                    preview.cwd,
                    preview.timeout_ms
                ),
                None,
                preview.redaction.truncated,
                !preview.redaction.redacted_fields.is_empty(),
            ),
            Self::Git(preview) => (
                "git",
                preview.path.clone().unwrap_or_else(|| preview.cwd.clone()),
                preview.summary.clone(),
                None,
                false,
                false,
            ),
            Self::Mcp(preview) => (
                "mcp",
                format!("{}:{}", preview.server_id, preview.tool_name),
                preview.input_summary.clone(),
                Some(preview.input_hash.clone()),
                preview.redaction.truncated,
                !preview.redaction.redacted_fields.is_empty(),
            ),
            Self::Plugin(preview) => (
                "plugin",
                format!("{}:{}", preview.plugin_id, preview.tool_or_hook),
                preview.input_summary.clone(),
                Some(preview.input_hash.clone()),
                preview.redaction.truncated,
                !preview.redaction.redacted_fields.is_empty(),
            ),
            Self::BackgroundTask(preview) => (
                "background_task",
                preview.workspace.clone(),
                format!(
                    "Schedule {} with quota {}",
                    preview.schedule, preview.quota_summary
                ),
                None,
                false,
                false,
            ),
            Self::Generic(preview) => (
                "generic",
                preview.tool_name.clone(),
                preview.input_summary.clone(),
                Some(preview.input_hash.clone()),
                preview.redaction.truncated,
                !preview.redaction.redacted_fields.is_empty(),
            ),
        };

        let mut summary = ApprovalPreviewSummary::new(kind, resource, summary)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        if let Some(content_hash) = content_hash {
            summary = summary.with_content_hash(content_hash);
        }
        Ok(summary.with_truncated(truncated).with_redacted(redacted))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileEditPreview {
    pub path: String,
    pub diff: String,
    pub added_lines: u32,
    pub removed_lines: u32,
    pub content_hash: String,
    pub apply: bool,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandPreview {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub timeout_ms: u64,
    pub env: Vec<CommandEnvPreview>,
    pub sandbox_profile: String,
    pub redaction: PreviewRedactionReport,
}

impl CommandPreview {
    fn command_line(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandEnvPreview {
    pub key: String,
    pub value: String,
    pub redacted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitPreview {
    pub operation: String,
    pub cwd: String,
    pub path: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPreview {
    pub server_id: String,
    pub tool_name: String,
    pub input_hash: String,
    pub input_summary: String,
    pub redaction: PreviewRedactionReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginPreview {
    pub plugin_id: String,
    pub tool_or_hook: String,
    pub input_hash: String,
    pub input_summary: String,
    pub redaction: PreviewRedactionReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackgroundTaskPreview {
    pub schedule: String,
    pub workspace: String,
    pub quota_summary: String,
    pub capability_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GenericToolPreview {
    pub tool_name: String,
    pub input_hash: String,
    pub input_summary: String,
    pub redaction: PreviewRedactionReport,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreviewRedactionReport {
    pub redacted_fields: Vec<String>,
    pub truncated: bool,
    pub original_bytes: u64,
    pub preview_bytes: u64,
}

pub fn preview_tool_call(
    tool_name: &ToolName,
    input: &ToolInput,
    sandbox: Option<&WorkspaceSandbox>,
) -> Result<ToolPreview, ToolError> {
    match tool_name.as_str() {
        "workspace.edit_file" => preview_file_edit(input, require_sandbox(sandbox)?),
        "workspace.write_file" => preview_write_file(input, sandbox),
        "workspace.shell_command" => preview_shell_command(input, sandbox),
        "workspace.git_status" => preview_git_status(input, sandbox),
        "workspace.git_diff" => preview_git_diff(input, sandbox),
        value if value.starts_with("mcp.") => Ok(preview_mcp(value, input)),
        value if value.starts_with("plugin.") => Ok(preview_plugin(value, input)),
        value if value.starts_with("background.") => Ok(preview_background_task(value, input)),
        value => Ok(preview_generic(value, input)),
    }
}

fn preview_write_file(
    input: &ToolInput,
    sandbox: Option<&WorkspaceSandbox>,
) -> Result<ToolPreview, ToolError> {
    let path = string_field(&input.value, "path")?;
    let content = string_field(&input.value, "content")?;
    if let Some(sandbox) = sandbox {
        sandbox
            .resolve_write_path(&path)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
    }
    let diff = format!("--- {path}\n+++ {path}\n@@\n{}", added_lines(&content));

    Ok(ToolPreview::FileEdit(FileEditPreview {
        path,
        content_hash: stable_hash(&diff),
        diff,
        added_lines: content.lines().count() as u32,
        removed_lines: 0,
        apply: true,
        truncated: false,
    }))
}

fn preview_file_edit(
    input: &ToolInput,
    sandbox: &WorkspaceSandbox,
) -> Result<ToolPreview, ToolError> {
    let path = string_field(&input.value, "path")?;
    let replacement = string_field(&input.value, "replacement")?;
    let apply = input
        .value
        .get("apply")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let resolved = sandbox
        .resolve_read_path(&path)
        .map_err(|error| ToolError::Validation(error.to_string()))?;
    let current = std::fs::read_to_string(resolved)
        .map_err(|error| ToolError::Execution(error.to_string()))?;
    let diff = simple_diff(&path, &current, &replacement);
    let added_lines = replacement.lines().count() as u32;
    let removed_lines = current.lines().count() as u32;

    Ok(ToolPreview::FileEdit(FileEditPreview {
        path,
        content_hash: stable_hash(&diff),
        diff,
        added_lines,
        removed_lines,
        apply,
        truncated: false,
    }))
}

fn preview_shell_command(
    input: &ToolInput,
    sandbox: Option<&WorkspaceSandbox>,
) -> Result<ToolPreview, ToolError> {
    let program = string_field(&input.value, "program")?;
    let args = parse_args(input.value.get("args"))?;
    let cwd = input
        .value
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".")
        .to_string();
    if let Some(sandbox) = sandbox {
        let resolved = sandbox
            .resolve_read_path(&cwd)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        if !resolved.is_dir() {
            return Err(ToolError::Validation(format!(
                "cwd is not a directory: {cwd}"
            )));
        }
    }
    let timeout_ms = input
        .value
        .get("timeout_ms")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(DEFAULT_TIMEOUT_MS);
    let env = parse_env(input.value.get("env"))?;
    let redacted_fields = env
        .keys()
        .map(|key| format!("env.{key}"))
        .collect::<Vec<_>>();
    let env = env
        .into_keys()
        .map(|key| CommandEnvPreview {
            key,
            value: "<redacted>".to_string(),
            redacted: true,
        })
        .collect();
    let redaction = PreviewRedactionReport {
        redacted_fields,
        truncated: false,
        original_bytes: serde_json_len(&input.value),
        preview_bytes: 0,
    };

    Ok(ToolPreview::Command(CommandPreview {
        program,
        args,
        cwd,
        timeout_ms,
        env,
        sandbox_profile: "workspace".to_string(),
        redaction,
    }))
}

fn preview_git_status(
    input: &ToolInput,
    sandbox: Option<&WorkspaceSandbox>,
) -> Result<ToolPreview, ToolError> {
    let cwd = cwd_field(input);
    validate_cwd(&cwd, sandbox)?;
    Ok(ToolPreview::Git(GitPreview {
        operation: "status".to_string(),
        cwd,
        path: None,
        summary: "Inspect Git status".to_string(),
    }))
}

fn preview_git_diff(
    input: &ToolInput,
    sandbox: Option<&WorkspaceSandbox>,
) -> Result<ToolPreview, ToolError> {
    let cwd = cwd_field(input);
    validate_cwd(&cwd, sandbox)?;
    let path = input
        .value
        .get("path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".")
        .to_string();
    validate_repo_path(&path)?;
    Ok(ToolPreview::Git(GitPreview {
        operation: "diff".to_string(),
        cwd,
        summary: format!("Inspect Git diff for {path}"),
        path: Some(path),
    }))
}

fn preview_mcp(tool_name: &str, input: &ToolInput) -> ToolPreview {
    let mut parts = tool_name.split('.');
    let _mcp = parts.next();
    let server_id = parts.next().unwrap_or("unknown").to_string();
    let remote_tool = parts.collect::<Vec<_>>().join(".");
    let (input_summary, redaction) = summarize_input(&input.value);
    ToolPreview::Mcp(McpPreview {
        server_id,
        tool_name: if remote_tool.is_empty() {
            "unknown".to_string()
        } else {
            remote_tool
        },
        input_hash: stable_hash(&canonical_json(&input.value)),
        input_summary,
        redaction,
    })
}

fn preview_plugin(tool_name: &str, input: &ToolInput) -> ToolPreview {
    let mut parts = tool_name.split('.');
    let _plugin = parts.next();
    let plugin_id = parts.next().unwrap_or("unknown").to_string();
    let tool_or_hook = parts.collect::<Vec<_>>().join(".");
    let (input_summary, redaction) = summarize_input(&input.value);
    ToolPreview::Plugin(PluginPreview {
        plugin_id,
        tool_or_hook: if tool_or_hook.is_empty() {
            "unknown".to_string()
        } else {
            tool_or_hook
        },
        input_hash: stable_hash(&canonical_json(&input.value)),
        input_summary,
        redaction,
    })
}

fn preview_background_task(_tool_name: &str, input: &ToolInput) -> ToolPreview {
    ToolPreview::BackgroundTask(BackgroundTaskPreview {
        schedule: string_field_or(&input.value, "schedule", "one_shot"),
        workspace: string_field_or(&input.value, "workspace", "current"),
        quota_summary: string_field_or(&input.value, "quota", "default"),
        capability_summary: string_field_or(&input.value, "capability", "declared"),
    })
}

fn preview_generic(tool_name: &str, input: &ToolInput) -> ToolPreview {
    let (input_summary, redaction) = summarize_input(&input.value);
    ToolPreview::Generic(GenericToolPreview {
        tool_name: tool_name.to_string(),
        input_hash: stable_hash(&canonical_json(&input.value)),
        input_summary,
        redaction,
    })
}

fn require_sandbox(sandbox: Option<&WorkspaceSandbox>) -> Result<&WorkspaceSandbox, ToolError> {
    sandbox.ok_or_else(|| ToolError::Validation("workspace root is required".to_string()))
}

fn string_field(value: &serde_json::Value, field: &str) -> Result<String, ToolError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| ToolError::Validation(format!("{field} must be a non-empty string")))
}

fn string_field_or(value: &serde_json::Value, field: &str, fallback: &str) -> String {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn cwd_field(input: &ToolInput) -> String {
    input
        .value
        .get("cwd")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".")
        .to_string()
}

fn validate_cwd(cwd: &str, sandbox: Option<&WorkspaceSandbox>) -> Result<(), ToolError> {
    if let Some(sandbox) = sandbox {
        let resolved = sandbox
            .resolve_read_path(cwd)
            .map_err(|error| ToolError::Validation(error.to_string()))?;
        if !resolved.is_dir() {
            return Err(ToolError::Validation(format!(
                "cwd is not a directory: {cwd}"
            )));
        }
    }
    Ok(())
}

fn validate_repo_path(path: &str) -> Result<(), ToolError> {
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

fn parse_args(value: Option<&serde_json::Value>) -> Result<Vec<String>, ToolError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let args = value
        .as_array()
        .ok_or_else(|| ToolError::Validation("args must be an array".to_string()))?;
    args.iter()
        .map(|value| {
            value
                .as_str()
                .map(std::string::ToString::to_string)
                .ok_or_else(|| ToolError::Validation("args must contain only strings".to_string()))
        })
        .collect()
}

fn parse_env(value: Option<&serde_json::Value>) -> Result<BTreeMap<String, String>, ToolError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let env = value
        .as_object()
        .ok_or_else(|| ToolError::Validation("env must be an object".to_string()))?;
    env.iter()
        .map(|(key, value)| {
            let value = value.as_str().ok_or_else(|| {
                ToolError::Validation("env values must contain only strings".to_string())
            })?;
            Ok((key.clone(), value.to_string()))
        })
        .collect()
}

fn summarize_input(value: &serde_json::Value) -> (String, PreviewRedactionReport) {
    let mut redacted_fields = Vec::new();
    let redacted = redact_secret_like_fields(value, "", &mut redacted_fields);
    redacted_fields.sort();
    let original = canonical_json(value);
    let redacted_json = canonical_json(&redacted);
    let (input_summary, truncated) = truncate_summary(&redacted_json);
    let preview_bytes = input_summary.len() as u64;
    (
        input_summary,
        PreviewRedactionReport {
            redacted_fields,
            truncated,
            original_bytes: original.len() as u64,
            preview_bytes,
        },
    )
}

fn redact_secret_like_fields(
    value: &serde_json::Value,
    path: &str,
    redacted_fields: &mut Vec<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in object {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if is_secret_like_key(key) {
                    redacted.insert(key.clone(), serde_json::Value::String("<redacted>".into()));
                    redacted_fields.push(field_path);
                } else {
                    redacted.insert(
                        key.clone(),
                        redact_secret_like_fields(value, &field_path, redacted_fields),
                    );
                }
            }
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    redact_secret_like_fields(value, &format!("{path}[{index}]"), redacted_fields)
                })
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn truncate_summary(value: &str) -> (String, bool) {
    if value.len() <= SUMMARY_LIMIT {
        return (value.to_string(), false);
    }
    (format!("{}...", &value[..SUMMARY_LIMIT]), true)
}

fn is_secret_like_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("secret")
        || key.contains("token")
        || key.contains("password")
        || key.contains("api_key")
        || key.contains("apikey")
}

fn simple_diff(path: &str, current: &str, replacement: &str) -> String {
    format!(
        "--- {path}\n+++ {path}\n@@\n{}{}",
        current
            .lines()
            .map(|line| format!("-{line}\n"))
            .collect::<String>(),
        added_lines(replacement)
    )
}

fn added_lines(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("+{line}\n"))
        .collect::<String>()
}

fn canonical_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).expect("serializing serde_json::Value cannot fail")
}

fn serde_json_len(value: &serde_json::Value) -> u64 {
    canonical_json(value).len() as u64
}

fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}
