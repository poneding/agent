use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_cli::{
    CliCommand, CliError, ModelSelection, OutputMode, parse_cli_invocation, render_cli_invocation,
};
use agent_core::ConfigOverride;
use agent_ui_bridge::{UiJsonFrame, UiProviderSupportRole};

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

fn anthropic_config_toml() -> String {
    let mut config = valid_config_toml();
    config.push_str(
        r#"
[[model.providers]]
id = "anthropic"
kind = "anthropic"
default_model = "claude-3-5-sonnet-latest"
enabled = true
base_url = "https://api.anthropic.com/v1"
timeout_ms = 30000
api_key_provider = "env"
api_key_name = "ANTHROPIC_API_KEY"
"#,
    );
    config
}

fn gemini_config_toml() -> String {
    let mut config = valid_config_toml();
    config.push_str(
        r#"
[[model.providers]]
id = "gemini"
kind = "gemini"
default_model = "gemini-1.5-pro"
enabled = true
base_url = "https://generativelanguage.googleapis.com/v1beta"
timeout_ms = 30000
api_key_provider = "env"
api_key_name = "GEMINI_API_KEY"
"#,
    );
    config
}

fn local_config_toml() -> String {
    let mut config = valid_config_toml();
    config.push_str(
        r#"
[[model.providers]]
id = "local"
kind = "local"
default_model = "llama3.1"
enabled = true
base_url = "http://localhost:11434"
timeout_ms = 30000
"#,
    );
    config
}

fn local_fallback_config_toml() -> String {
    let mut config = valid_config_toml().replace(
        "fallback_provider_ids = [\"fake\"]",
        "fallback_provider_ids = [\"fake\", \"local\"]",
    );
    config.push_str(
        r#"
[[model.providers]]
id = "local"
kind = "local"
default_model = "llama3.1"
enabled = true
base_url = "http://localhost:11434"
timeout_ms = 30000
"#,
    );
    config
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
    assert!(output.stdout.contains(
        "Provider capabilities: fake: streaming=true tool_calls=false structured_outputs=false image_input=false context=4096 output=1024; openai: streaming=true tool_calls=false structured_outputs=false image_input=false context=128000 output=16384"
    ));
    assert!(output.stdout.contains("Settings profile: default"));
    assert!(output.stdout.contains("Release channel: dev"));
    assert!(output.stdout.contains(
        "Secret store: os_keychain available=false writable=false fallback=env_fallback"
    ));
    assert!(output.stdout.contains("Config dir: "));
}

#[test]
fn cli_config_diagnostics_text_reports_provider_support() {
    let path = write_config("support-text", &local_fallback_config_toml());
    let invocation = parse_cli_invocation(["--config-check", "--config", path.to_str().unwrap()])
        .expect("parse config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains(
        "Provider support: selected=openai missing=tool_calls; fallback[0]=fake missing=tool_calls; fallback[1]=local compatible=true"
    ));
    assert!(!output.stdout.contains("OPENAI_API_KEY"));
    assert!(!output.stdout.contains("localhost:11434"));
    assert!(!output.stdout.contains("api_key"));
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
    let fake_provider = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "fake")
        .expect("fake provider summary");
    assert!(fake_provider.capabilities.streaming);
    assert!(!fake_provider.capabilities.tool_calls);
    assert!(!fake_provider.capabilities.structured_outputs);
    assert!(!fake_provider.capabilities.image_input);
    assert_eq!(fake_provider.capabilities.max_context_tokens, Some(4096));
    assert_eq!(fake_provider.capabilities.max_output_tokens, Some(1024));
    assert!(fake_provider.capabilities.cost.is_none());
    let openai_provider = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "openai")
        .expect("openai provider summary");
    assert_eq!(
        openai_provider.capabilities.max_context_tokens,
        Some(128_000)
    );
    assert_eq!(openai_provider.capabilities.max_output_tokens, Some(16_384));
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
fn cli_config_diagnostics_json_renders_provider_support() {
    let path = write_config("support-json", &local_fallback_config_toml());
    let invocation = parse_cli_invocation([
        "--json",
        "--config-check",
        "--config",
        path.to_str().unwrap(),
    ])
    .expect("parse json config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let snapshot = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSnapshot(snapshot) = frame {
            Some(snapshot)
        } else {
            None
        }
    });
    let snapshot = snapshot.expect("config snapshot frame");
    let selected = snapshot
        .provider_support
        .iter()
        .find(|provider| provider.provider_id == "openai")
        .expect("selected support");
    let fake = snapshot
        .provider_support
        .iter()
        .find(|provider| provider.provider_id == "fake")
        .expect("fake support");
    let local = snapshot
        .provider_support
        .iter()
        .find(|provider| provider.provider_id == "local")
        .expect("local support");

    assert_eq!(selected.role, UiProviderSupportRole::Selected);
    assert!(!selected.compatible);
    assert_eq!(selected.missing, vec!["tool_calls"]);
    assert_eq!(fake.role, UiProviderSupportRole::Fallback);
    assert_eq!(fake.fallback_index, Some(0));
    assert_eq!(fake.missing, vec!["tool_calls"]);
    assert_eq!(local.role, UiProviderSupportRole::Fallback);
    assert_eq!(local.fallback_index, Some(1));
    assert!(local.compatible);
    assert!(local.missing.is_empty());
    assert!(!output.stdout.contains("OPENAI_API_KEY"));
    assert!(!output.stdout.contains("localhost:11434"));
}

