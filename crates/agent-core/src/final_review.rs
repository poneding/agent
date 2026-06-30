use agent_types::{ConversationId, RuntimeEvent, RuntimeEventPayload, WorkspaceId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CommandExecutionStatus, CommandFailureClassification, CommandTestLoopReport,
    EditDiffFlowReport, LocalCodingError, replay_local_coding_outcome,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalReviewInput {
    pub edit: Option<EditDiffFlowReport>,
    pub command: Option<CommandTestLoopReport>,
    pub events: Vec<RuntimeEvent>,
    pub trace_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodingFinalReview {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub summary: String,
    pub changed_files: Vec<String>,
    pub command_results: Vec<FinalReviewCommandResult>,
    pub trace_id: Option<String>,
    pub trace_event_count: usize,
    pub audit_event_count: usize,
    pub replay_reconstructed: bool,
    pub unresolved_risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub diagnostics: Vec<FinalReviewDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinalReviewCommandResult {
    pub command: String,
    pub status: String,
    pub classification: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinalReviewDiagnostic {
    pub severity: FinalReviewDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalReviewDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum FinalReviewError {
    #[error("final review replay failed: {0}")]
    Replay(String),
}

pub fn build_final_review(input: FinalReviewInput) -> CodingFinalReview {
    if input.edit.is_none()
        && input.command.is_none()
        && !input.events.is_empty()
        && let Ok(review) = replay_final_review(&input.events, input.trace_id.clone())
    {
        return review;
    }

    let workspace_id = input
        .edit
        .as_ref()
        .map(|report| report.workspace_id)
        .or_else(|| input.command.as_ref().map(|report| report.workspace_id))
        .or_else(|| input.events.first().map(|event| event.workspace_id))
        .unwrap_or_default();
    let conversation_id = input
        .edit
        .as_ref()
        .map(|report| report.conversation_id)
        .or_else(|| input.command.as_ref().map(|report| report.conversation_id))
        .or_else(|| input.events.first().map(|event| event.conversation_id))
        .unwrap_or_default();

    let mut changed_files = input
        .edit
        .as_ref()
        .map(|report| sanitize_list(&report.changed_files))
        .unwrap_or_default();
    if changed_files.is_empty() {
        changed_files = changed_files_from_events(&input.events);
    }

    let mut command_results = input
        .command
        .as_ref()
        .map(|report| vec![command_result_from_report(report)])
        .unwrap_or_default();
    if command_results.is_empty() {
        command_results = command_results_from_events(&input.events);
    }

    let audit_event_count = count_audit_events(&input.events);
    let trace_event_count = count_trace_events(&input.events);
    let summary = final_summary_from_events(&input.events)
        .unwrap_or_else(|| render_summary(&changed_files, &command_results, audit_event_count));

    finalize_review(ReviewParts {
        workspace_id,
        conversation_id,
        summary: redact_secret_like_text(&summary),
        changed_files,
        command_results,
        trace_id: input.trace_id.map(|value| redact_secret_like_text(&value)),
        trace_event_count,
        audit_event_count,
        replay_reconstructed: !input.events.is_empty(),
        diagnostics: Vec::new(),
    })
}

pub fn replay_final_review(
    events: &[RuntimeEvent],
    trace_id: Option<String>,
) -> Result<CodingFinalReview, FinalReviewError> {
    let outcome = replay_local_coding_outcome(events).map_err(final_review_replay_error)?;
    let (workspace_id, conversation_id) = ids_from_events(events);
    let command_results = outcome
        .command_results
        .iter()
        .map(|result| command_result_from_local_summary(result))
        .collect::<Vec<_>>();

    Ok(finalize_review(ReviewParts {
        workspace_id,
        conversation_id,
        summary: redact_secret_like_text(&outcome.final_answer),
        changed_files: sanitize_list(&outcome.changed_files),
        command_results,
        trace_id: trace_id.map(|value| redact_secret_like_text(&value)),
        trace_event_count: count_trace_events(events),
        audit_event_count: outcome.audit_event_count,
        replay_reconstructed: !events.is_empty(),
        diagnostics: Vec::new(),
    }))
}

struct ReviewParts {
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    summary: String,
    changed_files: Vec<String>,
    command_results: Vec<FinalReviewCommandResult>,
    trace_id: Option<String>,
    trace_event_count: usize,
    audit_event_count: usize,
    replay_reconstructed: bool,
    diagnostics: Vec<FinalReviewDiagnostic>,
}

fn finalize_review(mut parts: ReviewParts) -> CodingFinalReview {
    let mut diagnostics = std::mem::take(&mut parts.diagnostics);
    let mut unresolved_risks = Vec::new();
    let mut next_actions = Vec::new();

    if parts.changed_files.is_empty() {
        diagnostics.push(FinalReviewDiagnostic::warning(
            "final_review.changed_files.missing",
            "No changed-file evidence was available.",
            true,
        ));
        unresolved_risks.push("No changed-file evidence was available.".to_string());
        next_actions.push("Inspect git diff before finishing.".to_string());
    }

    if parts.command_results.is_empty() {
        diagnostics.push(FinalReviewDiagnostic::warning(
            "final_review.commands.missing",
            "No command/test evidence was available.",
            true,
        ));
        unresolved_risks.push("No command/test evidence was available.".to_string());
        next_actions.push("Run a verification command before finishing.".to_string());
    }

    for result in &parts.command_results {
        if result.status != "succeeded" {
            unresolved_risks.push(format!(
                "Command/test evidence reported {} for {}.",
                result.classification, result.command
            ));
            next_actions.push("Fix failing verification and rerun tests.".to_string());
        }
    }

    if next_actions.is_empty() {
        next_actions.push("Review and commit the verified changes.".to_string());
    }

    CodingFinalReview {
        workspace_id: parts.workspace_id,
        conversation_id: parts.conversation_id,
        summary: parts.summary,
        changed_files: sanitize_list(&parts.changed_files),
        command_results: parts
            .command_results
            .into_iter()
            .map(sanitize_command_result)
            .collect(),
        trace_id: parts.trace_id,
        trace_event_count: parts.trace_event_count,
        audit_event_count: parts.audit_event_count,
        replay_reconstructed: parts.replay_reconstructed,
        unresolved_risks: sanitize_list(&unresolved_risks),
        next_actions: sanitize_list(&dedupe(next_actions)),
        diagnostics,
    }
}

fn command_result_from_report(report: &CommandTestLoopReport) -> FinalReviewCommandResult {
    let summary = if report.stderr.text.is_empty() {
        format!(
            "{} exited {}: {}",
            report.command.display,
            report.exit_code.unwrap_or_default(),
            report.stdout.text.trim()
        )
    } else {
        format!(
            "{} exited {}: {}",
            report.command.display,
            report.exit_code.unwrap_or_default(),
            report.stderr.text.trim()
        )
    };

    FinalReviewCommandResult {
        command: report.command.display.clone(),
        status: command_status_label(report.status).to_string(),
        classification: command_classification_label(report.classification).to_string(),
        exit_code: report.exit_code,
        timed_out: report.timed_out,
        summary,
    }
}

fn command_result_from_local_summary(summary: &str) -> FinalReviewCommandResult {
    let status = if summary.contains(" denied: ") {
        "denied"
    } else if summary.contains(" exited 0") {
        "succeeded"
    } else {
        "failed"
    };
    let exit_code = summary
        .split(" exited ")
        .nth(1)
        .and_then(|rest| rest.split([':', ' ']).next())
        .and_then(|value| value.parse::<i32>().ok());

    FinalReviewCommandResult {
        command: summary
            .split(" exited ")
            .next()
            .unwrap_or(summary)
            .split(" denied: ")
            .next()
            .unwrap_or(summary)
            .to_string(),
        status: status.to_string(),
        classification: if status == "succeeded" {
            "none".to_string()
        } else if status == "denied" {
            "policy_denied".to_string()
        } else {
            "command_failure".to_string()
        },
        exit_code,
        timed_out: false,
        summary: summary.to_string(),
    }
}

fn changed_files_from_events(events: &[RuntimeEvent]) -> Vec<String> {
    events
        .iter()
        .rev()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionCompleted(completed) => completed
                .output_json
                .get("files")
                .and_then(serde_json::Value::as_array)
                .map(|files| {
                    files
                        .iter()
                        .filter_map(|file| file.get("path").and_then(serde_json::Value::as_str))
                        .map(redact_secret_like_text)
                        .collect::<Vec<_>>()
                }),
            _ => None,
        })
        .unwrap_or_default()
}

fn command_results_from_events(events: &[RuntimeEvent]) -> Vec<FinalReviewCommandResult> {
    events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                let status = completed.output_json.get("status")?.as_i64()?;
                let stdout = completed
                    .output_json
                    .get("stdout")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .trim();
                let summary = if stdout.is_empty() {
                    format!("command exited {status}")
                } else {
                    format!("command exited {status}: {stdout}")
                };
                Some(command_result_from_local_summary(&summary))
            }
            _ => None,
        })
        .collect()
}

