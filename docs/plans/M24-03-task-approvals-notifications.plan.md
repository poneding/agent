# M24-03 Task Approvals and Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pause background tasks on approval requests and route safe task notifications to UI clients.

**Architecture:** `ruse-core` maps approval suspension to scheduler state; `ruse-scheduler` owns notifications; `ruse-ui-bridge` owns UI-safe frames.

**Tech Stack:** Rust 2024 edition, M19 approval contracts, scheduler notifications, UI bridge snapshots, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-03-task-approvals-notifications.spec.md`
- Read: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`
- Read: `crates/ruse-core/src/approval.rs`
- Read: `crates/ruse-scheduler/src/notification.rs`

- [x] **Step 1: Review approval and notification scope**

Expected: approval waits pause tasks, release compute lease, and notify UI clients.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-core/tests/background_task_approvals.rs`
- Create: `crates/ruse-ui-bridge/tests/task_notifications.rs`
- Modify if needed: `crates/ruse-scheduler/tests/notifications.rs`

- [x] **Step 1: Add approval suspension tests**

Cover policy ask, waiting approval state, lease release, and no tool execution before approval.

- [x] **Step 2: Add resume/deny tests**

Cover allow resumes continuation and deny cancels/fails safely.

- [x] **Step 3: Add UI bridge notification snapshots**

Cover task notification JSON and redaction.

- [x] **Step 4: Verify RED**

```bash
cargo test -p ruse-core background_task_approvals
cargo test -p ruse-ui-bridge task_notifications
```

### Task 3: Implement Approval Notifications

**Files:**
- Modify: `crates/ruse-core/src/background.rs`
- Modify: `crates/ruse-scheduler/src/notification.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/event.rs`
- Modify if needed: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add wait contracts**

Implement approval wait metadata with task id, approval id, and safe summary.

- [x] **Step 2: Map ask to waiting approval**

Runner stops execution, transitions task, and emits notification.

- [x] **Step 3: Add resume and deny handling**

Approval decisions resume, cancel, or fail tasks without duplicate tool execution.

- [x] **Step 4: Normalize UI events**

Expose task notification frames to CLI/TUI/Desktop consumers.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p ruse-core background_task_approvals
cargo test -p ruse-ui-bridge task_notifications
```

- [x] **Step 2: Run workspace checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M24-03 complete**

Advance focus to M24-04.

- [x] **Step 2: Commit M24-03**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add background task approvals"
```
