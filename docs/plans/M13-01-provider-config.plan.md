# M13-01 Provider Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add model provider config, endpoint config, secret references, catalog validation, and tests in `agent-model`.

**Architecture:** Keep config contracts inside `agent-model::config`. Do not add file loading, secret value resolution, live network calls, or runtime orchestration in this slice.

**Tech Stack:** Rust 2024 edition, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M13-01-provider-config.spec.md`
- Read: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`
- Read: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,260p' docs/specs/M13-01-provider-config.spec.md
```

Expected: the spec defines provider config contracts, validation, security constraints, and tests.

### Task 2: Write RED Provider Config Tests

**Files:**
- Create: `crates/agent-model/tests/provider_config.rs`

- [x] **Step 1: Add focused tests**

Add tests:

- `provider_config_serializes_stable_shape`
- `provider_config_requires_secret_ref_for_openai_compatible`
- `provider_config_rejects_invalid_ids_models_and_timeouts`
- `provider_config_rejects_unsafe_endpoint_urls`
- `provider_catalog_rejects_duplicate_provider_ids`
- `provider_catalog_requires_enabled_provider`
- `secret_ref_serializes_without_raw_secret_value`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-model --test provider_config
```

Expected: tests fail because provider config contracts do not exist.

### Task 3: Implement Provider Config Contracts

**Files:**
- Create: `crates/agent-model/src/config.rs`
- Modify: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Add config types**

Implement `SecretRef`, `ModelProviderKind`, `ModelEndpointConfig`, `ModelProviderConfig`, and `ModelProviderCatalog` with serde derives.

- [x] **Step 2: Add validation**

Implement constructors and `validate` methods that reject empty ids, unsafe ids, empty model names, zero timeouts, unsafe endpoint URLs, missing OpenAI-compatible API key references, duplicate catalog ids, and catalogs with no enabled providers.

- [x] **Step 3: Export contracts**

Export the config types and `ModelConfigError` from `crates/agent-model/src/lib.rs`.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-model/tests/provider_config.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-model --test provider_config
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-01-provider-config.plan.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`
- Modify: `README.md`

- [x] **Step 1: Mark M13-01 complete and set active focus to M13-02**

Update docs so the active spec and plan point at:

```text
docs/specs/M13-02-provider-registry-capabilities.spec.md
docs/plans/M13-02-provider-registry-capabilities.plan.md
```

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 provider config"
```

Expected: commit succeeds with source, tests, and documentation together.