fn final_summary_from_events(events: &[RuntimeEvent]) -> Option<String> {
    events.iter().rev().find_map(|event| match &event.payload {
        RuntimeEventPayload::FinalResponseProduced(response) => Some(response.message.clone()),
        _ => None,
    })
}

fn render_summary(
    changed_files: &[String],
    command_results: &[FinalReviewCommandResult],
    audit_event_count: usize,
) -> String {
    let files = if changed_files.is_empty() {
        "none".to_string()
    } else {
        changed_files.join(", ")
    };
    let commands = if command_results.is_empty() {
        "none".to_string()
    } else {
        command_results
            .iter()
            .map(|result| result.summary.clone())
            .collect::<Vec<_>>()
            .join("; ")
    };
    format!(
        "Coding final review. Changed files: {files}. Commands: {commands}. Audit events: {audit_event_count}."
    )
}

fn count_audit_events(events: &[RuntimeEvent]) -> usize {
    events
        .iter()
        .filter(|event| {
            matches!(
                event.payload,
                RuntimeEventPayload::ToolCallProposed(_)
                    | RuntimeEventPayload::PermissionDecisionRecorded(_)
                    | RuntimeEventPayload::ToolExecutionStarted(_)
                    | RuntimeEventPayload::ToolExecutionCompleted(_)
                    | RuntimeEventPayload::ToolExecutionFailed(_)
                    | RuntimeEventPayload::ToolObservationRecorded(_)
            )
        })
        .count()
}

