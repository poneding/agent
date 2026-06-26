# M14-01 Secret Resolver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted secret value and resolver contracts to `agent-model`.

**Architecture:** Implement `agent-model::secrets` with no dependency on runtime, storage, CLI, Desktop, or UI bridge. Export stable contracts from `agent-model::lib`.

**Tech Stack:** Rust 2024 edition, thiserror, standard environment access, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M14-01-secret-resolver.spec.md`
- Read: `crates/agent-model/src/config.rs`
- Read: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M14-01-secret-resolver.spec.md
```

Expected: the spec defines resolver contracts, privacy rules, tests, and acceptance criteria.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-model/tests/secret_resolver.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `secret_value_debug_and_display_are_redacted`
- `secret_value_rejects_empty_values`
- `in_memory_secret_resolver_resolves_known_secret`
- `in_memory_secret_resolver_reports_missing_secret`
- `env_secret_resolver_reads_env_provider`
- `env_secret_resolver_rejects_non_env_provider`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-model --test secret_resolver
```

Expected: tests fail because `agent_model::secrets` does not exist.

### Task 3: Implement Secret Resolver Contracts

**Files:**
- Create: `crates/agent-model/src/secrets.rs`
- Modify: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Add implementation**

Implement:

- `SecretValue`
- `SecretResolver`
- `InMemorySecretResolver`
- `EnvSecretResolver`
- `SecretResolutionError`

Ensure `SecretValue` does not derive `Serialize` and formats as `<redacted>`.

- [x] **Step 2: Export contracts**

Update `crates/agent-model/src/lib.rs` with:

```rust
pub mod secrets;

pub use secrets::{
    EnvSecretResolver, InMemorySecretResolver, SecretResolutionError, SecretResolver, SecretValue,
};
```

- [x] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-model --test secret_resolver
```

Expected: all secret resolver tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-01-secret-resolver.plan.md`
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
git commit -m "feat: add m14 secret resolver"
```

Expected: commit succeeds.
