# M08 Background Tasks and Scheduler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Support durable long-running and recurring work with leases, retries, quotas, and notifications.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `agent-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

## Slice Execution Order

Do not implement this milestone directly from the milestone-level plan. Execute the slice plans in order:

1. `docs/plans/M08-01-task-state-machine.plan.md`
2. `docs/plans/M08-02-queue-leases.plan.md`
3. `docs/plans/M08-03-quotas.plan.md`
4. `docs/plans/M08-04-notifications.plan.md`

Each slice must be implemented with RED/GREEN tests, standard verification, progress updates, and a dedicated commit.

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M08-scheduler-background-tasks.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M08-scheduler-background-tasks.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M08.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/agent-core/tests/m08_scheduler_background_tasks.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- task transition tests
- lease acquisition tests
- retry schedule tests
- quota rejection tests
- waiting-approval notification tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m08_scheduler_background_tasks_validates_public_contracts`
- `m08_scheduler_background_tasks_rejects_invalid_inputs`
- `m08_scheduler_background_tasks_emits_expected_runtime_events`
- `m08_scheduler_background_tasks_preserves_serialized_shape`
- `m08_scheduler_background_tasks_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m08_scheduler_background_tasks
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/agent-scheduler/src/lib.rs`
- Modify/Create: `crates/agent-storage/src/lib.rs`
- Modify/Create: `crates/agent-core/src/lib.rs`
- Modify/Create: `crates/agent-types/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- TaskRun contract
- TaskLease contract
- RetryPolicy contract
- QuotaPolicy contract
- TaskNotification contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m08_scheduler_background_tasks
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/agent-core/tests/m08_scheduler_background_tasks_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- A background task can run, pause for approval, resume, complete, and expose a trace.
- Concurrent tasks respect workspace and system limits.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m08_scheduler_background_tasks_integration
```

Expected: integration tests pass and fail deterministically when the milestone behavior is removed.

### Task 5: Run Full Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Format check**

Run:

```bash
cargo fmt --all -- --check
```

Expected: formatting check passes.

- [ ] **Step 2: Lint check**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clippy passes without warnings.

- [ ] **Step 3: Full test suite**

Run:

```bash
cargo test --workspace
```

Expected: all workspace tests pass.

### Task 6: Update Progress and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Update active milestone status**

Update `docs/milestones/README.md` so the completed milestone is marked as complete or the active focus advances to the next milestone.

- [ ] **Step 2: Commit the milestone**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m08 scheduler background tasks"
```

Expected: commit succeeds with source, tests, and documentation staged together.
