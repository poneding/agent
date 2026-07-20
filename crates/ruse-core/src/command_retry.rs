use std::path::PathBuf;

use ruse_types::{
    CommandRetryAttemptLink, CommandRetryCommandSummary, CommandRetryDiagnostic,
    CommandRetryDiagnosticSeverity, CommandRetryFailureClassification, CommandRetryPlan,
    CommandRetryReadiness, CommandRetryRequest,
};
use ruse_types::{ConversationId, CorrelationId, TurnId, WorkspaceId};

use crate::{
    CommandExecutionStatus, CommandFailureClassification, CommandIntentSummary,
    CommandTestLoopError, CommandTestLoopInput, CommandTestLoopReport,
    DEFAULT_COMMAND_TEST_TIMEOUT_MS, RuntimeIds, run_command_test_loop,
};

const MAX_RETRY_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Default)]
pub struct CommandRetryPlanner;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRetryExecutionInput {
    pub workspace_root: PathBuf,
    pub request: CommandRetryRequest,
    pub evidence: CommandTestLoopReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRetryExecutionReport {
    pub plan: CommandRetryPlan,
    pub retry_report: Option<CommandTestLoopReport>,
}

impl CommandRetryPlanner {
    #[must_use]
    pub fn plan_from_report(
        &self,
        request: CommandRetryRequest,
        report: &CommandTestLoopReport,
    ) -> CommandRetryPlan {
        let mut diagnostics = Vec::new();
        if request.reason.trim().is_empty() {
            diagnostics.push(diagnostic(
                CommandRetryDiagnosticSeverity::Error,
                "command_retry.reason.empty",
                "retry reason cannot be empty",
                false,
            ));
        }

        let readiness = readiness_for_report(report, &diagnostics);
        let command_run_id = bound_text(request.command_run_id);
        CommandRetryPlan {
            attempt: CommandRetryAttemptLink {
                previous_attempt_id: command_run_id.clone(),
                next_attempt_id: next_attempt_id(&command_run_id),
            },
            command_run_id,
            readiness,
            classification: retry_classification(report.classification),
            command: retry_command_summary(&report.command),
            exit_code: report.exit_code,
            reason: bound_text(request.reason),
            dry_run: request.dry_run,
            diagnostics,
        }
    }

    #[must_use]
    pub fn missing_evidence_plan(&self, request: CommandRetryRequest) -> CommandRetryPlan {
        let command_run_id = bound_text(request.command_run_id);
        CommandRetryPlan {
            attempt: CommandRetryAttemptLink {
                previous_attempt_id: command_run_id.clone(),
                next_attempt_id: next_attempt_id(&command_run_id),
            },
            command_run_id,
            readiness: CommandRetryReadiness::MissingEvidence,
            classification: CommandRetryFailureClassification::InvalidInput,
            command: CommandRetryCommandSummary {
                display: "unknown".to_string(),
                program: None,
                args: Vec::new(),
                timeout_ms: DEFAULT_COMMAND_TEST_TIMEOUT_MS,
                cwd: ".".to_string(),
                classify_as_test: false,
            },
            exit_code: None,
            reason: bound_text(request.reason),
            dry_run: request.dry_run,
            diagnostics: vec![diagnostic(
                CommandRetryDiagnosticSeverity::Error,
                "command_retry.evidence.missing",
                "command retry target was not found in available command evidence",
                false,
            )],
        }
    }

    pub fn execute_from_report(
        &self,
        input: CommandRetryExecutionInput,
    ) -> Result<CommandRetryExecutionReport, CommandTestLoopError> {
        let mut plan = self.plan_from_report(input.request, &input.evidence);
        if plan.readiness != CommandRetryReadiness::Retryable {
            return Ok(CommandRetryExecutionReport {
                plan,
                retry_report: None,
            });
        }

        let Some(program) = input.evidence.command.program.clone() else {
            plan.readiness = CommandRetryReadiness::InvalidTarget;
            plan.diagnostics.push(diagnostic(
                CommandRetryDiagnosticSeverity::Error,
                "command_retry.argv.missing",
                "command retry execution requires typed program evidence",
                false,
            ));
            return Ok(CommandRetryExecutionReport {
                plan,
                retry_report: None,
            });
        };

        let retry_report = run_command_test_loop(CommandTestLoopInput {
            workspace_root: input.workspace_root,
            command_run_id: Some(plan.attempt.next_attempt_id.clone()),
            program,
            args: input.evidence.command.args.clone(),
            cwd: input.evidence.command.cwd.clone(),
            timeout_ms: input.evidence.command.timeout_ms,
            classify_as_test: input.evidence.command.classify_as_test,
            ids: retry_runtime_ids(input.evidence.workspace_id, input.evidence.conversation_id),
        })?;

        Ok(CommandRetryExecutionReport {
            plan,
            retry_report: Some(retry_report),
        })
    }
}

fn readiness_for_report(
    report: &CommandTestLoopReport,
    diagnostics: &[CommandRetryDiagnostic],
) -> CommandRetryReadiness {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CommandRetryDiagnosticSeverity::Error)
    {
        return CommandRetryReadiness::InvalidTarget;
    }

