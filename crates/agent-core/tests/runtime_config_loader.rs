use std::path::PathBuf;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, ConfigDiagnosticSeverity,
    load_agent_config_file, load_agent_config_toml,
};
use agent_model::{ModelFailureKind, ModelProviderKind};

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

#[test]
fn runtime_config_loader_parses_valid_toml() {
    let config = load_agent_config_toml(&valid_config_toml()).expect("valid config");

    assert_eq!(config.schema_version, AGENT_CONFIG_SCHEMA_VERSION);
    assert_eq!(config.model.default_provider, "openai");
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
