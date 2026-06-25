# M08-03 Quotas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic scheduler quota decisions and queue acquisition checks.

**Architecture:** Quota evaluation is a pure `agent-scheduler` function over `QuotaPolicy`, `QuotaSnapshot`, and a candidate `TaskRun`. Queue integration skips rejected candidates without mutating their state.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-scheduler`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M08-03-quotas.spec.md`
- Read: `crates/agent-scheduler/src/queue.rs`

- [x] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M08-03-quotas.spec.md
```

### Task 2: Write Quota RED Tests

**Files:**
- Create: `crates/agent-scheduler/tests/quotas.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `quotas_allow_when_all_limits_have_capacity`
- `quotas_reject_system_limit`
- `quotas_reject_workspace_limit`
- `quotas_reject_background_and_cron_limits`
- `quotas_queue_acquire_skips_rejected_candidate`

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-scheduler quotas
```

### Task 3: Implement Quotas

**Files:**
- Create: `crates/agent-scheduler/src/quota.rs`
- Modify: `crates/agent-scheduler/src/queue.rs`
- Modify: `crates/agent-scheduler/src/lib.rs`

- [x] **Step 1: Add quota contracts**

Implement `QuotaPolicy`, `QuotaSnapshot`, `QuotaDecision`, and `QuotaRejection`.

- [x] **Step 2: Add pure evaluator**

Implement `evaluate_task_quota`.

- [x] **Step 3: Add queue quota-aware acquisition**

Add an acquisition path that evaluates candidates before leasing.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p agent-scheduler quotas
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `docs/plans/M08-03-quotas.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M08-04-notifications`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m08 quotas"
```
