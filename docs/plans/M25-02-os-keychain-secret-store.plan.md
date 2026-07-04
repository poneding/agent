# M25-02 OS Keychain Secret Store Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement OS keychain-oriented secret store contracts with env fallback and safe diagnostics.

**Architecture:** `agent-model` owns secret traits and provider resolution; CLI/Desktop call through settings flows and never render raw values.

**Tech Stack:** Rust 2024 edition, async trait, secret wrapper types, optional platform backends, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-02-os-keychain-secret-store.spec.md`
- Read: `docs/specs/M14-runtime-configuration-secrets.spec.md`
- Read: `crates/agent-model/src/secrets.rs`
- Read: `crates/agent-model/tests/secret_resolver.rs`

- [x] **Step 1: Review secret contracts**

Expected: existing `SecretRef` behavior and M25 storage requirements are aligned.

### Task 2: Write Failing Tests

**Files:**
- Create or modify: `crates/agent-model/tests/secret_store.rs`
- Modify if needed: `crates/agent-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Add backend lifecycle tests**

Cover put, get, delete, missing, duplicate, and idempotent delete.

- [x] **Step 2: Add redaction and provider tests**

Cover env fallback, missing secret before network call, and diagnostics without raw values.

- [x] **Step 3: Verify RED**

```bash
cargo test -p agent-model secret_store
```

### Task 3: Implement Secret Store

**Files:**
- Modify: `crates/agent-model/src/lib.rs`
- Create if needed: `crates/agent-model/src/secret_store.rs`
- Modify if needed: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Add backend trait and fixture**

Implement memory backend first for deterministic tests.

- [x] **Step 2: Add env fallback backend**

Resolve configured environment references with redacted failures.

- [x] **Step 3: Add OS keychain adapter boundary**

Add platform-aware backend selection and feature-gated adapter stubs where native integration is deferred.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-model secret_store
cargo test -p agent-cli config_diagnostics
```

- [x] **Step 2: Run workspace gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M25-02-os-keychain-secret-store.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [x] **Step 1: Mark M25-02 complete**

Advance active focus to M25-03.

- [x] **Step 2: Commit M25-02**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add secret store backends"
```
