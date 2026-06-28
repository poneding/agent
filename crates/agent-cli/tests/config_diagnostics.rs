use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_cli::{
    CliCommand, CliError, ModelSelection, OutputMode, parse_cli_invocation, render_cli_invocation,
};
use agent_core::ConfigOverride;
use agent_ui_bridge::UiJsonFrame;

fn valid_config_toml() -> String {
    r#"
schema_version = 1

[model]
default_provider = "openai"

[model.invocation]
timeout_ms = 30000
max_attempts = 2
retryable_failures = ["timeout", "transport", "rate_limited"]
fallback_provider_ids = ["fake"]

[[model.providers]]
id = "openai"
kind = "open_ai_compatible"
default_model = "gpt-4.1-mini"
enabled = true
base_url = "https://api.openai.com/v1"
timeout_ms = 30000
api_key_provider = "env"
api_key_name = "OPENAI_API_KEY"

[[model.providers]]
id = "fake"
kind = "fake"
default_model = "fake-scripted"
enabled = true
timeout_ms = 1000
"#
    .to_string()
}

fn write_config(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("agent-cli-{name}-{}.toml", std::process::id()));
    fs::write(&path, contents).expect("write config fixture");
    path
}

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn cli_config_diagnostics_parse_config_check() {
    let invocation = parse_cli_invocation([
        "--config-check",
        "--config",
        "agent.toml",
        "--provider",
        "openai",
        "--model",
        "gpt-4.1",
    ])
    .expect("parse config check");

    assert_eq!(invocation.command, CliCommand::ConfigCheck);
    assert_eq!(invocation.output_mode, OutputMode::Text);
    assert_eq!(invocation.config_path, Some(PathBuf::from("agent.toml")));
    assert_eq!(
        invocation.config_override,
        ConfigOverride {
            provider: Some("openai".to_string()),
            model: Some("gpt-4.1".to_string()),
        }
    );
    assert_eq!(
        invocation.model_selection,
        ModelSelection::new("openai", "gpt-4.1").expect("valid model selection")
    );

    let missing_config =
        parse_cli_invocation(["--config"]).expect_err("missing config value rejected");
    assert_eq!(
        missing_config,
        CliError::MissingValue("--config".to_string())
    );
}

#[test]
fn cli_config_diagnostics_text_reports_valid_config() {
    let path = write_config("valid-text", &valid_config_toml());
    let invocation = parse_cli_invocation(["--config-check", "--config", path.to_str().unwrap()])
        .expect("parse config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.exit);
    assert!(output.stdout.contains("Config: valid"));
    assert!(output.stdout.contains("Selected provider: openai"));
    assert!(output.stdout.contains("Selected model: gpt-4.1-mini"));
    assert!(output.stdout.contains("Providers: fake, openai"));
    assert!(output.stdout.contains("Settings profile: default"));
    assert!(output.stdout.contains("Release channel: dev"));
    assert!(output.stdout.contains(
        "Secret store: os_keychain available=false writable=false fallback=env_fallback"
    ));
    assert!(output.stdout.contains("Config dir: "));
}

#[test]
fn cli_config_diagnostics_json_renders_config_summary() {
    let path = write_config("valid-json", &valid_config_toml());
    let invocation = parse_cli_invocation([
        "--json",
        "--config-check",
        "--config",
        path.to_str().unwrap(),
        "--provider",
        "fake",
        "--model",
        "fake-custom",
    ])
    .expect("parse json config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ConfigDiagnostic { code, .. })
            if code == "config.layer.provider_overridden"
    ));
    let snapshot = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSnapshot(snapshot) = frame {
            Some(snapshot)
        } else {
            None
        }
    });
    let snapshot = snapshot.expect("config snapshot frame");
    assert_eq!(snapshot.selected_provider, "fake");
    assert_eq!(snapshot.selected_model, "fake-custom");
    assert_eq!(
        snapshot
            .settings
            .as_ref()
            .expect("settings summary")
            .release_channel,
        "dev"
    );
    assert!(
        !snapshot
            .platform_paths
            .as_ref()
            .expect("platform paths")
            .config_dir
            .is_empty()
    );
    let summary = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSummary {
            selected_provider,
            selected_model,
            provider_ids,
            layer_count,
            diagnostic_count,
        } = frame
        {
            Some((
                selected_provider,
                selected_model,
                provider_ids,
                layer_count,
                diagnostic_count,
            ))
        } else {
            None
        }
    });
    assert_eq!(
        summary,
        Some((
            &"fake".to_string(),
            &"fake-custom".to_string(),
            &vec!["fake".to_string(), "openai".to_string()],
            &2,
            &1,
        ))
    );
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: 0 })
    ));
}

#[test]
fn cli_config_diagnostics_invalid_toml_exits_non_zero() {
    let path = write_config("invalid-toml", "schema_version = 1\n[model\n");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--config-check", "--config", path.to_str().unwrap()])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("config check failed") || stderr.contains("config check failed"));
}

#[test]
fn cli_config_diagnostics_redacts_secret_like_values() {
    let config = valid_config_toml().replace(
        "api_key_name = \"OPENAI_API_KEY\"",
        "api_key_name = \"OPENAI_API_KEY\"\napi_key_value = \"sk-test-secret\"",
    );
    let path = write_config("raw-secret", &config);

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "--json",
            "--config-check",
            "--config",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("sk-test-secret"));
    assert!(!stderr.contains("sk-test-secret"));
    assert!(!stdout.contains("secret_value"));
    assert!(!stderr.contains("secret_value"));
}
