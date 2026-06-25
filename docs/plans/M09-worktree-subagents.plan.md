# M09 Worktree Isolation and Subagents Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable safe parallel work through isolated worktrees, conflict detection, reduced-capability subagents, and a team blackboard.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `agent-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M09-worktree-subagents.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M09-worktree-subagents.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M09.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/agent-core/tests/m09_worktree_subagents.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- worktree lifecycle tests
- conflict detection fixture tests
- capability reduction tests
- handoff serialization tests
- blackboard ordering tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m09_worktree_subagents_validates_public_contracts`
- `m09_worktree_subagents_rejects_invalid_inputs`
- `m09_worktree_subagents_emits_expected_runtime_events`
- `m09_worktree_subagents_preserves_serialized_shape`
- `m09_worktree_subagents_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m09_worktree_subagents
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/agent-worktree/src/lib.rs`
- Modify/Create: `crates/agent-team/src/lib.rs`
- Modify/Create: `crates/agent-policy/src/lib.rs`
- Modify/Create: `crates/agent-storage/src/lib.rs`
- Modify/Create: `crates/agent-core/src/lib.rs`
- Modify/Create: `crates/agent-types/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- WorktreeLease contract
- ConflictReport contract
- SubagentProfile contract
- HandoffEnvelope contract
- BlackboardEntry contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m09_worktree_subagents
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/agent-core/tests/m09_worktree_subagents_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- A lead agent can delegate isolated subtasks with reduced permissions.
- Subagent results return as artifacts, summaries, diffs, and trace links.
- Worktree conflicts are detected before consolidation.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m09_worktree_subagents_integration
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
git commit -m "feat: complete m09 worktree subagents"
```

Expected: commit succeeds with source, tests, and documentation staged together.
