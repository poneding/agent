# M24-05 Task Artifacts and Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add artifact metadata, review summaries, and final M24 smoke coverage for background tasks.

**Architecture:** `ruse-core` produces artifacts/reviews, `ruse-scheduler` links them to task runs, `ruse-ui-bridge` and UI crates render safe summaries.

**Tech Stack:** Rust 2024 edition, serde, trace/checkpoint ids, UI bridge snapshots, CLI/TUI/Desktop summary models, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-05-task-artifacts-review.spec.md`
- Read: `crates/ruse-core/src/trace.rs`
- Read: `crates/ruse-storage/src/checkpoint.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review artifact and review scope**

Expected: artifact metadata and review summary contracts are safe and UI-ready.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-core/tests/task_artifacts_review.rs`
- Create: `crates/ruse-ui-bridge/tests/task_review.rs`
- Create if needed: `crates/ruse-cli/tests/task_review_commands.rs`

- [x] **Step 1: Add artifact serialization tests**

Cover path/hash/summary shape and missing artifact diagnostics.

- [x] **Step 2: Add review summary tests**

Cover successful, failed, and cancelled tasks with trace links and artifact ids.

- [x] **Step 3: Add final M24 smoke test**

Cover create, run, approval/notification, quota/retry, artifact, and review fixture path.

- [x] **Step 4: Verify RED**

```bash
cargo test -p ruse-core task_artifacts_review
cargo test -p ruse-ui-bridge task_review
```

### Task 3: Implement Artifacts and Reviews

**Files:**
- Create: `crates/ruse-core/src/task_review.rs`
- Modify: `crates/ruse-core/src/background.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify: `crates/ruse-scheduler/src/task.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-tui/src/task_panel.rs`
- Modify: `crates/ruse-desktop/src/task.rs`

- [x] **Step 1: Add artifact contracts**

Implement `TaskArtifact`, artifact kind, safe summary, and validation.

- [x] **Step 2: Add review summaries**

Implement `TaskReviewSummary` for success/failure/cancelled outcomes.

- [x] **Step 3: Wire UI summaries**

Expose safe review snapshots to CLI/TUI/Desktop.

- [x] **Step 4: Add final smoke coverage**

Prove M24 lifecycle from command bridge through review output.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p ruse-core task_artifacts_review
cargo test -p ruse-ui-bridge task_review
```

- [x] **Step 2: Run final M24 checks**

```bash
cargo xtask check
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M24-05-task-artifacts-review.plan.md`
- Modify: `docs/plans/M24-background-agents-ux.plan.md`

- [x] **Step 1: Mark M24-05 and M24 complete**

Update milestone status and advance active focus to M25.

- [x] **Step 2: Commit M24 completion**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m24 background agents"
```
