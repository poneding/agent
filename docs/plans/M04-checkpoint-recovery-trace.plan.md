# M04 Checkpoint, Recovery, and Trace Replay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make interrupted turns durable, inspectable, and replayable without duplicating completed side effects.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `ruse-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M04-checkpoint-recovery-trace.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M04-checkpoint-recovery-trace.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M04.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/ruse-core/tests/m04_checkpoint_recovery_trace.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- checkpoint serialization tests
- idempotent tool replay tests
- interrupted turn recovery test
- trace command snapshot tests
- old trace replay fixture tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m04_checkpoint_recovery_trace_validates_public_contracts`
- `m04_checkpoint_recovery_trace_rejects_invalid_inputs`
- `m04_checkpoint_recovery_trace_emits_expected_runtime_events`
- `m04_checkpoint_recovery_trace_preserves_serialized_shape`
- `m04_checkpoint_recovery_trace_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m04_checkpoint_recovery_trace
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/ruse-storage/src/lib.rs`
- Modify/Create: `crates/ruse-core/src/lib.rs`
- Modify/Create: `crates/ruse-cli/src/lib.rs`
- Modify/Create: `crates/ruse-types/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- CheckpointStore contract
- IdempotencyKey contract
- RecoveryPlan contract
- TraceQuery contract
- Replay compatibility contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m04_checkpoint_recovery_trace
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/ruse-core/tests/m04_checkpoint_recovery_trace_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- Interrupted turns resume from checkpoint plus event log.
- Completed side-effecting calls are not duplicated.
- Users can inspect model, tool, policy, and final-state traces.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m04_checkpoint_recovery_trace_integration
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
git commit -m "feat: complete m04 checkpoint recovery trace"
```

Expected: commit succeeds with source, tests, and documentation staged together.
