use agent_cli::{
    CliCommand, OutputMode, TaskCliCommand, TaskCommandRuntime, parse_cli_command,
    parse_cli_invocation, render_cli_invocation, render_task_command,
};
use agent_scheduler::{TaskPriority, TaskSchedule};

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
