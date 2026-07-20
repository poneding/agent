use std::path::PathBuf;

use ruse_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, ConfigDiagnosticSeverity, RuntimeTerminalTheme,
    default_agent_config_path_for_home, legacy_agent_config_path_for_home, load_agent_config_file,
    load_agent_config_toml, load_agent_config_yaml_with_secrets,
    resolve_agent_config_path_for_home, write_agent_config_terminal_theme_atomic,
};
use ruse_model::{ModelFailureKind, ModelProviderKind, SecretResolver};

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

fn valid_config_yaml() -> String {
    r#"
schemaVersion: 1
terminal:
  theme: system
model:
  defaultProvider: openai
  invocation:
    timeoutMs: 30000
    maxAttempts: 2
    retryableFailures:
      - timeout
      - transport
    fallbackProviderIds:
      - fake
  providers:
    - id: openai
      kind: openai-compatible
      defaultModel: gpt-4.1-mini
      enabled: true
      baseUrl: https://api.openai.com/v1
      timeoutMs: 30000
      apiKey: sk-test-direct-key
    - id: fake
      kind: fake
      defaultModel: fake-scripted
      enabled: true
      timeoutMs: 1000
"#
    .to_string()
}

#[test]
fn runtime_config_loader_parses_valid_toml() {
    let config = load_agent_config_toml(&valid_config_toml()).expect("valid config");

    assert_eq!(config.schema_version, AGENT_CONFIG_SCHEMA_VERSION);
    assert_eq!(config.model.default_provider, "openai");
    assert_eq!(config.terminal.theme.as_str(), "system");
    assert_eq!(config.model.catalog.providers.len(), 2);
    let openai = &config.model.catalog.providers[0];
    assert_eq!(openai.id, "openai");
    assert_eq!(openai.kind, ModelProviderKind::OpenAiCompatible);
    assert_eq!(openai.default_model, "gpt-4.1-mini");
    assert_eq!(
        openai.api_key.as_ref().map(|secret| secret.name.as_str()),
        Some("OPENAI_API_KEY")
    );
    assert_eq!(config.model.invocation_policy.timeout_ms, 30_000);
    assert_eq!(config.model.invocation_policy.retry.max_attempts, 2);
    assert_eq!(
        config.model.invocation_policy.retry.retryable_failures,
        vec![
            ModelFailureKind::Timeout,
            ModelFailureKind::Transport,
            ModelFailureKind::RateLimited,
        ]
    );
    assert_eq!(
        config.model.invocation_policy.fallback.provider_ids,
        vec!["fake".to_string()]
    );
}

#[test]
fn runtime_config_loader_parses_terminal_theme() {
    let input = valid_config_toml().replace(
        "[model]\ndefault_provider",
        "[terminal]\ntheme = \"high-contrast\"\n\n[model]\ndefault_provider",
    );

    let config = load_agent_config_toml(&input).expect("valid config with terminal theme");

    assert_eq!(config.terminal.theme.as_str(), "high-contrast");
}

#[test]
fn runtime_config_loader_parses_yaml_with_direct_api_key_secret() {
    let loaded = load_agent_config_yaml_with_secrets(&valid_config_yaml()).expect("valid yaml");

    assert_eq!(loaded.config.schema_version, AGENT_CONFIG_SCHEMA_VERSION);
    assert_eq!(loaded.config.model.default_provider, "openai");
    let openai = &loaded.config.model.catalog.providers[0];
    assert_eq!(openai.kind, ModelProviderKind::OpenAiCompatible);
    let secret_ref = openai.api_key.as_ref().expect("api key ref");
    assert_eq!(secret_ref.provider, "config");
    let resolved = loaded
        .inline_secret_resolver
        .resolve(secret_ref)
        .expect("direct config api key resolves");
    assert_eq!(resolved.expose_secret(), "sk-test-direct-key");

    let serialized = serde_json::to_string(&loaded.config).expect("serialize config");
    assert!(!serialized.contains("sk-test-direct-key"));
    assert!(!serialized.contains("OPENAI_API_KEY"));
}

#[test]
fn runtime_config_loader_default_path_is_config_yaml_with_legacy_toml_path() {
    let home = PathBuf::from("/Users/example");

    assert_eq!(
        default_agent_config_path_for_home(&home),
        home.join(".config/agent/config.yaml")
    );
    assert_eq!(
        legacy_agent_config_path_for_home(&home),
        home.join(".config/agent/agent.toml")
    );
}

#[test]
fn runtime_config_loader_default_resolution_prefers_config_yaml() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_dir = temp.path().join(".config/agent");
    std::fs::create_dir_all(&config_dir).expect("config dir");
    let default_path = config_dir.join("config.yaml");
    let legacy_path = config_dir.join("agent.toml");
    std::fs::write(&default_path, valid_config_yaml()).expect("write yaml");
    std::fs::write(&legacy_path, valid_config_toml()).expect("write toml");

    let resolution = resolve_agent_config_path_for_home(temp.path());

    assert_eq!(
        resolution.load_path.as_deref(),
        Some(default_path.as_path())
    );
    assert_eq!(resolution.write_path, default_path);
    assert!(resolution.diagnostics.is_empty());
}

