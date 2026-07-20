use agent_core::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandTestLoopReport, DiffEvidenceState, DiffSummary, EditDiffFlowReport, EditPreviewSummary,
    FinalReviewInput, build_final_review, replay_final_review, run_local_coding_fixture,
};
use agent_types::{ConversationId, WorkspaceId};

fn edit_report() -> EditDiffFlowReport {
    EditDiffFlowReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        path: "README.md".to_string(),
        applied: true,
        preview: EditPreviewSummary {
            resource: "README.md".to_string(),
            summary: "Edit README.md: +1 -1 lines".to_string(),
            content_hash: Some("fnv1a64:edit".to_string()),
            truncated: false,
            redacted: false,
        },
        changed_files: vec!["README.md".to_string()],
        diff: DiffSummary {
            state: DiffEvidenceState::Applied,
            changed_file_count: 1,
            diff_line_count: 5,
            added_line_count: 1,
            removed_line_count: 1,
            preview: "-old\n+new\n".to_string(),
            truncated: false,
            redacted: false,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn command_report(status: CommandExecutionStatus) -> CommandTestLoopReport {
    let classification = match status {
        CommandExecutionStatus::Succeeded => CommandFailureClassification::None,
        CommandExecutionStatus::Failed => CommandFailureClassification::TestFailure,
        CommandExecutionStatus::TimedOut => CommandFailureClassification::Timeout,
        CommandExecutionStatus::WaitingApproval => CommandFailureClassification::WaitingApproval,
        CommandExecutionStatus::Denied => CommandFailureClassification::PolicyDenied,
        CommandExecutionStatus::ToolFailed => CommandFailureClassification::ToolFailure,
    };
    CommandTestLoopReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        command_run_id: "final-review-command-run".to_string(),
        command: CommandIntentSummary {
            display: "cargo test -p agent-core final_review".to_string(),
            program: Some("cargo".to_string()),
            args: vec![
                "test".to_string(),
                "-p".to_string(),
                "agent-core".to_string(),
                "final_review".to_string(),
            ],
            timeout_ms: 30_000,
            cwd: ".".to_string(),
            classify_as_test: true,
        },
        status,
        classification,
        exit_code: Some(if status == CommandExecutionStatus::Succeeded {
            0
        } else {
            101
        }),
        stdout: BoundedCommandOutput {
            text: "running tests\n".to_string(),
            byte_count: 14,
            truncated: false,
            redacted: false,
        },
        stderr: BoundedCommandOutput {
            text: if status == CommandExecutionStatus::Succeeded {
                String::new()
            } else {
                "test failed\n".to_string()
            },
            byte_count: if status == CommandExecutionStatus::Succeeded {
                0
            } else {
                12
            },
            truncated: false,
            redacted: false,
        },
        timed_out: status == CommandExecutionStatus::TimedOut,
        cancellation: CommandCancellationState {
            cancellable: status != CommandExecutionStatus::TimedOut,
            reason: "command can be cancelled before completion".to_string(),
        },
        observation: CommandObservationSummary {
            message: "command observation: status=0".to_string(),
            truncated: false,
            redacted: false,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn final_review_composes_edit_and_command_evidence() {
    let review = build_final_review(FinalReviewInput {
        edit: Some(edit_report()),
        command: Some(command_report(CommandExecutionStatus::Succeeded)),
        events: Vec::new(),
        trace_id: Some("trace-123".to_string()),
    });

    assert_eq!(review.changed_files, vec!["README.md"]);
    assert_eq!(review.command_results.len(), 1);
    assert_eq!(review.command_results[0].status, "succeeded");
    assert_eq!(review.command_results[0].classification, "none");
    assert_eq!(review.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(review.audit_event_count, 0);
    assert!(review.unresolved_risks.is_empty());
    assert!(
        review
            .next_actions
            .contains(&"Review and commit the verified changes.".to_string())
    );
}

#[test]
fn final_review_reconstructs_from_local_coding_events() {
    let run = run_local_coding_fixture().expect("local coding fixture");

    let review = replay_final_review(&run.events, Some("trace-local".to_string()))
        .expect("replay final review");

    assert_eq!(review.changed_files, vec!["README.md"]);
    assert_eq!(review.command_results.len(), 1);
    assert!(review.command_results[0].summary.contains("M README.md"));
    assert_eq!(review.audit_event_count, 30);
    assert!(review.trace_event_count > 0);
    assert!(review.replay_reconstructed);
    assert!(review.summary.contains("Local coding workflow completed"));
}

#[test]
fn final_review_reports_missing_evidence_as_risks() {
    let review = build_final_review(FinalReviewInput {
        edit: None,
        command: None,
        events: Vec::new(),
        trace_id: None,
    });

    assert!(
        review
            .unresolved_risks
            .contains(&"No changed-file evidence was available.".to_string())
    );
    assert!(
        review
            .unresolved_risks
            .contains(&"No command/test evidence was available.".to_string())
    );
    assert!(
        review
            .next_actions
            .contains(&"Run a verification command before finishing.".to_string())
    );
    assert!(review.diagnostics.len() >= 2);
}

#[test]
fn final_review_failed_command_becomes_risk_and_next_action() {
    let review = build_final_review(FinalReviewInput {
        edit: Some(edit_report()),
        command: Some(command_report(CommandExecutionStatus::Failed)),
        events: Vec::new(),
        trace_id: None,
    });

    assert!(
        review
            .unresolved_risks
            .iter()
            .any(|risk| risk.contains("test_failure"))
    );
    assert!(
        review
            .next_actions
            .iter()
            .any(|action| action.contains("Fix failing verification"))
    );
}

#[test]
fn final_review_redacts_secret_like_content() {
    let mut edit = edit_report();
    edit.changed_files = vec!["api_key.md".to_string()];
    let mut command = command_report(CommandExecutionStatus::Succeeded);
    command.command.display = "echo sk-test-secret password".to_string();
    command.stdout.text = "token sk-test-secret api_key password".to_string();
    command.stdout.redacted = true;

    let review = build_final_review(FinalReviewInput {
        edit: Some(edit),
        command: Some(command),
        events: Vec::new(),
        trace_id: Some("trace-secret-token".to_string()),
    });
    let serialized = serde_json::to_string(&review).expect("serialize review");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("trace-secret-token"));
    assert!(serialized.contains("[redacted]"));
}
