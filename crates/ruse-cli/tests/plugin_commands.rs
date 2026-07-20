use std::path::{Path, PathBuf};

use ruse_cli::{
    CliCommand, CliError, ModelSelection, OutputMode, PluginCommand, PluginCommandRuntime,
    render_plugin_command,
};
use ruse_core::{ConfigOverride, ExtensionCompatibility, ExtensionHealth, ExtensionId};
use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ruse-plugin/tests/fixtures")
        .join(name)
}

fn parse(args: impl IntoIterator<Item = &'static str>) -> ruse_cli::CliInvocation {
    ruse_cli::parse_cli_invocation(args).expect("parse invocation")
}

#[test]
fn plugin_commands_parse_install_and_management_subcommands() {
    let install = parse(["plugin", "install", "fixtures/hello", "--enable"]);
    assert_eq!(
        install.command,
        CliCommand::Plugin(PluginCommand::Install {
            path: PathBuf::from("fixtures/hello"),
            enable: true,
            accept_capabilities: false,
        })
    );
    assert_eq!(install.output_mode, OutputMode::Text);

    let enable = parse(["plugin", "enable", "hello-plugin", "--accept-capabilities"]);
    assert_eq!(
        enable.command,
        CliCommand::Plugin(PluginCommand::Enable {
            id: "hello-plugin".to_string(),
            accept_capabilities: true,
        })
    );

    assert_eq!(
        parse(["plugin", "list"]).command,
        CliCommand::Plugin(PluginCommand::List)
    );
    assert_eq!(
        parse(["plugin", "inspect", "hello-plugin"]).command,
        CliCommand::Plugin(PluginCommand::Inspect {
            id: "hello-plugin".to_string(),
        })
    );
    assert_eq!(
        parse(["plugin", "disable", "hello-plugin"]).command,
        CliCommand::Plugin(PluginCommand::Disable {
            id: "hello-plugin".to_string(),
        })
    );
    assert_eq!(
        parse(["plugin", "remove", "hello-plugin"]).command,
        CliCommand::Plugin(PluginCommand::Remove {
            id: "hello-plugin".to_string(),
        })
    );
}

#[test]
fn plugin_commands_render_install_report_without_enabling_by_default() {
    let mut runtime = PluginCommandRuntime::default();
    let output = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Installed plugin hello-plugin"));
    assert!(output.stdout.contains("enabled=false"));
    assert!(output.stdout.contains("capability_review_required=true"));
    assert_eq!(runtime.registry().snapshot().records.len(), 1);
    assert!(!runtime.registry().snapshot().records[0].enabled);
}

#[test]
fn plugin_commands_require_capability_acceptance_before_enablement() {
    let mut runtime = PluginCommandRuntime::default();
    let output = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: true,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("capability review is required"));
    assert!(runtime.registry().snapshot().records.is_empty());
}

#[test]
fn plugin_commands_install_enable_list_inspect_disable_and_remove() {
    let mut runtime = PluginCommandRuntime::default();
    let install = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: true,
            accept_capabilities: true,
        },
        OutputMode::Json,
    );
    assert_eq!(install.status_code, 0);
    let value: Value = serde_json::from_str(install.stdout.trim()).expect("json report");
    assert_eq!(value["type"], "plugin_install");
    assert_eq!(value["data"]["extension_id"], "hello-plugin");
    assert_eq!(value["data"]["enabled"], true);
    assert_eq!(value["data"]["diagnostics"][0]["code"], "plugin.enabled");
    assert_eq!(
        runtime.registry().snapshot().records[0]
            .granted_leases
            .len(),
        1
    );

    let list = render_plugin_command(&mut runtime, PluginCommand::List, OutputMode::Text);
    assert!(list.stdout.contains("hello-plugin"));
    assert!(list.stdout.contains("enabled=true"));

    let inspect = render_plugin_command(
        &mut runtime,
        PluginCommand::Inspect {
            id: "hello-plugin".to_string(),
        },
        OutputMode::Json,
    );
    let inspected: Value = serde_json::from_str(inspect.stdout.trim()).expect("inspect json");
    assert_eq!(inspected["data"]["id"], "hello-plugin");
    assert_eq!(
        inspected["data"]["declared_capabilities"][0]["name"],
        "hello.read"
    );

    let disabled = render_plugin_command(
        &mut runtime,
        PluginCommand::Disable {
            id: "hello-plugin".to_string(),
        },
        OutputMode::Text,
    );
    assert!(disabled.stdout.contains("Disabled plugin hello-plugin"));
    assert!(!runtime.registry().snapshot().records[0].enabled);
    assert!(
        runtime.registry().snapshot().records[0]
            .granted_leases
            .is_empty()
    );

    let enabled = render_plugin_command(
        &mut runtime,
        PluginCommand::Enable {
            id: "hello-plugin".to_string(),
            accept_capabilities: true,
        },
        OutputMode::Json,
    );
    assert_eq!(enabled.status_code, 0);
    let enabled_value: Value = serde_json::from_str(enabled.stdout.trim()).unwrap();
    assert_eq!(enabled_value["type"], "plugin_enabled");
    assert_eq!(
        enabled_value["data"]["granted_leases"][0]["capability_name"],
        "hello.read"
    );

    let removed = render_plugin_command(
        &mut runtime,
        PluginCommand::Remove {
            id: "hello-plugin".to_string(),
        },
        OutputMode::Text,
    );
    assert!(removed.stdout.contains("Removed plugin hello-plugin"));
    assert!(runtime.registry().snapshot().records.is_empty());
}

