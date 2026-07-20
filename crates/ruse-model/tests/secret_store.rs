use ruse_model::{
    EnvSecretStore, InMemorySecretStore, OsKeychainSecretStore, SecretRef, SecretResolutionError,
    SecretResolver, SecretStoreBackend, SecretStoreError, SecretStoreKind, SecretValue,
};

#[test]
fn secret_store_memory_lifecycle_is_redacted_and_idempotent() {
    let mut store = InMemorySecretStore::new("agent-secret").expect("store");
    let reference = store
        .put(
            "openai-api-key",
            SecretValue::new("sk-test-secret").expect("secret"),
        )
        .expect("put secret");

    assert_eq!(reference.provider, "agent-secret");
    assert_eq!(reference.name, "openai-api-key");
    assert_eq!(
        store.get(&reference).expect("get secret").expose_secret(),
        "sk-test-secret"
    );
    assert!(!format!("{store:?}").contains("sk-test-secret"));

    store.delete(&reference).expect("delete secret");
    store.delete(&reference).expect("delete is idempotent");
    assert!(matches!(
        store.get(&reference),
        Err(SecretStoreError::SecretNotFound { .. })
    ));
}

#[test]
fn secret_store_memory_rejects_invalid_secret_names_without_mutation() {
    let mut store = InMemorySecretStore::new("agent-secret").expect("store");
    let err = store
        .put(" ", SecretValue::new("sk-test-secret").expect("secret"))
        .expect_err("invalid name");

    assert!(matches!(err, SecretStoreError::InvalidReference(_)));
    assert!(store.health().available);
}

#[test]
fn secret_store_env_reads_environment_and_is_read_only() {
    let expected = std::env::var("PATH").expect("PATH should be present in test environment");
    let reference = SecretRef::new("env", "PATH").expect("env ref");
    let mut store = EnvSecretStore;

    assert_eq!(
        store.get(&reference).expect("env secret").expose_secret(),
        expected
    );
    assert!(matches!(
        store.put("NEW_SECRET", SecretValue::new("value").expect("secret")),
        Err(SecretStoreError::ReadOnly {
            kind: SecretStoreKind::Environment
        })
    ));
    assert!(!format!("{store:?}").contains(&expected));
}

#[test]
fn secret_store_keychain_boundary_reports_unavailable_and_falls_back_to_env() {
    let reference = SecretRef::new("env", "PATH").expect("env ref");
    let store = OsKeychainSecretStore::unavailable_with_env_fallback();

    assert!(store.get(&reference).is_ok());
    let health = store.health();
    assert_eq!(health.kind, SecretStoreKind::OsKeychain);
    assert!(!health.available);
    assert!(!health.writable);
    assert_eq!(
        health.diagnostics[0].code,
        "secret_store.keychain_unavailable"
    );
    assert!(!format!("{health:?}").contains("PATH="));
}

#[test]
fn secret_store_backend_can_drive_secret_resolver_contract() {
    let mut store = InMemorySecretStore::new("agent-secret").expect("store");
    let reference = store
        .put(
            "openai-api-key",
            SecretValue::new("sk-test-secret").expect("secret"),
        )
        .expect("put secret");

    let resolved = SecretResolver::resolve(&store, &reference).expect("resolved");

    assert_eq!(resolved.expose_secret(), "sk-test-secret");
    let missing = SecretRef::new("agent-secret", "missing").expect("missing ref");
    assert_eq!(
        SecretResolver::resolve(&store, &missing).expect_err("missing"),
        SecretResolutionError::SecretNotFound {
            provider: "agent-secret".to_string(),
            name: "missing".to_string(),
        }
    );
}
