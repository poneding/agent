use agent_model::{
    EnvSecretResolver, InMemorySecretResolver, SecretRef, SecretResolutionError, SecretResolver,
    SecretValue,
};

#[test]
fn secret_value_debug_and_display_are_redacted() {
    let value = SecretValue::new("sk-test-secret").expect("secret value");

    assert_eq!(format!("{value:?}"), "SecretValue(<redacted>)");
    assert_eq!(value.to_string(), "<redacted>");
    assert_eq!(value.expose_secret(), "sk-test-secret");
}

#[test]
fn secret_value_rejects_empty_values() {
    let error = SecretValue::new("   ").expect_err("empty values are rejected");

    assert_eq!(
        error,
        SecretResolutionError::EmptySecretValue {
            provider: "<direct>".to_string(),
            name: "<direct>".to_string(),
        }
    );
}

#[test]
fn in_memory_secret_resolver_resolves_known_secret() {
    let secret_ref = SecretRef::new("agent-secret", "openai-api-key").expect("secret ref");
    let resolver = InMemorySecretResolver::new()
        .with_secret(secret_ref.clone(), "sk-test-secret")
        .expect("resolver with secret");

    let value = resolver.resolve(&secret_ref).expect("resolved secret");

    assert_eq!(value.expose_secret(), "sk-test-secret");
    assert!(!format!("{resolver:?}").contains("sk-test-secret"));
}

#[test]
fn in_memory_secret_resolver_reports_missing_secret() {
    let secret_ref = SecretRef::new("agent-secret", "missing-api-key").expect("secret ref");
    let resolver = InMemorySecretResolver::new();

    let error = resolver.resolve(&secret_ref).expect_err("missing secret");

    assert_eq!(
        error,
        SecretResolutionError::SecretNotFound {
            provider: "agent-secret".to_string(),
            name: "missing-api-key".to_string(),
        }
    );
}

#[test]
fn env_secret_resolver_reads_env_provider() {
    let expected = std::env::var("PATH").expect("PATH should be present in test environment");
    assert!(!expected.is_empty());
    let secret_ref = SecretRef::new("env", "PATH").expect("secret ref");

    let value = EnvSecretResolver
        .resolve(&secret_ref)
        .expect("resolved env secret");

    assert_eq!(value.expose_secret(), expected);
    assert!(!format!("{value:?}").contains(&expected));
}

#[test]
fn env_secret_resolver_rejects_non_env_provider() {
    let secret_ref = SecretRef::new("agent-secret", "openai-api-key").expect("secret ref");

    let error = EnvSecretResolver
        .resolve(&secret_ref)
        .expect_err("unsupported provider");

    assert_eq!(
        error,
        SecretResolutionError::UnsupportedSecretProvider {
            provider: "agent-secret".to_string(),
        }
    );
}
