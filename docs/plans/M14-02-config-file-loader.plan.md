# M14-02 Config File Loader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add schema version 1 YAML runtime config loading in `ruse-core`, with legacy TOML compatibility.

**Architecture:** Keep YAML/TOML parsing and runtime composition in `ruse-core::config`; reuse `ruse-model` provider config and invocation policy contracts. Direct `apiKey` values are converted into an in-memory secret resolver; UI/CLI consumers still receive only redacted typed state.

**Tech Stack:** Rust 2024 edition, serde, serde_yaml, toml, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M14-02-config-file-loader.spec.md`
- Read: `docs/specs/M14-00-runtime-configuration-secrets-technical-design.spec.md`
- Read: `crates/ruse-core/src/lib.rs`
- Read: `crates/ruse-model/src/config.rs`
- Read: `crates/ruse-model/src/invocation_policy.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,240p' docs/specs/M14-02-config-file-loader.spec.md
```

Expected: the spec defines YAML schema, legacy TOML compatibility, contracts, validation, and tests.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-core/tests/runtime_config_loader.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `runtime_config_loader_parses_valid_yaml`
- `runtime_config_loader_parses_legacy_toml`
- `runtime_config_loader_parses_direct_api_key`
- `runtime_config_loader_resolves_default_config_yaml_before_legacy_toml`
- `runtime_config_loader_rejects_unsupported_schema_version`
- `runtime_config_loader_rejects_unknown_keys`
- `runtime_config_loader_rejects_raw_secret_value_keys`
- `runtime_config_loader_reports_invalid_provider_field_path`
- `runtime_config_loader_reports_missing_file`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-core --test runtime_config_loader
```

Expected: tests fail because runtime config loading is not implemented.

### Task 3: Implement Config Loader

**Files:**
- Modify: `crates/ruse-core/Cargo.toml`
- Create: `crates/ruse-core/src/config.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add dependency**

Add to `crates/ruse-core/Cargo.toml`:

```toml
serde_yaml = "0.9"
toml = "0.8"
```

- [x] **Step 2: Add config contracts**

Implement:

- `AGENT_CONFIG_SCHEMA_VERSION`
- `AgentRuntimeConfig`
- `RuntimeModelConfig`
- `ConfigDiagnostic`
- `ConfigDiagnosticSeverity`
- `AgentConfigError`
- `load_agent_config_yaml`
- `load_agent_config_toml`
- `load_agent_config_file`
- `resolve_agent_config_path`

- [x] **Step 3: Map YAML/TOML into model contracts**

Use existing constructors for:

- `ModelProviderConfig`
- `ModelEndpointConfig`
- `ModelProviderCatalog`
- `ModelInvocationPolicy`
- `ModelRetryPolicy`
- `ModelFallbackChain`

- [x] **Step 4: Verify focused tests**

Run:

```bash
cargo test -p ruse-core --test runtime_config_loader
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-02-config-file-loader.plan.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m14 runtime config loader"
```

Expected: commit succeeds.
