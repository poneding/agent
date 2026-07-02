# M63-01 Team Workflow Snapshot and Readiness Contracts Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, serializable team workflow snapshot and readiness contracts that compose worktree leases, subagent handoffs, blackboard output, reviewer votes, conflict summaries, merge readiness, and capability ceilings.

**Architecture:** `agent-team` owns UI-independent workflow summary types and readiness rules where possible. `agent-core` owns runtime-facing fixture/snapshot assembly when it needs recovery conflict summaries or multiple lower-level crates. No CLI/TUI/Desktop rendering is implemented in this slice.

**Tech Stack:** Rust 2024 edition, serde, existing `agent-team`, `agent-worktree`, `agent-policy`, `agent-core`, `agent-types`, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M63-01-team-workflow-snapshot-readiness.spec.md`
- Read: `docs/specs/M63-worktree-agent-team-workflows.spec.md`
- Read: `crates/agent-team/src/handoff.rs`
- Read: `crates/agent-team/src/blackboard.rs`
- Read: `crates/agent-team/src/capability.rs`
- Read: `crates/agent-worktree/src/lease.rs`
- Read: `crates/agent-worktree/src/conflict.rs`
- Read: `crates/agent-core/src/conflict_recovery.rs`

- [x] **Step 1: Review contract foundations**

Confirm existing types already cover worktree leases, handoff envelopes, capability reduction, blackboard entries, review votes, and recovery-safe conflict summaries.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-team/tests/team_workflow_snapshot.rs`
- Create if needed: `crates/agent-core/tests/team_workflow_snapshot.rs`

- [ ] **Step 1: Add agent-team contract tests**

Cover:

- stable serialized shape
- readiness precedence
- capability ceiling does not expand grants
- missing review and waiting handoff states
- bounded/redacted summary strings

- [ ] **Step 2: Add agent-core fixture tests if assembly lives in core**

Cover:

- completed handoff plus approval and no conflict is merge-ready
- conflicts force has-conflict readiness
- serialized runtime snapshot does not leak secret-like fixture values

- [ ] **Step 3: Verify tests fail before implementation**

Run:

```bash
cargo test -p agent-team --test team_workflow_snapshot
cargo test -p agent-core --test team_workflow_snapshot
```

Expected: new tests fail because the snapshot contracts are not implemented yet.

### Task 3: Implement Contracts

**Files:**
- Create: `crates/agent-team/src/workflow.rs`
- Modify: `crates/agent-team/src/lib.rs`
- Modify if needed: `crates/agent-core/Cargo.toml`
- Create if needed: `crates/agent-core/src/team_workflow.rs`
- Modify if needed: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Add workflow types**

Add typed serde contracts for status, readiness, diagnostics, worktree summaries, handoff summaries, blackboard summaries, merge review summaries, capability ceiling summaries, and the top-level snapshot.

- [ ] **Step 2: Add readiness calculation**

Readiness must be deterministic:

- conflict beats merge-ready
- policy blocked beats ready states
- waiting handoff beats review/merge states
- missing review becomes review-required
- completed, approved, no-conflict state becomes merge-ready

- [ ] **Step 3: Add redaction/bounding helpers**

Use existing redaction helpers if available. Otherwise, keep summaries short and redact secret-like path/hash/objective values before serialization.

- [ ] **Step 4: Export public contracts**

Expose the new contracts from `agent-team` and, if core assembly exists, from `agent-core`.

### Task 4: Verify Focused Slice

**Files:**
- Read: `crates/agent-team/Cargo.toml`
- Read if modified: `crates/agent-core/Cargo.toml`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-team --test team_workflow_snapshot
cargo test -p agent-core --test team_workflow_snapshot
```

Expected: focused tests pass.

- [ ] **Step 2: Run focused clippy**

Run:

```bash
cargo clippy -p agent-team -p agent-core --all-targets -- -D warnings
```

Expected: clippy passes without warnings.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M63-01-team-workflow-snapshot-readiness.plan.md`
- Modify: `docs/plans/M63-worktree-agent-team-workflows.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Update checkboxes and M63 status**

Mark M63-01 implementation complete only after focused tests and clippy pass. Keep M63 overall in progress.

- [ ] **Step 2: Run final slice checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-team -p agent-core --all-targets -- -D warnings
cargo test -p agent-team --test team_workflow_snapshot
cargo test -p agent-core --test team_workflow_snapshot
git diff --check
```

Expected: all checks pass.

- [ ] **Step 3: Commit M63-01**

Exact-stage only M63-01 files:

```bash
git status --short --branch
git add crates/agent-team/src/workflow.rs crates/agent-team/src/lib.rs crates/agent-team/tests/team_workflow_snapshot.rs crates/agent-core/Cargo.toml crates/agent-core/src/team_workflow.rs crates/agent-core/src/lib.rs crates/agent-core/tests/team_workflow_snapshot.rs docs/plans/M63-01-team-workflow-snapshot-readiness.plan.md docs/plans/M63-worktree-agent-team-workflows.plan.md docs/milestones/README.md
git commit -m "feat: add m63 team workflow snapshots"
```

Expected: commit contains typed team workflow snapshot/readiness contracts, tests, and status updates.
