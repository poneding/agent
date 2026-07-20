# M61-02 CLI and JSON Plan/Todo Commands Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:executing-plans` or `superpowers:subagent-driven-development` to execute this plan task by task. If unavailable, follow the same checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable text and JSON plan/todo commands over the shared M61-01 contracts.

**Architecture:** `ruse-types` owns plan/todo data, `ruse-core` owns validation and revision-aware updates, `ruse-ui-bridge` owns UI-safe commands/frames, and `ruse-cli` parses/renders without duplicating runtime transition rules.

**Tech Stack:** Rust 2024 edition, serde, existing `UiJsonFrame` newline JSON pattern, Cargo fmt, focused Cargo clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M61-plan-todo-task-ux.spec.md`
- Read: `docs/plans/M61-plan-todo-task-ux.plan.md`
- Read: `docs/specs/M61-01-plan-todo-state.spec.md`
- Read: `docs/plans/M61-01-plan-todo-state.plan.md`
- Read: `docs/specs/M61-02-cli-json-plan-todo.spec.md`
- Read: `crates/ruse-types/src/plan.rs`
- Read: `crates/ruse-core/src/plan.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/task_commands.rs`

- [x] **Step 1: Review command and frame patterns**

Confirm the task command fixture, `UiCommand` serde style, `UiJsonFrame` shape, and shared redaction helpers.

### Task 2: Add Bridge Contracts

**Files:**
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Create: `crates/ruse-ui-bridge/tests/plan_todo_bridge.rs`

- [x] **Step 1: Add `UiPlanTodoCommand`**

Add typed show/list/update/replace commands under `UiCommand::PlanTodo` and validate empty reasons plus nil conversation ids.

- [x] **Step 2: Add `UiPlanTodoSnapshot` frame**

Normalize snapshots through `ruse-core`, sort items, include progress and errors, and expose a `plan_todo_snapshot` JSON frame.

- [x] **Step 3: Test bridge serialization and redaction**

Run:

```bash
cargo test -p ruse-ui-bridge --test plan_todo_bridge
```

Expected: command and frame tests pass, including redaction and empty snapshots.

### Task 3: Add CLI Commands

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/tests/plan_todo_commands.rs`

- [x] **Step 1: Parse plan/todo commands**

Add `plan show`, `todo list`, `todo update`, and `todo set --snapshot-json` parser paths.

- [x] **Step 2: Add fixture runtime and rendering**

Use a deterministic in-memory `PlanTodoCommandRuntime` fixture, apply updates through `ruse-core::apply_plan_todo_update`, and render all JSON through `UiJsonFrame::PlanTodoSnapshot`.

- [x] **Step 3: Test CLI text and JSON behavior**

Run:

```bash
cargo test -p ruse-cli --test plan_todo_commands
```

Expected: parser, text, JSON, update, stale revision, terminal status, and replacement tests pass.

### Task 4: Update Status Docs

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M61-plan-todo-task-ux.plan.md`

- [x] **Step 1: Mark M61-02 complete after verification**

Update M61 indexes and the milestone plan checkboxes only after focused checks pass.

### Task 5: Verify and Commit

**Files:**
- Read: `Cargo.toml`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-ui-bridge -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-ui-bridge --test plan_todo_bridge
cargo test -p ruse-cli --test plan_todo_commands
cargo test -p ruse-cli --test task_commands
git diff --check
```

Expected: all focused checks pass.

- [ ] **Step 2: Commit the slice**

Use exact-path staging for the docs, bridge, CLI, and tests, then commit:

```bash
git commit -m "feat: add m61 cli json plan todo commands"
```

Expected: commit succeeds and the worktree is clean except for unrelated user changes, if any.

### Completion Checklist

- [x] M61-02 spec and plan exist.
- [x] Bridge has typed plan/todo commands and JSON frame.
- [x] CLI text commands can show/list/update/set plan/todo state.
- [x] CLI JSON commands emit stable `plan_todo_snapshot` frames.
- [x] Stale revisions, missing items, terminal status transitions, and invalid snapshots fail safely.
- [x] Plan/todo content, reasons, diagnostics, and errors are bounded and redacted.
- [x] Focused M61-02 checks pass.
