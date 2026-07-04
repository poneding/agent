# M13-03 Invocation Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add model invocation timeout, retry, and fallback policy contracts.

**Architecture:** Implement pure policy data and decision helpers in `agent-model::invocation_policy`. Runtime event emission is reserved for later `agent-core` integration.

**Tech Stack:** Rust 2024 edition, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M13-03-invocation-policy.spec.md`
- Read: `crates/agent-model/src/provider.rs`
- Read: `crates/agent-model/src/registry.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M13-03-invocation-policy.spec.md
```

Expected: the spec defines timeout, retry, and fallback contracts.

### Task 2: Write RED Invocation Policy Tests

**Files:**
- Create: `crates/agent-model/tests/invocation_policy.rs`

- [x] **Step 1: Add focused tests**

Add tests:

- `invocation_policy_serializes_stable_shape`
- `invocation_policy_rejects_zero_timeout`
- `retry_policy_classifies_retryable_errors`
- `fallback_chain_rejects_empty_and_duplicate_provider_ids`
- `fallback_chain_selects_first_compatible_provider`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-model --test invocation_policy
```

Expected: tests fail because invocation policy contracts do not exist.

### Task 3: Implement Invocation Policy

**Files:**
- Create: `crates/agent-model/src/invocation_policy.rs`
- Modify: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Add policy types**

Implement `ModelInvocationPolicy`, `ModelRetryPolicy`, and `ModelFallbackChain`.

- [x] **Step 2: Add decision helpers**

Implement retry classification and first-compatible fallback selection using registry capabilities.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-03-invocation-policy.plan.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo test -p agent-model --test invocation_policy
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 model invocation policy"
```

Expected: commit succeeds.
