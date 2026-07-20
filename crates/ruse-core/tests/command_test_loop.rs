use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ruse_core::{
    CommandExecutionStatus, CommandFailureClassification, CommandRetryExecutionInput,
    CommandRetryPlanner, CommandTestLoopError, CommandTestLoopInput, RuntimeIds,
    run_command_test_loop,
};
use ruse_types::{
    CommandRetryReadiness, CommandRetryRequest, ConversationId, CorrelationId, RuntimeEventPayload,
    TurnId, WorkspaceId,
};
use tempfile::TempDir;

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn compile_helper(temp: &TempDir) -> PathBuf {
    let source = temp.path().join("command_helper.rs");
    fs::write(
        &source,
        r#"
use std::env;
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("stdout") => print!("hello stdout\n"),
        Some("stderr") => eprint!("hello stderr\n"),
        Some("exit7") => {
            print!("before exit\n");
            eprint!("failure stderr\n");
            process::exit(7);
        }
        Some("sleep") => thread::sleep(Duration::from_secs(5)),
        Some("long") => {
            for _ in 0..5000 {
                print!("x");
            }
        }
        Some("secret") => print!("token sk-test-secret api_key password\n"),
        _ => process::exit(64),
    }
}
"#,
    )
    .expect("write helper source");
    let binary = temp
        .path()
        .join(format!("command-helper{}", std::env::consts::EXE_SUFFIX));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let status = Command::new(rustc)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .status()
        .expect("compile helper");
    assert!(status.success(), "helper program should compile");
    binary
}

fn workspace() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir(temp.path().join("subdir")).expect("subdir");
    let helper = compile_helper(&temp);
    (temp, helper)
}

fn input(
    root: &Path,
    helper: &Path,
    args: &[&str],
    classify_as_test: bool,
) -> CommandTestLoopInput {
    CommandTestLoopInput {
        workspace_root: root.to_path_buf(),
        command_run_id: None,
        program: helper.display().to_string(),
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
        cwd: ".".to_string(),
        timeout_ms: 30_000,
        classify_as_test,
        ids: runtime_ids(),
    }
}

#[test]
fn command_test_loop_success_returns_bounded_stdout_and_events() {
    let (temp, helper) = workspace();

    let report =
        run_command_test_loop(input(temp.path(), &helper, &["stdout"], false)).expect("command");

    assert_eq!(report.status, CommandExecutionStatus::Succeeded);
    assert_eq!(report.classification, CommandFailureClassification::None);
    assert_eq!(report.stdout.text, "hello stdout\n");
    assert_eq!(report.stderr.text, "");
    assert!(!report.stdout.truncated);
    assert!(!report.stdout.redacted);
    assert!(report.cancellation.cancellable);
    assert!(report.observation.message.contains("status=0"));
    assert!(report.observation.message.contains("hello stdout"));
    assert!(
        report
            .command
            .display
            .contains(helper.file_name().unwrap().to_str().unwrap())
    );

    let phases: Vec<&'static str> = report
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(call)
                if call.tool_name == "workspace.shell_command" =>
            {
                Some("proposed")
            }
            RuntimeEventPayload::PermissionDecisionRecorded(_) => Some("decision"),
            RuntimeEventPayload::ToolExecutionStarted(started)
                if started.tool_name == "workspace.shell_command" =>
            {
                Some("started")
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed)
                if completed.call_id == "command-test-shell" =>
            {
                Some("completed")
            }
            RuntimeEventPayload::ToolObservationRecorded(observed)
                if observed.call_id == "command-test-shell" =>
            {
                Some("observed")
            }
            _ => None,
        })
        .collect();

    assert!(
        phases.windows(5).any(|window| {
            window == ["proposed", "decision", "started", "completed", "observed"]
        }),
        "phases were {phases:?}"
    );
}

#[test]
fn command_test_loop_classifies_non_zero_command_failure() {
    let (temp, helper) = workspace();

    let report =
        run_command_test_loop(input(temp.path(), &helper, &["exit7"], false)).expect("command");

    assert_eq!(report.status, CommandExecutionStatus::Failed);
    assert_eq!(
        report.classification,
        CommandFailureClassification::CommandFailure
    );
    assert_eq!(report.exit_code, Some(7));
    assert!(report.stdout.text.contains("before exit"));
    assert!(report.stderr.text.contains("failure stderr"));
}

