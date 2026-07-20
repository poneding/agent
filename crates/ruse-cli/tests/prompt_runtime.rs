use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn write_config(name: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ruse-cli-prompt-runtime-{name}-{}.toml",
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

fn openai_missing_secret_config_yaml(secret_name: &str) -> String {
    format!(
        r#"
schemaVersion: 1
model:
  defaultProvider: openai
  invocation:
    timeoutMs: 30000
    maxAttempts: 1
    retryableFailures: []
    fallbackProviderIds: []
  providers:
    - id: openai
      kind: openai-compatible
      defaultModel: gpt-4.1-mini
      enabled: true
      baseUrl: https://api.openai.com/v1
      timeoutMs: 30000
      apiKeyProvider: env
      apiKeyName: {secret_name}
"#
    )
}

fn local_unreachable_config_toml(base_url: &str) -> String {
    format!(
        r#"
schema_version = 1

[model]
default_provider = "local"

[model.invocation]
timeout_ms = 1000
max_attempts = 1
retryable_failures = []
fallback_provider_ids = []

[[model.providers]]
id = "local"
kind = "local"
default_model = "llama3.1"
enabled = true
base_url = "{base_url}"
timeout_ms = 1000
"#
    )
}

#[test]
fn cli_prompt_runtime_runs_default_fake_turn() {
    let home = tempfile::tempdir().expect("home");
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .env("HOME", home.path())
        .args(["--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fake response: hello\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_text_streams_deltas_without_json_frames() {
    let home = tempfile::tempdir().expect("home");
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .env("HOME", home.path())
        .args(["--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout, "fake response: hello\n");
    assert!(!stdout.contains("\"type\""));
    assert!(!stdout.contains("model_delta"));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_uses_configured_fake_provider() {
    let path = write_config("fake", &fake_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "configured"])
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fake response: configured\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_uses_default_config_yaml_without_flag() {
    let home = tempfile::tempdir().expect("home");
    let config_dir = home.path().join(".ruse");
    fs::create_dir_all(&config_dir).expect("config dir");
    let missing_secret = format!(
        "AGENT_TEST_DEFAULT_CONFIG_YAML_MISSING_{}",
        std::process::id()
    );
    fs::write(
        config_dir.join("config.yaml"),
        openai_missing_secret_config_yaml(&missing_secret),
    )
    .expect("write default config");

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .env("HOME", home.path())
        .args(["--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prompt runtime failed"));
    assert!(stdout.contains("openai"));
    assert!(!stdout.contains(&missing_secret));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_live_http_provider_has_tokio_drivers() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral local port");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    drop(listener);
    let path = write_config(
        "local-unreachable",
        &local_unreachable_config_toml(&base_url),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("prompt runtime failed"));
    assert!(!stderr.contains("panicked"));
    assert!(!stderr.contains("timers are disabled"));
}

#[test]
fn cli_prompt_runtime_rejects_missing_openai_secret_safely() {
    let path = write_config("openai", &openai_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prompt runtime failed"));
    assert!(stdout.contains("openai"));
    assert!(stdout.contains("secret"));
    assert!(!stdout.contains("model provider is not registered: openai"));
    assert!(!stdout.contains("OPENAI_API_KEY"));
    assert!(output.stderr.is_empty());
}

#[test]
fn cli_prompt_runtime_redacts_secret_like_values() {
    let path = write_config("openai-redaction", &openai_config_toml());

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["--config", path.to_str().unwrap(), "--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

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
