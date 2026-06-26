use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn write_config(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agent-cli-prompt-runtime-{name}-{}.toml",
        std::process::id()
    ));
    fs::write(&path, contents).expect("write config fixture");
    path
}

fn fake_config_toml() -> String {
    r#"
schema_version = 1

[model]
default_provider = "local"

[model.invocation]
timeout_ms = 30000
max_attempts = 1
retryable_failures = []
fallback_provider_ids = []

[[model.providers]]
id = "local"
kind = "fake"
default_model = "local-scripted"
enabled = true
timeout_ms = 1000
"#
    .to_string()
}

fn openai_config_toml() -> String {
    r#"
schema_version = 1

[model]
default_provider = "openai"

[model.invocation]
timeout_ms = 30000
max_attempts = 1
retryable_failures = []
fallback_provider_ids = []

[[model.providers]]
id = "openai"
kind = "open_ai_compatible"
default_model = "gpt-4.1-mini"
enabled = true
base_url = "https://api.openai.com/v1"
timeout_ms = 30000
api_key_provider = "env"
api_key_name = "OPENAI_API_KEY"
"#
    .to_string()
}

#[test]
fn cli_prompt_runtime_runs_default_fake_turn() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--prompt", "hello"])
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fake response: hello\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_uses_configured_fake_provider() {
    let path = write_config("fake", &fake_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "configured"])
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fake response: configured\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_rejects_unregistered_openai_provider_safely() {
    let path = write_config("openai", &openai_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "hello"])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("model provider is not registered: openai"));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_redacts_secret_like_values() {
    let path = write_config("openai-redaction", &openai_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "hello"])
        .output()
        .expect("agent-cli binary should run");

    let rendered = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!rendered.contains("OPENAI_API_KEY"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}
