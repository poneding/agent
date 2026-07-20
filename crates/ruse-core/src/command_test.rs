use std::path::PathBuf;

use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_tool::ToolRegistry;
use ruse_tool::builtins::{ShellCommandResult, ShellCommandTool};
use ruse_types::{ConversationId, RuntimeEvent, RuntimeEventPayload, WorkspaceId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{RuntimeIds, ToolCallRequest, ToolLoop, ToolLoopStatus};

const COMMAND_OUTPUT_LIMIT: usize = 4_000;
const OBSERVATION_LIMIT: usize = 4_000;
pub const DEFAULT_COMMAND_TEST_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandTestLoopInput {
    pub workspace_root: PathBuf,
    pub command_run_id: Option<String>,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub timeout_ms: u64,
    pub classify_as_test: bool,
    pub ids: RuntimeIds,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandTestLoopReport {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    #[serde(default = "legacy_command_run_id")]
    pub command_run_id: String,
    pub command: CommandIntentSummary,
    pub status: CommandExecutionStatus,
    pub classification: CommandFailureClassification,
    pub exit_code: Option<i32>,
    pub stdout: BoundedCommandOutput,
    pub stderr: BoundedCommandOutput,
    pub timed_out: bool,
    pub cancellation: CommandCancellationState,
    pub observation: CommandObservationSummary,
    pub events: Vec<RuntimeEvent>,
    pub diagnostics: Vec<CommandTestDiagnostic>,
}

fn legacy_command_run_id() -> String {
    "legacy-command-run".to_string()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandIntentSummary {
    pub display: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default = "default_command_timeout_ms")]
    pub timeout_ms: u64,
    pub cwd: String,
    pub classify_as_test: bool,
}

fn default_command_timeout_ms() -> u64 {
    DEFAULT_COMMAND_TEST_TIMEOUT_MS
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandExecutionStatus {
    Succeeded,
    Failed,
    TimedOut,
    WaitingApproval,
    Denied,
    ToolFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandFailureClassification {
    None,
    TestFailure,
    CommandFailure,
    Timeout,
    PolicyDenied,
    WaitingApproval,
    ToolFailure,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BoundedCommandOutput {
    pub text: String,
    pub byte_count: u64,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandCancellationState {
    pub cancellable: bool,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandObservationSummary {
    pub message: String,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandTestDiagnostic {
    pub severity: CommandTestDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandTestDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CommandTestLoopError {
    #[error("command test workspace is invalid")]
    InvalidWorkspace {
        diagnostics: Vec<CommandTestDiagnostic>,
    },
    #[error("command test tool failed")]
    ToolFailed {
        diagnostics: Vec<CommandTestDiagnostic>,
        events: Vec<RuntimeEvent>,
    },
}

impl CommandTestLoopError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidWorkspace { .. } => "command_test.workspace.invalid",
            Self::ToolFailed { .. } => "command_test.tool_failed",
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[CommandTestDiagnostic] {
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

pub fn run_command_test_loop(
    input: CommandTestLoopInput,
) -> Result<CommandTestLoopReport, CommandTestLoopError> {
    validate_command_input(&input)?;
    let sandbox = WorkspaceSandbox::new(&input.workspace_root).map_err(|_| {
        CommandTestLoopError::InvalidWorkspace {
            diagnostics: vec![CommandTestDiagnostic::error(
                "command_test.workspace.invalid",
                "workspace root does not exist or cannot be accessed",
                false,
            )],
        }
    })?;

    let mut registry = ToolRegistry::default();
    registry
        .register(ShellCommandTool::new(sandbox))
        .map_err(|error| tool_setup_error(error.to_string()))?;
    let policy = allow_policy(["workspace.shell_command"]);

    let command = CommandIntentSummary {
        display: command_display(&input.program, &input.args),
        program: Some(redact_secret_like_text(&input.program)),
        args: input
            .args
            .iter()
            .map(|arg| redact_secret_like_text(arg))
            .collect(),
        timeout_ms: input.timeout_ms,
        cwd: redact_secret_like_text(&input.cwd),
        classify_as_test: input.classify_as_test,
    };
    let call = ToolCallRequest {
        call_id: "command-test-shell".to_string(),
        tool_name: "workspace.shell_command".to_string(),
        input_json: json!({
            "program": input.program,
            "args": input.args,
            "cwd": input.cwd,
            "timeout_ms": input.timeout_ms,
            "env": {},
        }),
    };

    let outcome = ToolLoop
        .run(input.ids, call, &registry, &policy)
        .map_err(|error| CommandTestLoopError::ToolFailed {
            diagnostics: vec![CommandTestDiagnostic::error(
                "command_test.tool_loop.failed",
                error.to_string(),
                true,
            )],
            events: Vec::new(),
        })?;
    let events = outcome.events;

    match outcome.status {
        ToolLoopStatus::Completed => {
            let output = completed_output(&events, "command-test-shell").ok_or_else(|| {
                CommandTestLoopError::ToolFailed {
                    diagnostics: vec![CommandTestDiagnostic::error(
                        "command_test.tool_output.missing",
                        "shell command output was not recorded",
                        true,
                    )],
                    events: events.clone(),
                }
            })?;
            let shell =
                serde_json::from_value::<ShellCommandResult>(output.clone()).map_err(|error| {
                    CommandTestLoopError::ToolFailed {
                        diagnostics: vec![CommandTestDiagnostic::error(
                            "command_test.tool_output.invalid",
                            error.to_string(),
                            true,
                        )],
                        events: events.clone(),
                    }
                })?;
            Ok(report_from_shell_result(
                input.ids,
                command,
                input.classify_as_test,
                input
                    .command_run_id
                    .clone()
                    .unwrap_or_else(|| input.ids.correlation_id.to_string()),
                shell,
                events,
            ))
        }
        ToolLoopStatus::Denied => Ok(policy_report(
            input.ids,
            input
                .command_run_id
                .clone()
                .unwrap_or_else(|| input.ids.correlation_id.to_string()),
            command,
            CommandExecutionStatus::Denied,
            CommandFailureClassification::PolicyDenied,
            events,
            "command execution was denied by policy",
        )),
        ToolLoopStatus::WaitingApproval => Ok(policy_report(
            input.ids,
            input
                .command_run_id
                .clone()
                .unwrap_or_else(|| input.ids.correlation_id.to_string()),
            command,
            CommandExecutionStatus::WaitingApproval,
            CommandFailureClassification::WaitingApproval,
            events,
            "command execution is waiting for approval",
        )),
        ToolLoopStatus::Failed => Err(CommandTestLoopError::ToolFailed {
            diagnostics: vec![CommandTestDiagnostic::error(
                "command_test.tool_failed",
                "shell command tool did not complete",
                true,
            )],
            events,
        }),
    }
}

fn validate_command_input(input: &CommandTestLoopInput) -> Result<(), CommandTestLoopError> {
    if input.program.trim().is_empty() {
        return Err(CommandTestLoopError::InvalidWorkspace {
            diagnostics: vec![CommandTestDiagnostic::error(
                "command_test.program.empty",
                "program must be a non-empty string",
                false,
            )],
        });
    }
    if input.cwd.trim().is_empty() {
        return Err(CommandTestLoopError::InvalidWorkspace {
            diagnostics: vec![CommandTestDiagnostic::error(
                "command_test.cwd.empty",
                "cwd must be a non-empty workspace-relative path",
                false,
            )],
        });
    }
    Ok(())
}

fn report_from_shell_result(
    ids: RuntimeIds,
    command: CommandIntentSummary,
    classify_as_test: bool,
    command_run_id: String,
    shell: ShellCommandResult,
    events: Vec<RuntimeEvent>,
) -> CommandTestLoopReport {
    let stdout = bounded_output(&shell.stdout);
    let stderr = bounded_output(&shell.stderr);
    let (status, classification) = classify_shell_result(&shell, classify_as_test);
    let cancellation = if shell.timed_out {
        CommandCancellationState {
            cancellable: false,
            reason: "process was stopped after timeout".to_string(),
        }
    } else {
        CommandCancellationState {
            cancellable: true,
            reason: "command can be cancelled before completion".to_string(),
        }
    };
    let observation = command_observation(
        &command,
        status,
        classification,
        shell.status,
        shell.timed_out,
        &stdout,
        &stderr,
    );

    CommandTestLoopReport {
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        command_run_id,
        command,
        status,
        classification,
        exit_code: Some(shell.status),
        stdout,
        stderr,
        timed_out: shell.timed_out,
        cancellation,
        observation,
        events: sanitized_events(events),
        diagnostics: Vec::new(),
    }
}

fn policy_report(
    ids: RuntimeIds,
    command_run_id: String,
    command: CommandIntentSummary,
    status: CommandExecutionStatus,
    classification: CommandFailureClassification,
    events: Vec<RuntimeEvent>,
    message: &str,
) -> CommandTestLoopReport {
    let observation = bounded_observation(message.to_string(), false, false);
    CommandTestLoopReport {
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        command_run_id,
        command,
        status,
        classification,
        exit_code: None,
        stdout: bounded_output(""),
        stderr: bounded_output(""),
        timed_out: false,
        cancellation: CommandCancellationState {
            cancellable: false,
            reason: message.to_string(),
        },
        observation,
        events,
        diagnostics: vec![CommandTestDiagnostic::warning(
            "command_test.policy",
            message,
            true,
        )],
    }
}

fn classify_shell_result(
    shell: &ShellCommandResult,
    classify_as_test: bool,
) -> (CommandExecutionStatus, CommandFailureClassification) {
    if shell.timed_out {
        return (
            CommandExecutionStatus::TimedOut,
            CommandFailureClassification::Timeout,
        );
    }
    if shell.status == 0 {
        return (
            CommandExecutionStatus::Succeeded,
            CommandFailureClassification::None,
        );
    }
    (
        CommandExecutionStatus::Failed,
        if classify_as_test {
            CommandFailureClassification::TestFailure
        } else {
            CommandFailureClassification::CommandFailure
        },
    )
}

fn command_observation(
    command: &CommandIntentSummary,
    status: CommandExecutionStatus,
    classification: CommandFailureClassification,
    exit_code: i32,
    timed_out: bool,
    stdout: &BoundedCommandOutput,
    stderr: &BoundedCommandOutput,
) -> CommandObservationSummary {
    let message = format!(
        "command observation: command={} cwd={} status={} execution_status={} classification={} timed_out={} stdout={} stderr={}",
        command.display,
        command.cwd,
        exit_code,
        command_status_label(status),
        classification_label(classification),
        timed_out,
        stdout.text,
        stderr.text
    );
    bounded_observation(
        message,
        stdout.truncated || stderr.truncated,
        stdout.redacted || stderr.redacted,
    )
}

fn bounded_observation(
    message: String,
    inherited_truncated: bool,
    inherited_redacted: bool,
) -> CommandObservationSummary {
    let redacted = redact_secret_like_text(&message);
    let truncated = redacted.chars().count() > OBSERVATION_LIMIT;
    let bounded = if truncated {
        redacted.chars().take(OBSERVATION_LIMIT).collect()
    } else {
        redacted
    };
    CommandObservationSummary {
        message: bounded,
        truncated: truncated || inherited_truncated,
        redacted: inherited_redacted || preview_contains_redaction(&message),
    }
}

fn bounded_output(value: &str) -> BoundedCommandOutput {
    let byte_count = value.len().try_into().unwrap_or(u64::MAX);
    let redaction_needed = preview_contains_redaction(value);
    let safe = if redaction_needed {
        redact_secret_like_text(value)
    } else {
        value.to_string()
    };
    let truncated = safe.chars().count() > COMMAND_OUTPUT_LIMIT;
    let text = if truncated {
        safe.chars().take(COMMAND_OUTPUT_LIMIT).collect()
    } else {
        safe
    };
    BoundedCommandOutput {
        text,
        byte_count,
        truncated,
        redacted: redaction_needed,
    }
}

fn sanitized_events(events: Vec<RuntimeEvent>) -> Vec<RuntimeEvent> {
    events
        .into_iter()
        .map(|mut event| {
            event.payload = match event.payload {
                RuntimeEventPayload::ToolCallProposed(mut proposed) => {
                    proposed.input_json = sanitize_json_value(proposed.input_json);
                    RuntimeEventPayload::ToolCallProposed(proposed)
                }
                RuntimeEventPayload::ToolExecutionCompleted(mut completed) => {
                    completed.output_json = sanitize_json_value(completed.output_json);
                    RuntimeEventPayload::ToolExecutionCompleted(completed)
                }
                RuntimeEventPayload::ToolExecutionFailed(mut failed) => {
                    failed.message = redact_secret_like_text(&failed.message);
                    RuntimeEventPayload::ToolExecutionFailed(failed)
                }
                RuntimeEventPayload::ToolObservationRecorded(mut observed) => {
                    observed.observation = bound_text(
                        &redact_secret_like_text(&observed.observation),
                        OBSERVATION_LIMIT,
                    );
                    RuntimeEventPayload::ToolObservationRecorded(observed)
                }
                other => other,
            };
            event
        })
        .collect()
}

fn sanitize_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(value) => serde_json::Value::String(sanitize_json_string(&value)),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(sanitize_json_value).collect())
        }
        serde_json::Value::Object(values) => serde_json::Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, sanitize_json_value(value)))
                .collect(),
        ),
        other => other,
    }
}

fn sanitize_json_string(value: &str) -> String {
    let redacted = if preview_contains_redaction(value) {
        redact_secret_like_text(value)
    } else {
        value.to_string()
    };
    bound_text(&redacted, COMMAND_OUTPUT_LIMIT)
}

fn bound_text(value: &str, limit: usize) -> String {
    if value.chars().count() > limit {
        value.chars().take(limit).collect()
    } else {
        value.to_string()
    }
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

fn allow_policy<const N: usize>(tool_names: [&str; N]) -> StaticPolicyEngine {
    let mut policy = StaticPolicyEngine::default();
    for tool_name in tool_names {
        policy = policy.with_decision(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::Tool, tool_name).expect("static scope"),
            PolicyDecision::Allow {
                reason: "command test flow allows built-in workspace command tool".to_string(),
            },
        );
    }
    policy
}

fn tool_setup_error(message: String) -> CommandTestLoopError {
    CommandTestLoopError::ToolFailed {
        diagnostics: vec![CommandTestDiagnostic::error(
            "command_test.tool_setup.failed",
            message,
            false,
        )],
        events: Vec::new(),
    }
}

impl CommandTestDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: CommandTestDiagnosticSeverity::Warning,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: CommandTestDiagnosticSeverity::Error,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }
}

fn command_display(program: &str, args: &[String]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .map(redact_secret_like_text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn command_status_label(status: CommandExecutionStatus) -> &'static str {
    match status {
        CommandExecutionStatus::Succeeded => "succeeded",
        CommandExecutionStatus::Failed => "failed",
        CommandExecutionStatus::TimedOut => "timed_out",
        CommandExecutionStatus::WaitingApproval => "waiting_approval",
        CommandExecutionStatus::Denied => "denied",
        CommandExecutionStatus::ToolFailed => "tool_failed",
    }
}

fn classification_label(classification: CommandFailureClassification) -> &'static str {
    match classification {
        CommandFailureClassification::None => "none",
        CommandFailureClassification::TestFailure => "test_failure",
        CommandFailureClassification::CommandFailure => "command_failure",
        CommandFailureClassification::Timeout => "timeout",
        CommandFailureClassification::PolicyDenied => "policy_denied",
        CommandFailureClassification::WaitingApproval => "waiting_approval",
        CommandFailureClassification::ToolFailure => "tool_failure",
        CommandFailureClassification::InvalidInput => "invalid_input",
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