#[test]
fn plugin_commands_reject_invalid_manifest_and_duplicate_install() {
    let mut runtime = PluginCommandRuntime::default();
    let invalid = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("invalid-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );
    assert_eq!(invalid.status_code, 2);
    assert!(invalid.stdout.contains("plugin install failed"));
    assert!(runtime.registry().snapshot().records.is_empty());

    let first = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );
    assert_eq!(first.status_code, 0);

    let duplicate = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );
    assert_eq!(duplicate.status_code, 2);
    assert!(duplicate.stdout.contains("already installed"));
}

#[test]
fn plugin_commands_render_stable_install_diagnostics() {
    let mut runtime = PluginCommandRuntime::default();
    let output = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 0);
    let value: Value = serde_json::from_str(output.stdout.trim()).unwrap();
    assert_eq!(value["type"], "plugin_install");
    assert_eq!(
        value["data"]["diagnostics"][0]["code"],
        "plugin.capability_review_required"
    );
    assert_eq!(
        value["data"]["diagnostics"][1]["code"],
        "plugin.installed_disabled"
    );
}

#[test]
fn plugin_commands_reject_enable_for_incompatible_plugin() {
    let mut runtime = PluginCommandRuntime::default();
    let install = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: fixture_path("hello-process-plugin"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Text,
    );
    assert_eq!(install.status_code, 0);
    let mut record = runtime
        .registry()
        .get(&ExtensionId::new("hello-plugin").unwrap())
        .cloned()
        .unwrap();
    record.compatibility =
        ExtensionCompatibility::unsupported(99, 1, "token sk-secret unsupported");
    record.health = ExtensionHealth::unhealthy(record.compatibility.summary());
    let mut registry = runtime.registry().clone();
    registry.upsert(record).unwrap();
    let mut runtime = PluginCommandRuntime::new(registry);

    let output = render_plugin_command(
        &mut runtime,
        PluginCommand::Enable {
            id: "hello-plugin".to_string(),
            accept_capabilities: true,
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("plugin_error"));
    assert!(output.stdout.contains("cannot be enabled"));
    assert!(output.stdout.contains("[redacted]"));
    assert!(!output.stdout.contains("sk-secret"));
    assert!(!output.stdout.contains("token"));
}

#[test]
fn plugin_commands_redact_secret_like_errors() {
    let mut runtime = PluginCommandRuntime::default();
    let output = render_plugin_command(
        &mut runtime,
        PluginCommand::Install {
            path: PathBuf::from("/tmp/sk-secret-token/plugin.json"),
            enable: false,
            accept_capabilities: false,
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(!output.stdout.contains("sk-secret-token"));
    assert!(!output.stdout.contains("token"));
    assert!(output.stdout.contains("[redacted]"));
}

#[test]
fn plugin_commands_invocation_shape_remains_script_safe() {
    let invocation = parse(["--json", "plugin", "list"]);

    assert_eq!(invocation.command, CliCommand::Plugin(PluginCommand::List));
    assert_eq!(invocation.output_mode, OutputMode::Json);
    assert_eq!(invocation.model_selection, ModelSelection::default());
    assert_eq!(invocation.config_override, ConfigOverride::default());
}

#[test]
fn plugin_commands_reject_missing_values() {
    let error = ruse_cli::parse_cli_invocation(["plugin", "inspect"]).expect_err("missing id");

    assert_eq!(error, CliError::MissingValue("plugin inspect".to_string()));
}
