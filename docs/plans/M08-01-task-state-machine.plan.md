# M08-01 Task State Machine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-scheduler` with task run contracts and legal task state transitions.

**Architecture:** `ruse-scheduler` depends on `ruse-types` only in this slice. It owns task metadata and transition validation; queueing, leases, quotas, and notifications are later M08 slices.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `thiserror`, existing `ruse-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-scheduler`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-scheduler/Cargo.toml`
- Create: `crates/ruse-scheduler/src/lib.rs`

- [x] **Step 1: Add crate scaffold**

Create the crate with dependencies:

```toml
[dependencies]
ruse-types = { path = "../ruse-types" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

Export placeholder modules only after RED tests are written.

### Task 2: Write Task State RED Tests

**Files:**
- Create: `crates/ruse-scheduler/tests/task_state_machine.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `task_state_machine_serializes_task_run_shape`
- `task_state_machine_allows_created_to_queued_to_running_to_succeeded`
- `task_state_machine_rejects_invalid_transition`
- `task_state_machine_rejects_terminal_transition`
- `task_state_machine_rejects_empty_summary_and_reason`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-scheduler task_state_machine
```

Expected: fails because task contracts do not exist.

### Task 3: Implement Minimal State Machine

**Files:**
- Create: `crates/ruse-scheduler/src/error.rs`
- Create: `crates/ruse-scheduler/src/task.rs`
- Modify: `crates/ruse-scheduler/src/lib.rs`

- [x] **Step 1: Add `SchedulerError`**

Add `InvalidTask`, `InvalidTransition`, and `InvalidLease` variants.

- [x] **Step 2: Add task contracts**

Implement `TaskKind`, `TaskPriority`, `TaskRun`, and `TaskTransition`.

- [x] **Step 3: Add transition validation**

Implement `transition_task_state` using the transition matrix from the spec.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p ruse-scheduler task_state_machine
```

Expected: focused tests pass.

### Task 4: Update Progress and Commit

**Files:**
- Modify: `docs/plans/M08-01-task-state-machine.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M08-02-queue-leases`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m08 task state machine"
```
