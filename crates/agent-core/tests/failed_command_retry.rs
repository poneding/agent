use agent_core::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandRetryPlanner, CommandTestLoopReport,
};
use agent_types::{
    CommandRetryFailureClassification, CommandRetryReadiness, CommandRetryRequest, ConversationId,
    WorkspaceId,
};

fn request(command_run_id: &str, reason: &str) -> CommandRetryRequest {
    CommandRetryRequest {
        command_run_id: command_run_id.to_string(),
        reason: reason.to_string(),
        dry_run: true,
    }
}

fn report(
    status: CommandExecutionStatus,
    classification: CommandFailureClassification,
    classify_as_test: bool,
) -> CommandTestLoopReport {
    CommandTestLoopReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        command: CommandIntentSummary {
            display: "cargo test --package agent-core".to_string(),
            cwd: ".".to_string(),
            classify_as_test,
        },
        status,
        classification,
        exit_code: Some(101),
        stdout: bounded("must-not-appear sk-secret-token"),
        stderr: bounded("api_key token password"),
        timed_out: classification == CommandFailureClassification::Timeout,
        cancellation: CommandCancellationState {
            cancellable: true,
            reason: "command can be cancelled".to_string(),
        },
        observation: CommandObservationSummary {
            message: "redacted observation".to_string(),
            truncated: false,
            redacted: true,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn bounded(text: &str) -> BoundedCommandOutput {
    BoundedCommandOutput {
        text: text.to_string(),
        byte_count: text.len() as u64,
        truncated: false,
        redacted: true,
    }
}

#[test]
fn retry_planner_marks_command_failure_retryable() {
    let planner = CommandRetryPlanner;
    let report = report(
        CommandExecutionStatus::Failed,
        CommandFailureClassification::CommandFailure,
        false,
    );

    let plan = planner.plan_from_report(
        request("command-run-1", "rerun after editing sk-secret-token"),
        &report,
    );
    let serialized = serde_json::to_string(&plan).expect("serialize plan");

    assert_eq!(plan.readiness, CommandRetryReadiness::Retryable);
    assert_eq!(
        plan.classification,
        CommandRetryFailureClassification::CommandFailure
    );
    assert_eq!(plan.attempt.previous_attempt_id, "command-run-1");
    assert_eq!(plan.attempt.next_attempt_id, "command-run-1-retry-1");
    assert_eq!(plan.reason, "rerun after editing [redacted]");
    assert!(!serialized.contains("must-not-appear"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn retry_planner_preserves_test_failure_intent() {
    let planner = CommandRetryPlanner;
    let report = report(
        CommandExecutionStatus::Failed,
        CommandFailureClassification::TestFailure,
        true,
    );

    let plan = planner.plan_from_report(request("test-run-1", "rerun tests"), &report);

    assert_eq!(plan.readiness, CommandRetryReadiness::Retryable);
    assert_eq!(
        plan.classification,
        CommandRetryFailureClassification::TestFailure
    );
    assert!(plan.command.classify_as_test);
}

#[test]
fn retry_planner_blocks_success_waiting_approval_and_policy_denied() {
    let planner = CommandRetryPlanner;

    let success = planner.plan_from_report(
        request("command-run-success", "rerun"),
        &report(
            CommandExecutionStatus::Succeeded,
            CommandFailureClassification::None,
            false,
        ),
    );
    assert_eq!(success.readiness, CommandRetryReadiness::AlreadySucceeded);

    let waiting = planner.plan_from_report(
        request("command-run-waiting", "rerun"),
        &report(
            CommandExecutionStatus::WaitingApproval,
            CommandFailureClassification::WaitingApproval,
            false,
        ),
    );
    assert_eq!(waiting.readiness, CommandRetryReadiness::WaitingApproval);

    let denied = planner.plan_from_report(
        request("command-run-denied", "rerun"),
        &report(
            CommandExecutionStatus::Denied,
            CommandFailureClassification::PolicyDenied,
            false,
        ),
    );
    assert_eq!(denied.readiness, CommandRetryReadiness::PolicyBlocked);
}

#[test]
fn retry_planner_reports_missing_evidence_and_empty_reason() {
    let planner = CommandRetryPlanner;

    let missing = planner.missing_evidence_plan(request("missing-run", "inspect target"));
    assert_eq!(missing.readiness, CommandRetryReadiness::MissingEvidence);
    assert!(
        missing
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "command_retry.evidence.missing")
    );

    let empty_reason = planner.plan_from_report(
        request("bad-run", " "),
        &report(
            CommandExecutionStatus::Failed,
            CommandFailureClassification::CommandFailure,
            false,
        ),
    );
    assert_eq!(empty_reason.readiness, CommandRetryReadiness::InvalidTarget);
    assert!(
        empty_reason
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "command_retry.reason.empty")
    );
}
