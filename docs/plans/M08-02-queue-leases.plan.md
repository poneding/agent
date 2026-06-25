# M08-02 Queue and Leases Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add in-memory task queue, active leases, cancellation, pause/resume, and retry decisions.

**Architecture:** Queue behavior stays in `agent-scheduler` and builds on `TaskRun` from M08-01. Storage-backed queues remain out of scope.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `thiserror`, existing `agent-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M08-02-queue-leases.spec.md`
- Read: `crates/agent-scheduler/src/task.rs`

- [ ] **Step 1: Review the spec**

```bash
sed -n '1,280p' docs/specs/M08-02-queue-leases.spec.md
```

### Task 2: Write Queue and Lease RED Tests

**Files:**
- Create: `crates/agent-scheduler/tests/queue_leases.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- `queue_leases_acquires_queued_task_and_creates_active_lease`
- `queue_leases_complete_requires_active_lease`
- `queue_leases_failure_schedules_retry_until_exhausted`
- `queue_leases_cancel_pause_and_resume_update_state`
- `queue_leases_release_expired_leases_requeues_task`

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-scheduler queue_leases
```

Expected: fails because queue and lease APIs do not exist.

### Task 3: Implement Queue, Leases, and Retry

**Files:**
- Create: `crates/agent-scheduler/src/lease.rs`
- Create: `crates/agent-scheduler/src/retry.rs`
- Create: `crates/agent-scheduler/src/queue.rs`
- Modify: `crates/agent-scheduler/src/lib.rs`

- [ ] **Step 1: Add lease contracts**

Implement `TaskLeaseId`, `TaskLease`, and `LeaseStatus`.

- [ ] **Step 2: Add retry contracts**

Implement `RetryPolicy` and `RetryDecision`.

- [ ] **Step 3: Add `TaskQueue` and `InMemoryTaskQueue`**

Implement enqueue, acquire, complete, fail, cancel, pause, resume, and release-expired behavior.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test -p agent-scheduler queue_leases
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `docs/plans/M08-02-queue-leases.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Advance active focus**

Set active focus to `M08-03-quotas`.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m08 queue leases"
```
