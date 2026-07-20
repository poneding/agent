use ruse_cli::{
    CliCommand, ExtensionCommand, ExtensionCommandRuntime, OutputMode, render_extension_command,
};
use ruse_core::{
    ExtensionCapabilitySummary, ExtensionCompatibility, ExtensionHealth, ExtensionId,
    ExtensionKind, ExtensionRecord, ExtensionRegistry, ExtensionSource, ExtensionSourceKind,
};
use serde_json::Value;

fn plugin_record(id: &str, enabled: bool, health: ExtensionHealth) -> ExtensionRecord {
    ExtensionRecord::new(
        ExtensionId::new(id).unwrap(),
        ExtensionKind::Plugin,
        ExtensionSource::new(
            ExtensionSourceKind::LocalPath,
            format!("/tmp/{id}"),
            format!("{id} source"),
            Option::<String>::None,
        )
        .unwrap(),
        enabled,
        Some("0.1.0".to_string()),
        ExtensionCompatibility::supported(1),
        vec![ExtensionCapabilitySummary::new("tool", "hello.read", Vec::new()).unwrap()],
        Vec::new(),
        health,
    )
    .unwrap()
}

fn runtime() -> ExtensionCommandRuntime {
    let registry = ExtensionRegistry::new(vec![
        plugin_record("hello-plugin", true, ExtensionHealth::healthy("installed")),
        plugin_record(
            "broken-plugin",
            false,
            ExtensionHealth::unhealthy("token sk-secret failed"),
        ),
    ])
    .unwrap();
    ExtensionCommandRuntime::new(registry)
}

fn parse(args: impl IntoIterator<Item = &'static str>) -> ruse_cli::CliInvocation {
    ruse_cli::parse_cli_invocation(args).expect("parse invocation")
}

#[test]
fn extension_health_commands_parse_list_inspect_and_health() {
    assert_eq!(
        parse(["extension", "list"]).command,
        CliCommand::Extension(ExtensionCommand::List)
    );
    assert_eq!(
        parse(["extension", "health"]).command,
        CliCommand::Extension(ExtensionCommand::Health)
    );
    assert_eq!(
        parse(["extension", "inspect", "hello-plugin"]).command,
        CliCommand::Extension(ExtensionCommand::Inspect {
            id: "hello-plugin".to_string(),
        })
    );
    assert_eq!(
        parse(["--json", "extension", "list"]).output_mode,
        OutputMode::Json
    );
}

#[test]
fn extension_health_commands_render_text_list_and_inspect() {
    let mut runtime = runtime();

    let list = render_extension_command(&mut runtime, ExtensionCommand::List, OutputMode::Text);
    assert_eq!(list.status_code, 0);
    assert!(list.stdout.contains("hello-plugin"));
    assert!(list.stdout.contains("broken-plugin"));
    assert!(list.stdout.contains("unhealthy"));

    let inspect = render_extension_command(
        &mut runtime,
        ExtensionCommand::Inspect {
            id: "hello-plugin".to_string(),
        },
        OutputMode::Text,
    );
    assert!(inspect.stdout.contains("Extension hello-plugin"));
    assert!(inspect.stdout.contains("capabilities=1"));
}

#[test]
fn extension_health_commands_render_json_snapshot_and_redact_diagnostics() {
    let mut runtime = runtime();

    let output = render_extension_command(&mut runtime, ExtensionCommand::Health, OutputMode::Json);
    assert_eq!(output.status_code, 0);
    let value: Value = serde_json::from_str(output.stdout.trim()).unwrap();

    assert_eq!(value["type"], "extension_snapshot");
    assert_eq!(value["data"]["extensions"][0]["id"], "broken-plugin");
    assert_eq!(value["data"]["extensions"][1]["id"], "hello-plugin");
    assert!(output.stdout.contains("[redacted]"));
    assert!(!output.stdout.contains("sk-secret"));
    assert!(!output.stdout.contains("token"));
}

#[test]
fn extension_health_commands_report_missing_extension_safely() {
    let mut runtime = runtime();
    let output = render_extension_command(
        &mut runtime,
        ExtensionCommand::Inspect {
            id: "missing-plugin".to_string(),
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("extension_error"));
    assert!(output.stdout.contains("missing-plugin"));
}
