use ruse_cli::{
    CliCommand, OutputMode, TaskCliCommand, TaskCommandRuntime, parse_cli_command,
    parse_cli_invocation, render_cli_invocation, render_task_command, render_task_review_summary,
};
use ruse_scheduler::{TaskPriority, TaskSchedule};
use ruse_types::{CheckpointId, TaskId, TaskState};
use ruse_ui_bridge::{UiTaskReviewSummary, UiTaskReviewSummaryArtifact};

#[test]
fn task_commands_parse_task_subcommands() {
    assert_eq!(
        parse_cli_command(["task", "create", "--prompt", "review repo"]).expect("parse create"),
        CliCommand::Task(TaskCliCommand::Create {
            prompt: "review repo".to_string(),
            priority: TaskPriority::Normal,
            schedule: TaskSchedule::OneShot { run_at: None },
        })
    );
    assert_eq!(
        parse_cli_command(["task", "list"]).expect("parse list"),
        CliCommand::Task(TaskCliCommand::List)
    );
    assert_eq!(
        parse_cli_command(["task", "inspect", "00000000-0000-0000-0000-000000002401"])
            .expect("parse inspect"),
        CliCommand::Task(TaskCliCommand::Inspect {
            id: "00000000-0000-0000-0000-000000002401".to_string()
        })
    );
    assert_eq!(
        parse_cli_command([
            "task",
            "schedule",
            "00000000-0000-0000-0000-000000002401",
            "--cron",
            "0 8 * * *",
            "--reason",
            "morning run",
        ])
        .expect("parse schedule"),
        CliCommand::Task(TaskCliCommand::Schedule {
            id: "00000000-0000-0000-0000-000000002401".to_string(),
            schedule: TaskSchedule::Cron {
                expression: "0 8 * * *".to_string(),
            },
            reason: "morning run".to_string(),
        })
    );
}

#[test]
fn task_commands_render_text_lifecycle_against_fixture_runtime() {
    let mut runtime = TaskCommandRuntime::fixture();

    let create = render_task_command(
        &mut runtime,
        TaskCliCommand::Create {
            prompt: "review sk-secret-token".to_string(),
            priority: TaskPriority::Normal,
            schedule: TaskSchedule::OneShot { run_at: None },
        },
        OutputMode::Text,
    );
    assert_eq!(create.status_code, 0);
    assert!(create.stdout.contains("Created task"));
    assert!(create.stdout.contains("state=queued"));
    assert!(create.stdout.contains("[redacted]"));
    assert!(!create.stdout.contains("sk-secret-token"));

    let id = runtime
        .service()
        .list()
        .first()
        .expect("created task")
        .task_id
        .to_string();

    let list = render_task_command(&mut runtime, TaskCliCommand::List, OutputMode::Text);
    assert_eq!(list.status_code, 0);
    assert!(list.stdout.contains(&id));
    assert!(list.stdout.contains("queued"));

    let paused = render_task_command(
        &mut runtime,
        TaskCliCommand::Pause {
            id: id.clone(),
            reason: "hold for user".to_string(),
        },
        OutputMode::Text,
    );
    assert_eq!(paused.status_code, 0);
    assert!(paused.stdout.contains("Paused task"));
    assert!(paused.stdout.contains("notification=paused"));
    assert!(paused.stdout.contains("notification_summary=hold for user"));

    let resumed = render_task_command(
        &mut runtime,
        TaskCliCommand::Resume {
            id: id.clone(),
            reason: "continue".to_string(),
        },
        OutputMode::Text,
    );
    assert_eq!(resumed.status_code, 0);
    assert!(resumed.stdout.contains("Resumed task"));
    assert!(resumed.stdout.contains("notification=resumed"));

    let cancelled = render_task_command(
        &mut runtime,
        TaskCliCommand::Cancel {
            id,
            reason: "stop now".to_string(),
        },
        OutputMode::Text,
    );
    assert_eq!(cancelled.status_code, 0);
    assert!(cancelled.stdout.contains("Cancelled task"));
    assert!(cancelled.stdout.contains("notification=cancelled"));
}

#[test]
fn task_commands_render_empty_list_states() {
    let mut runtime = TaskCommandRuntime::fixture();

    let text = render_task_command(&mut runtime, TaskCliCommand::List, OutputMode::Text);
    assert_eq!(text.status_code, 0);
    assert_eq!(text.stdout, "No tasks\n");

    let json = render_task_command(&mut runtime, TaskCliCommand::List, OutputMode::Json);
    assert_eq!(json.status_code, 0);
    assert!(json.stdout.contains("\"type\":\"task_snapshot\""));
    assert!(json.stdout.contains("\"tasks\":[]"));
}

#[test]
fn task_commands_render_json_snapshot() {
    let invocation =
        parse_cli_invocation(["--json", "task", "create", "--prompt", "index workspace"])
            .expect("parse json create");
    assert_eq!(invocation.output_mode, OutputMode::Json);

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("\"type\":\"task_snapshot\""));
    assert!(output.stdout.contains("\"state\":\"queued\""));
    assert!(
        output
            .stdout
            .contains("\"latest_notification_kind\":\"queued\"")
    );
    assert!(output.stdout.contains("index workspace"));
}

#[test]
fn task_commands_report_missing_task_as_json_error() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "task",
            "inspect",
            "00000000-0000-0000-0000-000000002499",
        ])
        .expect("parse inspect"),
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("\"type\":\"task_snapshot\""));
    assert!(output.stdout.contains("\"code\":\"task_command_failed\""));
    assert!(output.stdout.contains("task not found"));
}

#[test]
fn task_commands_render_task_review_summary() {
    let task_id: TaskId = "00000000-0000-0000-0000-000000002405"
        .parse()
        .expect("task id");
    let checkpoint_id: CheckpointId = "00000000-0000-0000-0000-000000002415"
        .parse()
        .expect("checkpoint id");
    let review = UiTaskReviewSummary {
        task_id,
        final_state: TaskState::Succeeded,
        trace_id: Some("trace-123".to_string()),
        checkpoint_id: Some(checkpoint_id),
        artifact_ids: vec!["artifact-report".to_string()],
        artifact_count: 1,
        summary: "completed background task".to_string(),
        artifacts: vec![UiTaskReviewSummaryArtifact {
            artifact_id: "artifact-report".to_string(),
            kind: "report".to_string(),
            path: Some("artifacts/report.md".to_string()),
            content_hash: Some("sha256:abc123".to_string()),
            summary: "review report".to_string(),
            missing: false,
            diagnostics: Vec::new(),
        }],
        diagnostics: Vec::new(),
    };

    let output = render_task_review_summary(&review);

    assert!(output.contains("Task review 00000000-0000-0000-0000-000000002405"));
    assert!(output.contains("state=succeeded"));
    assert!(output.contains("trace=trace-123"));
    assert!(output.contains("checkpoint=00000000-0000-0000-0000-000000002415"));
    assert!(output.contains("artifact artifact-report kind=report missing=false"));
    assert!(output.contains("artifact_path artifacts/report.md"));
}
