use std::process::Command;

use agent_cli::{
    CliCommand, parse_cli_command, render_observability_json_bundle, render_observability_summary,
};
use agent_core::{ObservabilityEvidenceOptions, build_observability_evidence};
use agent_types::{
    ActorRef, ConversationId, CorrelationId, RuntimeEvent, RuntimeEventPayload, SchemaVersion,
    Timestamp, ToolCallProposed, TurnId, WorkspaceId,
};
use serde_json::{Value, json};

fn secret_event() -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload: RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-secret".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({
                "path": "README.md",
                "api_key": "secret-value",
                "nested": { "token": "another-secret" }
            }),
        }),
    }
}

#[test]
fn observability_commands_parse_flags() {
    assert_eq!(
        parse_cli_command(["--observability-summary"]).expect("parse summary"),
        CliCommand::ObservabilitySummary
    );
    assert_eq!(
        parse_cli_command(["--observability-json"]).expect("parse json"),
        CliCommand::ObservabilityJson
    );
}

#[test]
fn observability_commands_cli_prints_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .arg("--observability-summary")
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Observability evidence"));
    assert!(stdout.contains("Events:"));
    assert!(stdout.contains("Trace entries:"));
    assert!(stdout.contains("Audit records:"));
    assert!(stdout.contains("Replayable: yes"));
}

#[test]
fn observability_commands_cli_prints_json_bundle() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .arg("--observability-json")
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.ends_with('\n'));
    let value: Value = serde_json::from_str(stdout.trim()).expect("json evidence bundle");
    assert_eq!(value["schema_version"], 1);
    assert!(value["summary"]["total_events"].as_u64().expect("events") > 0);
    assert!(value["summary"]["trace_entries"].as_u64().expect("trace") > 0);
    assert!(
        value["audit"]["summary"]["audit_records"]
            .as_u64()
            .expect("audit")
            > 0
    );
}

#[test]
fn observability_renderers_do_not_expose_raw_secrets() {
    let event = secret_event();
    let bundle = build_observability_evidence(
        &[event],
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect("build evidence");

    let summary = render_observability_summary(&bundle);
    let json = render_observability_json_bundle(&bundle).expect("render json");

    assert!(!summary.contains("secret-value"));
    assert!(!summary.contains("another-secret"));
    assert!(!json.contains("secret-value"));
    assert!(!json.contains("another-secret"));
    assert!(json.contains("[redacted]"));
}
