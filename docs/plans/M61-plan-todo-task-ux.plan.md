# M61 Plan, Todo, and Task UX Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make plans, todos, and tasks one shared product surface across CLI/JSON, TUI, Desktop, and background work.

**Architecture:** `agent-types` owns durable serde contracts, `agent-core` owns plan/todo validation and progress state, `agent-scheduler` owns background task lifecycle and notifications, `agent-ui-bridge` owns UI-safe commands/frames, and CLI/TUI/Desktop render or submit typed contracts.

**Tech Stack:** Rust 2024 edition, serde contracts, existing scheduler/task commands, UI bridge frames, Ratatui task panel state, Desktop Rust view models, React/TypeScript/Vitest, Cargo fmt/clippy/test, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, and `cargo xtask eval`.

---

### Task 1: Confirm M61 Inputs

**Files:**
- Read: `docs/specs/M61-plan-todo-task-ux.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/specs/M08-scheduler-background-tasks.spec.md`
- Read: `docs/specs/M24-background-agents-ux.spec.md`
- Read: `docs/specs/M59-05-task-status-json-recipes.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-types/src/state.rs`
- Read: `crates/agent-scheduler/src/task.rs`
- Read: `crates/agent-scheduler/src/command.rs`
- Read: `crates/agent-scheduler/src/notification.rs`
- Read: `crates/agent-core/src/background.rs`
- Read: `crates/agent-core/src/task_review.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-tui/src/task_panel.rs`
- Read: `crates/agent-desktop/src/task.rs`
- Read: `apps/desktop/src/components/TaskList.tsx`
- Read: `apps/desktop/src/components/TaskDetail.tsx`

- [x] **Step 1: Review existing task and background foundations**

Baseline reviewed during M61 planning:

- scheduler task state, queue, commands, approval waits, pause/resume/cancel, and notifications
- background runner approval wait and notification behavior
- task review artifact contracts
- UI bridge task snapshots, review snapshots, and task notification events
- CLI task lifecycle rendering and JSON frames
- TUI task panel state
- Desktop task monitor and React task components

- [ ] **Step 2: Run baseline focused checks before implementation**

Run:

```bash
cargo test -p agent-scheduler
cargo test -p agent-core background
cargo test -p agent-core task_review
cargo test -p agent-ui-bridge --test task_command_bridge
cargo test -p agent-ui-bridge --test task_review
cargo test -p agent-cli --test task_commands
cargo test -p agent-tui --test task_panel
cargo test -p agent-desktop --test conversation_tasks
npm --prefix apps/desktop test -- desktop-smoke
```

Expected: existing task, review, bridge, CLI, TUI, Desktop, and frontend task surfaces pass before M61 implementation changes.

### Task 2: M61-01 Shared Plan/Todo State Contracts

**Files:**
- Create: `docs/specs/M61-01-plan-todo-state.spec.md`
- Create: `docs/plans/M61-01-plan-todo-state.plan.md`
- Modify: `crates/agent-types/src/lib.rs`
- Modify: `crates/agent-types/src/event.rs`
- Create: `crates/agent-types/src/plan.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/src/plan.rs`
- Create: `crates/agent-core/tests/plan_todo_state.rs`

- [x] **Step 1: Plan shared state slice**

Define TodoWrite-compatible state, progress counts, revision checks, evidence links, event payloads, and redaction/validation rules.

M61-01 planning is captured in:

- `docs/specs/M61-01-plan-todo-state.spec.md`
- `docs/plans/M61-01-plan-todo-state.plan.md`

- [x] **Step 2: Implement typed plan/todo contracts**

Add serde-friendly plan/todo ids, states, item snapshots, progress summaries, diagnostics, evidence references, and runtime validation helpers.

- Implemented `agent-types::plan` contracts for todo ids, statuses, priorities, sources, evidence, diagnostics, snapshots, progress, and plan/todo runtime event payloads.
- Implemented `agent-core::plan` validation, normalization, progress, revision checks, update application, and evidence append helpers.
- Added minimal UI bridge normalization for the new runtime event payloads so workspace consumers remain exhaustive; richer plan/todo bridge snapshots remain scoped to M61-02.

- [x] **Step 3: Verify shared state contracts**

Run focused tests:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-types -p agent-core --all-targets -- -D warnings
cargo test -p agent-types
cargo test -p agent-core --test plan_todo_state
```

Expected: shared plan/todo contracts serialize, validate, redact, and reject invalid/stale updates.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p agent-types -p agent-core --all-targets -- -D warnings`
- `cargo test -p agent-types`
- `cargo test -p agent-core --test plan_todo_state`
- `cargo test -p agent-core --test task_artifacts_review`
- `cargo test -p agent-ui-bridge --test m61_plan_todo_events`
- `cargo clippy -p agent-ui-bridge --all-targets -- -D warnings`

### Task 3: M61-02 CLI and JSON Plan/Todo Commands

**Files:**
- Create: `docs/specs/M61-02-cli-json-plan-todo.spec.md`
- Create: `docs/plans/M61-02-cli-json-plan-todo.plan.md`
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Create: `crates/agent-ui-bridge/tests/plan_todo_bridge.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Create: `crates/agent-cli/tests/plan_todo_commands.rs`

- [x] **Step 1: Plan CLI/JSON slice**

Define plan/todo command syntax, bridge command envelopes, JSON frame shapes, and stable text rendering.

- [x] **Step 2: Implement CLI/JSON commands**

Expose `plan show`, `todo list`, and typed todo update/set commands through shared bridge/runtime contracts.

- [x] **Step 3: Verify CLI/JSON behavior**

Run:

```bash
cargo clippy -p agent-ui-bridge -p agent-cli --all-targets -- -D warnings
cargo test -p agent-ui-bridge --test plan_todo_bridge
cargo test -p agent-cli --test plan_todo_commands
```

Expected: text and JSON plan/todo commands are stable, bounded, redacted, and revision-aware.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p agent-ui-bridge -p agent-cli --all-targets -- -D warnings`
- `cargo test -p agent-ui-bridge --test plan_todo_bridge`
- `cargo test -p agent-cli --test plan_todo_commands`
- `cargo test -p agent-cli --test task_commands`
- `cargo test -p agent-tui --test trace_panel`
- `cargo test -p agent-tui --test transcript`