#[test]
fn cli_config_diagnostics_text_reports_anthropic_capabilities() {
    let path = write_config("anthropic-text", &anthropic_config_toml());
    let invocation = parse_cli_invocation(["--config-check", "--config", path.to_str().unwrap()])
        .expect("parse config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Providers: anthropic, fake, openai"));
    assert!(output.stdout.contains(
        "anthropic: streaming=true tool_calls=true structured_outputs=false image_input=false context=200000 output=8192"
    ));
    assert!(!output.stdout.contains("ANTHROPIC_API_KEY"));
}

#[test]
fn cli_config_diagnostics_json_renders_anthropic_capabilities() {
    let path = write_config("anthropic-json", &anthropic_config_toml());
    let invocation = parse_cli_invocation([
        "--json",
        "--config-check",
        "--config",
        path.to_str().unwrap(),
        "--provider",
        "anthropic",
        "--model",
        "claude-3-5-sonnet-latest",
    ])
    .expect("parse json config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let snapshot = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSnapshot(snapshot) = frame {
            Some(snapshot)
        } else {
            None
        }
    });
    let snapshot = snapshot.expect("config snapshot frame");
    let anthropic_provider = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "anthropic")
        .expect("anthropic provider summary");

    assert_eq!(snapshot.selected_provider, "anthropic");
    assert!(anthropic_provider.capabilities.streaming);
    assert!(anthropic_provider.capabilities.tool_calls);
    assert!(!anthropic_provider.capabilities.structured_outputs);
    assert!(!anthropic_provider.capabilities.image_input);
    assert_eq!(
        anthropic_provider.capabilities.max_context_tokens,
        Some(200_000)
    );
    assert_eq!(
        anthropic_provider.capabilities.max_output_tokens,
        Some(8192)
    );
    assert!(anthropic_provider.capabilities.cost.is_none());
    assert!(!output.stdout.contains("ANTHROPIC_API_KEY"));
}

#[test]
fn cli_config_diagnostics_text_reports_gemini_capabilities() {
    let path = write_config("gemini-text", &gemini_config_toml());
    let invocation = parse_cli_invocation(["--config-check", "--config", path.to_str().unwrap()])
        .expect("parse config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Providers: fake, gemini, openai"));
    assert!(output.stdout.contains(
        "gemini: streaming=true tool_calls=true structured_outputs=false image_input=false context=1000000 output=8192"
    ));
    assert!(!output.stdout.contains("GEMINI_API_KEY"));
}

#[test]
fn cli_config_diagnostics_json_renders_gemini_capabilities() {
    let path = write_config("gemini-json", &gemini_config_toml());
    let invocation = parse_cli_invocation([
        "--json",
        "--config-check",
        "--config",
        path.to_str().unwrap(),
        "--provider",
        "gemini",
        "--model",
        "gemini-1.5-pro",
    ])
    .expect("parse json config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let snapshot = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSnapshot(snapshot) = frame {
            Some(snapshot)
        } else {
            None
        }
    });
    let snapshot = snapshot.expect("config snapshot frame");
    let gemini_provider = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "gemini")
        .expect("gemini provider summary");

    assert_eq!(snapshot.selected_provider, "gemini");
    assert!(gemini_provider.capabilities.streaming);
    assert!(gemini_provider.capabilities.tool_calls);
    assert!(!gemini_provider.capabilities.structured_outputs);
    assert!(!gemini_provider.capabilities.image_input);
    assert_eq!(
        gemini_provider.capabilities.max_context_tokens,
        Some(1_000_000)
    );
    assert_eq!(gemini_provider.capabilities.max_output_tokens, Some(8192));
    assert!(gemini_provider.capabilities.cost.is_none());
    assert!(!output.stdout.contains("GEMINI_API_KEY"));
}

#[test]
fn cli_config_diagnostics_text_reports_local_capabilities() {
    let path = write_config("local-text", &local_config_toml());
    let invocation = parse_cli_invocation(["--config-check", "--config", path.to_str().unwrap()])
        .expect("parse config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Providers: fake, local, openai"));
    assert!(output.stdout.contains(
        "local: streaming=true tool_calls=true structured_outputs=false image_input=false context=8192 output=2048"
    ));
    assert!(!output.stdout.contains("localhost:11434"));
}

#[test]
fn cli_config_diagnostics_json_renders_local_capabilities() {
    let path = write_config("local-json", &local_config_toml());
    let invocation = parse_cli_invocation([
        "--json",
        "--config-check",
        "--config",
        path.to_str().unwrap(),
        "--provider",
        "local",
        "--model",
        "llama3.1",
    ])
    .expect("parse json config check");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    let snapshot = frames.iter().find_map(|frame| {
        if let UiJsonFrame::ConfigSnapshot(snapshot) = frame {
            Some(snapshot)
        } else {
            None
        }
    });
    let snapshot = snapshot.expect("config snapshot frame");
    let local_provider = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "local")
        .expect("local provider summary");

    assert_eq!(snapshot.selected_provider, "local");
    assert!(local_provider.capabilities.streaming);
    assert!(local_provider.capabilities.tool_calls);
    assert!(!local_provider.capabilities.structured_outputs);
    assert!(!local_provider.capabilities.image_input);
    assert_eq!(local_provider.capabilities.max_context_tokens, Some(8192));
    assert_eq!(local_provider.capabilities.max_output_tokens, Some(2048));
    assert!(local_provider.capabilities.cost.is_none());
    assert!(!output.stdout.contains("localhost:11434"));
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
