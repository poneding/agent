use std::collections::BTreeSet;
use std::path::PathBuf;

use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_tool::builtins::{FileEditTool, GitDiffTool, GitStatusResult, GitStatusTool};
use ruse_tool::{ToolRegistry, preview_tool_call};
use ruse_types::{ConversationId, RuntimeEvent, RuntimeEventPayload, WorkspaceId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopStatus};

const DIFF_PREVIEW_LIMIT: usize = 4_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditDiffFlowInput {
    pub workspace_root: PathBuf,
    pub path: String,
    pub expected: Option<String>,
    pub replacement: String,
    pub apply: bool,
    pub ids: RuntimeIds,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EditDiffFlowReport {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub path: String,
    pub applied: bool,
    pub preview: EditPreviewSummary,
    pub changed_files: Vec<String>,
    pub diff: DiffSummary,
    pub events: Vec<RuntimeEvent>,
    pub diagnostics: Vec<EditDiffDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EditPreviewSummary {
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub state: DiffEvidenceState,
    pub changed_file_count: u32,
    pub diff_line_count: u32,
    pub added_line_count: u32,
    pub removed_line_count: u32,
    pub preview: String,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffEvidenceState {
    Preview,
    Applied,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EditDiffDiagnostic {
    pub severity: EditDiffDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditDiffDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EditDiffFlowError {
    #[error("edit diff workspace is invalid")]
    InvalidWorkspace {
        diagnostics: Vec<EditDiffDiagnostic>,
    },
    #[error("edit diff tool failed")]
    ToolFailed {
        diagnostics: Vec<EditDiffDiagnostic>,
        events: Vec<RuntimeEvent>,
    },
}

impl EditDiffFlowError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidWorkspace { .. } => "edit_diff.workspace.invalid",
            Self::ToolFailed { .. } => "edit_diff.tool_failed",
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[EditDiffDiagnostic] {
        match self {
            Self::InvalidWorkspace { diagnostics } | Self::ToolFailed { diagnostics, .. } => {
                diagnostics
            }
        }
    }

    #[must_use]
    pub fn events(&self) -> &[RuntimeEvent] {
        match self {
            Self::InvalidWorkspace { .. } => &[],
            Self::ToolFailed { events, .. } => events,
        }
    }
}

pub fn run_edit_diff_flow(
    input: EditDiffFlowInput,
) -> Result<EditDiffFlowReport, EditDiffFlowError> {
    let sandbox = WorkspaceSandbox::new(&input.workspace_root).map_err(|_| {
        EditDiffFlowError::InvalidWorkspace {
            diagnostics: vec![EditDiffDiagnostic::error(
                "edit_diff.workspace.invalid",
                "workspace root does not exist or cannot be accessed",
                false,
            )],
        }
    })?;
    let mut registry = ToolRegistry::default();
    registry
        .register(FileEditTool::new(sandbox.clone()))
        .map_err(|error| tool_setup_error(error.to_string()))?;
    registry
        .register(GitStatusTool::new(sandbox.clone()))
        .map_err(|error| tool_setup_error(error.to_string()))?;
    registry
        .register(GitDiffTool::new(sandbox.clone()))
        .map_err(|error| tool_setup_error(error.to_string()))?;
    let policy = allow_policy([
        "workspace.edit_file",
        "workspace.git_status",
        "workspace.git_diff",
    ]);

    let edit_input = edit_tool_input(&input);
    let preview = edit_preview_summary(&input, &sandbox)?;
    let edit_call = ToolCallRequest {
        call_id: "edit-diff-edit".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        input_json: edit_input,
    };
    let mut events = Vec::new();
    let edit_outcome = ToolLoop
        .run(input.ids, edit_call, &registry, &policy)
        .map_err(|error| EditDiffFlowError::ToolFailed {
            diagnostics: vec![EditDiffDiagnostic::error(
                "edit_diff.tool_loop.failed",
                error.to_string(),
                true,
            )],
            events: Vec::new(),
        })?;
    events.extend(edit_outcome.events);
    if edit_outcome.status != ToolLoopStatus::Completed {
        return Err(EditDiffFlowError::ToolFailed {
            diagnostics: vec![EditDiffDiagnostic::error(
                "edit_diff.tool_failed",
                "file edit tool did not complete",
                true,
            )],
            events,
        });
    }

    let edit_output = completed_output(&events, "edit-diff-edit").ok_or_else(|| {
        EditDiffFlowError::ToolFailed {
            diagnostics: vec![EditDiffDiagnostic::error(
                "edit_diff.tool_output.missing",
                "file edit output was not recorded",
                true,
            )],
            events: events.clone(),
        }
    })?;
    let applied = edit_output
        .get("applied")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let edit_diff = edit_output
        .get("diff")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    let mut diagnostics = Vec::new();
    let mut changed_files = Vec::new();
    let diff = if applied {
        match collect_applied_git_evidence(input.ids, &registry, &policy, &input.path) {
            Ok(git_evidence) => {
                events.extend(git_evidence.events);
                changed_files = git_evidence.changed_files;
                git_evidence.diff
            }
            Err(mut git_error) => {
                events.append(&mut git_error.events);
                diagnostics.append(&mut git_error.diagnostics);
                changed_files = vec![input.path.clone()];
                unavailable_diff()
            }
        }
    } else {
        diff_summary(DiffEvidenceState::Preview, edit_diff)
    };

    Ok(EditDiffFlowReport {
        workspace_id: input.ids.workspace_id,
        conversation_id: input.ids.conversation_id,
        path: input.path,
        applied,
        preview,
        changed_files,
        diff,
        events,
        diagnostics,
    })
}

fn edit_tool_input(input: &EditDiffFlowInput) -> serde_json::Value {
    let mut value = json!({
        "path": input.path,
        "replacement": input.replacement,
        "apply": input.apply,
    });
    if let Some(expected) = &input.expected {
        value["expected"] = serde_json::Value::String(expected.clone());
    }
    value
}

fn edit_preview_summary(
    input: &EditDiffFlowInput,
    sandbox: &WorkspaceSandbox,
) -> Result<EditPreviewSummary, EditDiffFlowError> {
    let tool_name = ruse_tool::ToolName::new("workspace.edit_file").map_err(|error| {
        EditDiffFlowError::ToolFailed {
            diagnostics: vec![EditDiffDiagnostic::error(
                "edit_diff.preview.failed",
                error.to_string(),
                true,
            )],
            events: Vec::new(),
        }
    })?;
    let preview = preview_tool_call(
        &tool_name,
        &ruse_tool::ToolInput {
            value: edit_tool_input(input),
        },
        Some(sandbox),
    )
    .map_err(|error| EditDiffFlowError::ToolFailed {
        diagnostics: vec![EditDiffDiagnostic::error(
            "edit_diff.preview.failed",
            error.to_string(),
            true,
        )],
        events: Vec::new(),
    })?;
    let summary =
        preview
            .to_approval_preview_summary()
            .map_err(|error| EditDiffFlowError::ToolFailed {
                diagnostics: vec![EditDiffDiagnostic::error(
                    "edit_diff.preview.failed",
                    error.to_string(),
                    true,
                )],
                events: Vec::new(),
            })?;
    Ok(EditPreviewSummary {
        resource: summary.resource,
        summary: redact_secret_like_text(&summary.summary),
        content_hash: summary.content_hash,
        truncated: summary.truncated,
        redacted: summary.redacted,
    })
}

struct GitEvidence {
    changed_files: Vec<String>,
    diff: DiffSummary,
    events: Vec<RuntimeEvent>,
}

struct GitEvidenceError {
    diagnostics: Vec<EditDiffDiagnostic>,
    events: Vec<RuntimeEvent>,
}

fn collect_applied_git_evidence(
    ids: RuntimeIds,
    registry: &ToolRegistry,
    policy: &StaticPolicyEngine,
    path: &str,
) -> Result<GitEvidence, GitEvidenceError> {
    let mut events = Vec::new();
    let status_outcome = ToolLoop
        .run(
            ids,
            ToolCallRequest {
                call_id: "edit-diff-git-status".to_string(),
                tool_name: "workspace.git_status".to_string(),
                input_json: json!({ "cwd": "." }),
            },
            registry,
            policy,
        )
        .map_err(|error| GitEvidenceError {
            diagnostics: vec![git_unavailable(error.to_string())],
            events: Vec::new(),
        })?;
    events.extend(status_outcome.events);
    if status_outcome.status != ToolLoopStatus::Completed {
        return Err(GitEvidenceError {
            diagnostics: vec![git_unavailable("git status did not complete")],
            events,
        });
    }
    let changed_files = completed_output(&events, "edit-diff-git-status")
        .and_then(|value| serde_json::from_value::<GitStatusResult>(value.clone()).ok())
        .map(|status| {
            status
                .files
                .into_iter()
                .map(|file| file.path)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let diff_outcome = ToolLoop
        .run(
            ids,
            ToolCallRequest {
                call_id: "edit-diff-git-diff".to_string(),
                tool_name: "workspace.git_diff".to_string(),
                input_json: json!({ "cwd": ".", "path": path }),
            },
            registry,
            policy,
        )
        .map_err(|error| GitEvidenceError {
            diagnostics: vec![git_unavailable(error.to_string())],
            events: events.clone(),
        })?;
    events.extend(diff_outcome.events);
    if diff_outcome.status != ToolLoopStatus::Completed {
        return Err(GitEvidenceError {
            diagnostics: vec![git_unavailable("git diff did not complete")],
            events,
        });
    }
    let diff_text = completed_output(&events, "edit-diff-git-diff")
        .and_then(|value| value.get("diff"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    Ok(GitEvidence {
        changed_files,
        diff: diff_summary(DiffEvidenceState::Applied, diff_text),
        events,
    })
}

fn completed_output<'a>(
    events: &'a [RuntimeEvent],
    call_id: &str,
) -> Option<&'a serde_json::Value> {
    events.iter().find_map(|event| match &event.payload {
        RuntimeEventPayload::ToolExecutionCompleted(completed) if completed.call_id == call_id => {
            Some(&completed.output_json)
        }
        _ => None,
    })
}

fn diff_summary(state: DiffEvidenceState, diff: &str) -> DiffSummary {
    let redacted = redact_secret_like_text(diff);
    let truncated = redacted.chars().count() > DIFF_PREVIEW_LIMIT;
    let preview = if truncated {
        redacted.chars().take(DIFF_PREVIEW_LIMIT).collect()
    } else {
        redacted
    };
    let added_line_count = diff
        .lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    let removed_line_count = diff
        .lines()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    DiffSummary {
        state,
        changed_file_count: if added_line_count > 0 || removed_line_count > 0 {
            1
        } else {
            0
        },
        diff_line_count: diff.lines().count().try_into().unwrap_or(u32::MAX),
        added_line_count,
        removed_line_count,
        preview,
        truncated,
        redacted: preview_contains_redaction(diff),
    }
}

fn unavailable_diff() -> DiffSummary {
    DiffSummary {
        state: DiffEvidenceState::Unavailable,
        changed_file_count: 0,
        diff_line_count: 0,
        added_line_count: 0,
        removed_line_count: 0,
        preview: String::new(),
        truncated: false,
        redacted: false,
    }
}

fn allow_policy<const N: usize>(tool_names: [&str; N]) -> StaticPolicyEngine {
    let mut policy = StaticPolicyEngine::default();
    for tool_name in tool_names {
        policy = policy.with_decision(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::Tool, tool_name).expect("static scope"),
            PolicyDecision::Allow {
                reason: "edit diff flow allows built-in workspace evidence tool".to_string(),
            },
        );
    }
    policy
}

fn tool_setup_error(message: String) -> EditDiffFlowError {
    EditDiffFlowError::ToolFailed {
        diagnostics: vec![EditDiffDiagnostic::error(
            "edit_diff.tool_setup.failed",
            message,
            false,
        )],
        events: Vec::new(),
    }
}

fn git_unavailable(message: impl Into<String>) -> EditDiffDiagnostic {
    EditDiffDiagnostic::warning("edit_diff.git_diff.unavailable", message.into(), true)
}

impl EditDiffDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: EditDiffDiagnosticSeverity::Warning,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: EditDiffDiagnosticSeverity::Error,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }
}

fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            if is_secret_like(word) {
                "[redacted]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn preview_contains_redaction(input: &str) -> bool {
    input.split_whitespace().any(is_secret_like)
}

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