#[test]
fn runtime_config_loader_default_resolution_falls_back_to_legacy_toml_with_diagnostic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_dir = temp.path().join(".config/agent");
    std::fs::create_dir_all(&config_dir).expect("config dir");
    let default_path = config_dir.join("config.yaml");
    let legacy_path = config_dir.join("agent.toml");
    std::fs::write(&legacy_path, valid_config_toml()).expect("write toml");

    let resolution = resolve_agent_config_path_for_home(temp.path());

    assert_eq!(resolution.load_path.as_deref(), Some(legacy_path.as_path()));
    assert_eq!(resolution.write_path, default_path);
    assert_eq!(resolution.diagnostics.len(), 1);
    assert_eq!(
        resolution.diagnostics[0].code,
        "config.legacy_toml_fallback"
    );
    assert!(resolution.diagnostics[0].message.contains("config.yaml"));
}

#[test]
fn runtime_config_loader_writes_terminal_theme_atomically() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("agent.toml");
    std::fs::write(&path, valid_config_toml()).expect("write config");

    write_agent_config_terminal_theme_atomic(&path, RuntimeTerminalTheme::Dark)
        .expect("write terminal theme");

    let updated = load_agent_config_file(&path).expect("updated config");
    assert_eq!(updated.terminal.theme, RuntimeTerminalTheme::Dark);
    let contents = std::fs::read_to_string(&path).expect("read updated config");
    assert!(contents.contains("[terminal]"));
    assert!(contents.contains("theme = \"dark\""));
}

#[test]
fn runtime_config_loader_writes_yaml_terminal_theme_atomically() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("config.yaml");
    std::fs::write(&path, valid_config_yaml()).expect("write config");

    write_agent_config_terminal_theme_atomic(&path, RuntimeTerminalTheme::Dark)
        .expect("write terminal theme");

    let updated = load_agent_config_file(&path).expect("updated config");
    assert_eq!(updated.terminal.theme, RuntimeTerminalTheme::Dark);
    let contents = std::fs::read_to_string(&path).expect("read updated config");
    assert!(contents.contains("terminal:"));
    assert!(contents.contains("theme: dark"));
}

#[test]
fn runtime_config_loader_rejects_unsupported_schema_version() {
    let input = valid_config_toml().replace("schema_version = 1", "schema_version = 99");

    let error = load_agent_config_toml(&input).expect_err("unsupported schema");

    assert_eq!(
        error,
        AgentConfigError::UnsupportedSchemaVersion {
            found: 99,
            supported: AGENT_CONFIG_SCHEMA_VERSION,
        }
    );
}

#[test]
fn runtime_config_loader_rejects_unknown_keys() {
    let input = valid_config_toml().replace(
        "[model]\ndefault_provider",
        "[model]\nunexpected = true\ndefault_provider",
    );

    let error = load_agent_config_toml(&input).expect_err("unknown key rejected");

    assert!(
        matches!(error, AgentConfigError::Parse { message } if message.contains("unknown field"))
    );
}

#[test]
fn runtime_config_loader_rejects_raw_secret_value_keys() {
    let input = valid_config_toml().replace(
        "api_key_name = \"OPENAI_API_KEY\"",
        "api_key_name = \"OPENAI_API_KEY\"\napi_key_value = \"sk-test-secret\"",
    );

    let error = load_agent_config_toml(&input).expect_err("raw secret key rejected");

    let AgentConfigError::InvalidConfig { diagnostics } = error else {
        panic!("expected invalid config diagnostics");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, ConfigDiagnosticSeverity::Error);
    assert_eq!(diagnostics[0].code, "config.raw_secret_key");
    assert_eq!(
        diagnostics[0].field_path.as_deref(),
        Some("model.providers[0].api_key_value")
    );
    assert!(!diagnostics[0].message.contains("sk-test-secret"));
}

#[test]
fn runtime_config_loader_reports_invalid_provider_field_path() {
    let input = valid_config_toml().replace("id = \"openai\"", "id = \"OpenAI\"");

    let error = load_agent_config_toml(&input).expect_err("invalid provider id rejected");

    let AgentConfigError::InvalidConfig { diagnostics } = error else {
        panic!("expected invalid config diagnostics");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "model.provider.invalid");
    assert_eq!(
        diagnostics[0].field_path.as_deref(),
        Some("model.providers[0].id")
    );
    assert!(!diagnostics[0].message.contains("sk-"));
}

#[test]
fn runtime_config_loader_reports_missing_file() {
    let path = PathBuf::from("target/agent-missing-config-for-test.toml");

    let error = load_agent_config_file(&path).expect_err("missing file");

    assert!(matches!(
        error,
        AgentConfigError::FileRead { path: error_path, .. } if error_path.ends_with("agent-missing-config-for-test.toml")
    ));
}