    match report.status {
        CommandExecutionStatus::Succeeded => CommandRetryReadiness::AlreadySucceeded,
        CommandExecutionStatus::WaitingApproval => CommandRetryReadiness::WaitingApproval,
        CommandExecutionStatus::Denied => CommandRetryReadiness::PolicyBlocked,
        CommandExecutionStatus::Failed
        | CommandExecutionStatus::TimedOut
        | CommandExecutionStatus::ToolFailed => match report.classification {
            CommandFailureClassification::CommandFailure
            | CommandFailureClassification::TestFailure
            | CommandFailureClassification::Timeout
            | CommandFailureClassification::ToolFailure => CommandRetryReadiness::Retryable,
            CommandFailureClassification::InvalidInput => CommandRetryReadiness::InvalidTarget,
            CommandFailureClassification::PolicyDenied => CommandRetryReadiness::PolicyBlocked,
            CommandFailureClassification::WaitingApproval => CommandRetryReadiness::WaitingApproval,
            CommandFailureClassification::None => CommandRetryReadiness::InvalidTarget,
        },
    }
}

fn retry_command_summary(command: &CommandIntentSummary) -> CommandRetryCommandSummary {
    CommandRetryCommandSummary {
        display: bound_text(command.display.clone()),
        program: command.program.clone().map(bound_text),
        args: command.args.iter().cloned().map(bound_text).collect(),
        timeout_ms: command.timeout_ms,
        cwd: bound_text(command.cwd.clone()),
        classify_as_test: command.classify_as_test,
    }
}

fn retry_runtime_ids(workspace_id: WorkspaceId, conversation_id: ConversationId) -> RuntimeIds {
    RuntimeIds {
        workspace_id,
        conversation_id,
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn retry_classification(
    classification: CommandFailureClassification,
) -> CommandRetryFailureClassification {
    match classification {
        CommandFailureClassification::None => CommandRetryFailureClassification::None,
        CommandFailureClassification::TestFailure => CommandRetryFailureClassification::TestFailure,
        CommandFailureClassification::CommandFailure => {
            CommandRetryFailureClassification::CommandFailure
        }
        CommandFailureClassification::Timeout => CommandRetryFailureClassification::Timeout,
        CommandFailureClassification::PolicyDenied => {
            CommandRetryFailureClassification::PolicyDenied
        }
        CommandFailureClassification::WaitingApproval => {
            CommandRetryFailureClassification::WaitingApproval
        }
        CommandFailureClassification::ToolFailure => CommandRetryFailureClassification::ToolFailure,
        CommandFailureClassification::InvalidInput => {
            CommandRetryFailureClassification::InvalidInput
        }
    }
}

fn diagnostic(
    severity: CommandRetryDiagnosticSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
    recoverable: bool,
) -> CommandRetryDiagnostic {
    let (message, truncated) = bound_text_with_flag(message);
    CommandRetryDiagnostic {
        severity,
        code: bound_text(code),
        message,
        recoverable,
        truncated,
    }
}

fn next_attempt_id(command_run_id: &str) -> String {
    let base = command_run_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    bound_text(format!("{base}-retry-1"))
}

fn bound_text(value: impl Into<String>) -> String {
    bound_text_with_flag(value).0
}

fn bound_text_with_flag(value: impl Into<String>) -> (String, bool) {
    let value = redact_secret_like(&value.into());
    if value.chars().count() <= MAX_RETRY_TEXT_CHARS {
        return (value, false);
    }
    let mut output = value.chars().take(MAX_RETRY_TEXT_CHARS).collect::<String>();
    output.push_str("...");
    (output, true)
}

fn redact_secret_like(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if lower.contains("secret")
                || lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("password")
                || lower.starts_with("sk-")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