fn count_trace_events(events: &[RuntimeEvent]) -> usize {
    events
        .iter()
        .filter(|event| {
            matches!(
                event.payload,
                RuntimeEventPayload::ToolCallProposed(_)
                    | RuntimeEventPayload::ToolExecutionStarted(_)
                    | RuntimeEventPayload::ToolExecutionCompleted(_)
                    | RuntimeEventPayload::ToolExecutionFailed(_)
                    | RuntimeEventPayload::ToolObservationRecorded(_)
                    | RuntimeEventPayload::PermissionDecisionRecorded(_)
                    | RuntimeEventPayload::CheckpointRecorded(_)
                    | RuntimeEventPayload::FinalResponseProduced(_)
                    | RuntimeEventPayload::TurnCompleted(_)
            )
        })
        .count()
}

fn ids_from_events(events: &[RuntimeEvent]) -> (WorkspaceId, ConversationId) {
    events
        .first()
        .map(|event| (event.workspace_id, event.conversation_id))
        .unwrap_or_else(|| (WorkspaceId::new(), ConversationId::new()))
}

fn sanitize_command_result(mut result: FinalReviewCommandResult) -> FinalReviewCommandResult {
    result.command = redact_secret_like_text(&result.command);
    result.status = redact_secret_like_text(&result.status);
    result.classification = redact_secret_like_text(&result.classification);
    result.summary = redact_secret_like_text(&result.summary);
    result
}

fn sanitize_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| redact_secret_like_text(value))
        .collect()
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        if !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

impl FinalReviewDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: FinalReviewDiagnosticSeverity::Warning,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            severity: FinalReviewDiagnosticSeverity::Error,
            code: redact_secret_like_text(&code.into()),
            message: redact_secret_like_text(&message.into()),
            recoverable,
        }
    }
}

fn final_review_replay_error(error: LocalCodingError) -> FinalReviewError {
    FinalReviewError::Replay(error.to_string())
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

fn command_classification_label(classification: CommandFailureClassification) -> &'static str {
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

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
