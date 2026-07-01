# M61-03 TUI and Desktop Plan/Task Surfaces Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:executing-plans` or `superpowers:subagent-driven-development` to execute this plan task by task. If unavailable, follow the same checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render shared plan/todo/task state in TUI and Desktop surfaces from typed bridge snapshots.

**Architecture:** `agent-ui-bridge` owns UI-safe snapshot contracts, `agent-tui` and `agent-desktop` render/shape presentation state, and `apps/desktop` reduces/render frames. Runtime transition validation stays in `agent-core`.

**Tech Stack:** Rust 2024, serde, existing TUI task panel state, Desktop Rust view models, React/TypeScript/Vitest, Cargo fmt/clippy/test, npm test/build.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M61-plan-todo-task-ux.spec.md`
- Read: `docs/plans/M61-plan-todo-task-ux.plan.md`
- Read: `docs/specs/M61-02-cli-json-plan-todo.spec.md`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-tui/src/task_panel.rs`
- Read: `crates/agent-tui/tests/task_panel.rs`
- Read: `crates/agent-desktop/src/task.rs`
- Read: `crates/agent-desktop/tests/conversation_tasks.rs`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/state/conversation.ts`
- Read: `apps/desktop/src/state/tasks.ts`
- Read: `apps/desktop/src/components/TaskList.tsx`
- Read: `apps/desktop/src/components/TaskDetail.tsx`
- Read: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`
- Read: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`
- Read: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Review current task surfaces**

Confirm current task, review, notification, and plan/todo frame shapes and existing redaction/bounding behavior.

### Task 2: Add Bridge Presentation Fields

**Files:**
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify: `crates/agent-ui-bridge/tests/task_command_bridge.rs`

- [x] **Step 1: Add optional task plan fields**

Add serde-defaulted `plan_progress`, `active_todo`, `waiting_approval`, `latest_notification`, and `artifact_count` fields to `UiTaskSummary`.

- [x] **Step 2: Test backward-compatible task snapshots**

Existing task fixtures without new fields must still deserialize and render.

### Task 3: Implement TUI Plan Rendering

**Files:**
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify: `crates/agent-tui/tests/task_panel.rs`

- [x] **Step 1: Store latest plan snapshot in task panel**

Reduce `UiJsonFrame::PlanTodoSnapshot` into task panel state.

- [x] **Step 2: Render plan progress and active todo**

Show revision, counts, active todo content, diagnostics/errors, task statuses, reviews, and notifications.

- [x] **Step 3: Verify TUI panel behavior**

Run:

```bash
cargo test -p agent-tui --test task_panel
```

Expected: task panel renders plan progress and remains compatible with task-only frames.

### Task 4: Implement Desktop Rust View Models

**Files:**
- Modify: `crates/agent-desktop/src/task.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Create: `crates/agent-desktop/tests/plan_todo_tasks.rs`

- [x] **Step 1: Add typed Desktop plan fields**

Add Desktop task monitor fields for plan progress, active todo, waiting approval, latest notification, and artifact count.

- [x] **Step 2: Verify Rust Desktop conversion**

Run:

```bash
cargo test -p agent-desktop --test plan_todo_tasks
```

Expected: Desktop Rust view models serialize safe plan/task details without secrets.

### Task 5: Implement Desktop TypeScript State and UI

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/components/TaskList.tsx`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Add TS plan/task fields**

Define plan progress and active todo types, reduce `plan_todo_snapshot` frames into matching tasks, and preserve task-only fixtures.

- [x] **Step 2: Render plan details**

Show progress counts, active todo, waiting state, latest notification, and artifact count in task list/detail.

- [x] **Step 3: Verify frontend behavior**

Run:

```bash
npm --prefix apps/desktop test -- conversation-task-ui trace-settings-ui desktop-smoke
npm --prefix apps/desktop run build
```

Expected: Desktop task UI renders plan/todo/task state and remains redacted.

### Task 6: Status Docs, Verification, and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M61-plan-todo-task-ux.plan.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-tui -p agent-desktop --all-targets -- -D warnings
cargo test -p agent-tui --test task_panel
cargo test -p agent-desktop --test plan_todo_tasks
npm --prefix apps/desktop test -- conversation-task-ui trace-settings-ui desktop-smoke
npm --prefix apps/desktop run build
git diff --check
```

Expected: all focused checks pass.

- [ ] **Step 2: Mark M61-03 complete and commit**

Use exact-path staging and commit:

```bash
git commit -m "feat: add m61 tui desktop plan task surfaces"
```

Expected: commit succeeds and the worktree is clean except unrelated user changes, if any.

### Completion Checklist

- [x] M61-03 spec and plan exist.
- [x] TUI task panel renders plan progress and active todo from bridge frames.
- [x] Desktop Rust task view models include typed plan progress, active todo, waiting, notification, and artifact fields.
- [x] Desktop TypeScript reducers and React views render plan/task state from typed data.
- [x] Existing task-only fixtures remain backward-compatible.
- [x] Focused Rust/frontend checks pass.