#[test]
fn command_test_loop_classifies_non_zero_test_failure() {
    let (temp, helper) = workspace();

    let report =
        run_command_test_loop(input(temp.path(), &helper, &["exit7"], true)).expect("command");

    assert_eq!(report.status, CommandExecutionStatus::Failed);
    assert_eq!(
        report.classification,
        CommandFailureClassification::TestFailure
    );
    assert!(report.command.classify_as_test);
}

#[test]
fn command_retry_executor_reruns_from_typed_command_evidence() {
    let (temp, helper) = workspace();
    let report =
        run_command_test_loop(input(temp.path(), &helper, &["exit7"], true)).expect("command");

    let execution = CommandRetryPlanner
        .execute_from_report(CommandRetryExecutionInput {
            workspace_root: temp.path().to_path_buf(),
            request: CommandRetryRequest {
                command_run_id: report.command_run_id.clone(),
                reason: "rerun failed helper".to_string(),
                dry_run: false,
            },
            evidence: report,
        })
        .expect("retry execution");

    assert_eq!(execution.plan.readiness, CommandRetryReadiness::Retryable);
    assert!(!execution.plan.dry_run);
    let retry_report = execution.retry_report.expect("retry report");
    assert_eq!(
        retry_report.command_run_id,
        execution.plan.attempt.next_attempt_id
    );
    assert_eq!(retry_report.status, CommandExecutionStatus::Failed);
    assert_eq!(
        retry_report.classification,
        CommandFailureClassification::TestFailure
    );
    assert_eq!(
        retry_report.command.program.as_deref(),
        Some(helper.to_str().expect("utf8"))
    );
    assert_eq!(retry_report.command.args, vec!["exit7"]);
}

#[test]
fn command_test_loop_classifies_timeout_and_marks_stopped() {
    let (temp, helper) = workspace();
    let mut request = input(temp.path(), &helper, &["sleep"], true);
    request.timeout_ms = 10;

    let report = run_command_test_loop(request).expect("command");

    assert_eq!(report.status, CommandExecutionStatus::TimedOut);
    assert_eq!(report.classification, CommandFailureClassification::Timeout);
    assert_eq!(report.exit_code, Some(-1));
    assert!(report.timed_out);
    assert!(!report.cancellation.cancellable);
    assert!(report.cancellation.reason.contains("stopped"));
}

#[test]
fn command_test_loop_bounds_long_output() {
    let (temp, helper) = workspace();

    let report =
        run_command_test_loop(input(temp.path(), &helper, &["long"], false)).expect("command");

    assert_eq!(report.status, CommandExecutionStatus::Succeeded);
    assert!(report.stdout.byte_count >= 5000);
    assert!(report.stdout.truncated);
    assert!(report.stdout.text.len() < report.stdout.byte_count as usize);
    assert!(report.observation.truncated);
}

#[test]
fn command_test_loop_redacts_secret_like_output() {
    let (temp, helper) = workspace();

    let report =
        run_command_test_loop(input(temp.path(), &helper, &["secret"], false)).expect("command");

    let rendered = serde_json::to_string(&report).expect("serialize report");
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("password"));
    assert!(rendered.contains("[redacted]"));
    assert!(report.stdout.redacted);
    assert!(report.observation.redacted);
}

#[test]
fn command_test_loop_rejects_invalid_workspace_before_execution() {
    let (temp, helper) = workspace();
    let missing = temp.path().join("missing");

    let error =
        run_command_test_loop(input(&missing, &helper, &["stdout"], false)).expect_err("invalid");

    assert_eq!(error.code(), "command_test.workspace.invalid");
    assert!(matches!(
        error,
        CommandTestLoopError::InvalidWorkspace { .. }
    ));
    assert!(error.events().is_empty());
}

#[test]
fn command_test_loop_rejects_cwd_escape_before_command_output() {
    let (temp, helper) = workspace();
    let mut request = input(temp.path(), &helper, &["stdout"], false);
    request.cwd = "..".to_string();

    let error = run_command_test_loop(request).expect_err("cwd rejected");

    assert_eq!(error.code(), "command_test.tool_failed");
    assert!(
        error
            .events()
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionFailed(_)))
    );
}
