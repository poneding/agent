# M14-01 Secret Resolver Spec

## Purpose

Add model-domain secret resolver contracts that validate secret references and safely wrap secret values without exposing raw credentials through debug, display, serialization, events, or UI output.

## Product Context

M13 introduced `SecretRef`; M14-01 makes that reference resolvable while preserving the core security rule that config and traces contain handles, not values.

## Scope

This spec delivers:

- `ruse-model::secrets`
- `SecretResolver` trait
- `SecretValue` redaction wrapper
- `InMemorySecretResolver`
- `EnvSecretResolver`
- typed secret resolution errors

Owned by later specs:

- OS keychain integration
- encrypted local secret store
- interactive secret prompts
- live HTTP transport credential injection

## Architecture

Secret resolution lives in `ruse-model` because model provider authentication belongs to the model domain. The module depends only on `SecretRef`, standard environment access, and local collections.

## Public Contracts

```rust
pub struct SecretValue;

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretResolutionError>;
    pub fn expose_secret(&self) -> &str;
}

pub trait SecretResolver: Send + Sync {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError>;
}

pub struct InMemorySecretResolver;
pub struct EnvSecretResolver;

pub enum SecretResolutionError {
    EmptySecretValue { provider: String, name: String },
    UnsupportedSecretProvider { provider: String },
    SecretNotFound { provider: String, name: String },
}
```

## State And Failure Handling

- empty resolved values fail
- unsupported providers fail before any value is read
- missing values are recoverable errors
- cloned secret values preserve redaction behavior

## Security And Privacy

- `SecretValue` does not implement `Serialize`
- `Debug` and `Display` return a redacted marker
- error messages mention provider/name only
- tests include raw secret-shaped values and assert they do not appear in debug strings

## Testing Strategy

- `secret_value_debug_and_display_are_redacted`
- `secret_value_rejects_empty_values`
- `in_memory_secret_resolver_resolves_known_secret`
- `in_memory_secret_resolver_reports_missing_secret`
- `env_secret_resolver_reads_env_provider`
- `env_secret_resolver_rejects_non_env_provider`

## Acceptance Criteria

- `ruse-model` exports secret resolver contracts.
- Resolver tests pass without real credentials.
- No public formatter or serde output exposes raw secret values.

## Completion Signal

M14-01 is complete when `cargo test -p ruse-model --test secret_resolver` and the workspace gates pass, status advances to M14-02, and the slice is committed.