### Task 4: M61-03 TUI and Desktop Plan/Task Surfaces

**Files:**
- Create: `docs/specs/M61-03-tui-desktop-plan-task-surfaces.spec.md`
- Create: `docs/plans/M61-03-tui-desktop-plan-task-surfaces.plan.md`
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify: `crates/agent-tui/tests/task_panel.rs`
- Modify: `crates/agent-desktop/src/task.rs`
- Modify: `crates/agent-desktop/src/conversation.rs`
- Create: `crates/agent-desktop/tests/plan_todo_tasks.rs`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/components/TaskList.tsx`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Plan UI surface slice**

Define how TUI and Desktop render plan progress, active todo, waiting approval, latest notification, and artifact evidence from bridge snapshots.

- [x] **Step 2: Implement shared UI rendering**

Update TUI panel, Desktop view models, React task list/detail state, and fixture tests without moving runtime state into presentation layers.

- [x] **Step 3: Verify UI surfaces**

Run:

```bash
cargo clippy -p agent-tui -p agent-desktop --all-targets -- -D warnings
cargo test -p agent-tui --test task_panel
cargo test -p agent-desktop --test plan_todo_tasks
npm --prefix apps/desktop test -- desktop-smoke
npm --prefix apps/desktop run build
```

Expected: TUI and Desktop show plan/todo/task truth from typed snapshots.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p agent-tui -p agent-desktop --all-targets -- -D warnings`
- `cargo test -p agent-tui --test task_panel`
- `cargo test -p agent-desktop --test plan_todo_tasks`
- `cargo test -p agent-desktop --test conversation_tasks`
- `cargo test -p agent-ui-bridge --test task_command_bridge`
- `npm --prefix apps/desktop test -- conversation-task-ui trace-settings-ui desktop-smoke`
- `npm --prefix apps/desktop run build`

### Task 5: M61-04 Background Wait Controls and Notifications

**Files:**
- Create: `docs/specs/M61-04-background-waits-controls-notifications.spec.md`
- Create: `docs/plans/M61-04-background-waits-controls-notifications.plan.md`
- Modify: `crates/agent-scheduler/src/command.rs`
- Modify: `crates/agent-scheduler/src/notification.rs`
- Modify: `crates/agent-core/src/background.rs`
- Modify: `crates/agent-ui-bridge/src/event.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-cli/tests/task_commands.rs`
- Modify: `crates/agent-desktop/tests/conversation_tasks.rs`

- [ ] **Step 1: Plan wait/control slice**

Define the evidence linking approval waits, pause/resume/cancel, task notifications, and plan/todo progress.

- [ ] **Step 2: Implement wait/control evidence**

Thread approval ids, latest notifications, and control results through existing scheduler/core/bridge surfaces.

- [ ] **Step 3: Verify wait/control behavior**

Run:

```bash
cargo clippy -p agent-scheduler -p agent-core -p agent-ui-bridge -p agent-cli -p agent-desktop --all-targets -- -D warnings
cargo test -p agent-scheduler
cargo test -p agent-core background
cargo test -p agent-ui-bridge --test task_command_bridge
cargo test -p agent-cli --test task_commands
cargo test -p agent-desktop --test conversation_tasks
```

Expected: background waits and controls are visible, typed, redacted, and still scheduler/policy backed.

### Task 6: M61-05 Artifacts, Smoke, Docs, and Closeout

**Files:**
- Create: `docs/specs/M61-05-artifacts-smoke-closeout.spec.md`
- Create: `docs/plans/M61-05-artifacts-smoke-closeout.plan.md`
- Modify: `crates/agent-core/src/task_review.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-cli/tests/daily_driver_recipes.rs`
- Modify: `crates/agent-desktop/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Create or modify: `docs/cli/daily-driver-recipes.md`
- Create: `docs/desktop/plan-todo-task-smoke.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Plan smoke and closeout slice**

Define the final credential-free acceptance path for CLI/JSON and Desktop plan/todo/task UX.

- [ ] **Step 2: Implement artifact evidence and docs**

Link task reviews/artifacts into plan/todo evidence and document the product workflow.

- [ ] **Step 3: Run final M61 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: workspace, Desktop, and eval gates pass before M61 status is marked complete.

- [ ] **Step 4: Commit M61 closeout**

Use exact-path staging for M61 source, tests, docs, and status updates, then commit.

### Completion Checklist

- [ ] M61 milestone spec/plan and slice specs/plans are complete.
- [ ] TodoWrite-compatible state is typed, serializable, validated, bounded, and redacted.
- [ ] CLI/JSON can view and update plan/todo state.
- [ ] TUI and Desktop render plan progress, active todo, approval waits, notifications, and artifact evidence from shared snapshots.
- [ ] Background pause/resume/cancel and approval waits remain scheduler/policy backed.
- [ ] Deterministic smoke coverage proves the product workflow without live provider credentials.
- [ ] Final M61 fmt, clippy, workspace tests, frontend tests/build, and eval gates pass.
