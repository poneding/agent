# M24-05 Task Artifacts and Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add artifact metadata, review summaries, and final M24 smoke coverage for background tasks.

**Architecture:** `agent-core` produces artifacts/reviews, `agent-scheduler` links them to task runs, `agent-ui-bridge` and UI crates render safe summaries.

**Tech Stack:** Rust 2024 edition, serde, trace/checkpoint ids, UI bridge snapshots, CLI/TUI/Desktop summary models, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-05-task-artifacts-review.spec.md`
- Read: `crates/agent-core/src/trace.rs`
- Read: `crates/agent-storage/src/checkpoint.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [ ] **Step 1: Review artifact and review scope**

Expected: artifact metadata and review summary contracts are safe and UI-ready.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/task_artifacts_review.rs`
- Create: `crates/agent-ui-bridge/tests/task_review.rs`
- Create if needed: `crates/agent-cli/tests/task_review_commands.rs`

- [ ] **Step 1: Add artifact serialization tests**

Cover path/hash/summary shape and missing artifact diagnostics.

- [ ] **Step 2: Add review summary tests**

Cover successful, failed, and cancelled tasks with trace links and artifact ids.

- [ ] **Step 3: Add final M24 smoke test**

Cover create, run, approval/notification, quota/retry, artifact, and review fixture path.

- [ ] **Step 4: Verify RED**

```bash
cargo test -p agent-core task_artifacts_review
cargo test -p agent-ui-bridge task_review
```

### Task 3: Implement Artifacts and Reviews

**Files:**
- Create: `crates/agent-core/src/task_review.rs`
- Modify: `crates/agent-core/src/background.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Modify: `crates/agent-scheduler/src/task.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify: `crates/agent-desktop/src/task.rs`

- [ ] **Step 1: Add artifact contracts**

Implement `TaskArtifact`, artifact kind, safe summary, and validation.

- [ ] **Step 2: Add review summaries**

Implement `TaskReviewSummary` for success/failure/cancelled outcomes.

- [ ] **Step 3: Wire UI summaries**

Expose safe review snapshots to CLI/TUI/Desktop.

- [ ] **Step 4: Add final smoke coverage**

Prove M24 lifecycle from command bridge through review output.

### Task 4: Verify

- [ ] **Step 1: Run focused tests**

```bash
cargo test -p agent-core task_artifacts_review
cargo test -p agent-ui-bridge task_review
```

- [ ] **Step 2: Run final M24 checks**

```bash
cargo xtask check
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M24-05-task-artifacts-review.plan.md`
- Modify: `docs/plans/M24-background-agents-ux.plan.md`

- [ ] **Step 1: Mark M24-05 and M24 complete**

Update milestone status and advance active focus to M25.

- [ ] **Step 2: Commit M24 completion**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m24 background agents"
```
