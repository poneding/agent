# M15-02 Invocation Policy Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute model streams through configured retry and fallback policy.

**Architecture:** Extend `agent-core::model_runtime` with an invocation runner that calls `ModelProvider` handles from `agent-model`. Keep attempt history in memory and defer durable attempt events to later milestones.

**Tech Stack:** Rust 2024 edition, async model providers, existing invocation policy contracts, deterministic fake/failing providers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-02-invocation-policy-runner.spec.md`
- Read: `crates/agent-core/src/model_runtime.rs`
- Read: `crates/agent-model/src/invocation_policy.rs`
- Read: `crates/agent-model/src/provider.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M15-02-invocation-policy-runner.spec.md
```

Expected: the spec defines retry/fallback behavior, attempts, and failure handling.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/model_invocation_policy_runner.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `model_invocation_policy_retries_retryable_failure`
- `model_invocation_policy_does_not_retry_non_retryable_failure`
- `model_invocation_policy_uses_fallback_provider`
- `model_invocation_policy_reports_missing_fallback_safely`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core --test model_invocation_policy_runner
```

Expected: tests fail because invocation runner contracts do not exist.

### Task 3: Implement Invocation Runner

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`

- [x] **Step 1: Add attempt and outcome contracts**

Implement:

- `ModelInvocationAttempt`
- `ModelInvocationOutcome`
- `ModelInvocationRunner`

- [x] **Step 2: Implement retry and fallback**

Rules:

- retry selected provider for retryable failures until `max_attempts`
- do not retry non-retryable failures
- attempt fallback providers in configured order
- record provider/model/attempt index for each try

- [x] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-core --test model_invocation_policy_runner
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-02-invocation-policy-runner.plan.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

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
git commit -m "feat: add m15 invocation policy runner"
```

Expected: commit succeeds.
