# M24-04 Task Quotas, Retries, and Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce background task resource limits and recover stale leases/retryable failures deterministically.

**Architecture:** `ruse-scheduler` owns quota/retry/recovery state; `ruse-core` reports usage and observes cancellation; UI crates render summaries only.

**Tech Stack:** Rust 2024 edition, scheduler quota/retry primitives, fake usage snapshots, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-04-task-quotas-retries-recovery.spec.md`
- Read: `crates/ruse-scheduler/src/quota.rs`
- Read: `crates/ruse-scheduler/src/retry.rs`
- Read: `crates/ruse-scheduler/src/queue.rs`

- [x] **Step 1: Review quota and recovery scope**

Expected: quota dimensions, retry decisions, stale lease recovery, and cancellation behavior are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-scheduler/tests/task_quotas_retries_recovery.rs`
- Create if needed: `crates/ruse-core/tests/background_task_cancellation.rs`

- [x] **Step 1: Add quota tests**

Cover running, model call, tool call, wall-clock, output byte, and worktree quota rejection.

- [x] **Step 2: Add retry tests**

Cover retry backoff, sleeping-to-queued wakeup, and retry exhaustion.

- [x] **Step 3: Add stale lease recovery tests**

Cover expired active leases requeue/fail according to policy.

- [x] **Step 4: Verify RED**

```bash
cargo test -p ruse-scheduler task_quotas_retries_recovery
```

### Task 3: Implement Quotas, Retries, Recovery

**Files:**
- Modify: `crates/ruse-scheduler/src/quota.rs`
- Modify: `crates/ruse-scheduler/src/retry.rs`
- Modify: `crates/ruse-scheduler/src/queue.rs`
- Create: `crates/ruse-scheduler/src/recovery.rs`
- Modify: `crates/ruse-scheduler/src/lib.rs`
- Modify if needed: `crates/ruse-core/src/background.rs`

- [x] **Step 1: Add usage quota contracts**

Implement task usage snapshots and quota rejection summaries.

- [x] **Step 2: Extend retry visibility**

Expose attempt summaries, next run time, and exhausted failure reason.

- [x] **Step 3: Add stale lease recovery report**

Implement deterministic recovery for expired leases.

- [x] **Step 4: Add cancellation propagation hooks**

Runner observes cancellation and maps it to scheduler state.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p ruse-scheduler task_quotas_retries_recovery
```

- [x] **Step 2: Run workspace checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M24-04 complete**

Advance focus to M24-05.

- [x] **Step 2: Commit M24-04**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add task quotas and recovery"
```
