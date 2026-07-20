# M61-04 Background Wait Controls and Notifications Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:executing-plans` to execute this plan task by task. If unavailable, follow the same checklist directly.

## Goal

Thread scheduler-owned wait/control evidence through task summaries so CLI, JSON, and Desktop can show background approval ids, latest notification metadata, and pause/resume/cancel results.

## Files

- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M61-plan-todo-task-ux.plan.md`
- Modify: `crates/ruse-scheduler/src/notification.rs`
- Modify: `crates/ruse-scheduler/src/task.rs`
- Modify: `crates/ruse-scheduler/src/queue.rs`
- Modify: `crates/ruse-scheduler/src/command.rs`
- Modify: `crates/ruse-scheduler/src/lib.rs`
- Modify: `crates/ruse-scheduler/tests/task_command_bridge.rs`
- Modify: `crates/ruse-core/tests/background_task_approvals.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/tests/task_command_bridge.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/tests/task_commands.rs`
- Modify: `crates/ruse-desktop/src/task.rs`
- Modify: `crates/ruse-desktop/tests/conversation_tasks.rs`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`

## Implementation Steps

- [x] Add `TaskNotificationSummary` and additive latest-notification fields to scheduler task contracts.
- [x] Persist `waiting_approval_id` and latest notification summaries during queue wait/resume/cancel/complete/fail transitions.
- [x] Add command-service control evidence for create, pause, resume, cancel, and schedule update responses.
- [x] Map scheduler evidence into `UiTaskSummary` as defaulted additive fields.
- [x] Render evidence in CLI text and preserve it in task JSON snapshots.
- [x] Carry evidence through Desktop Rust models and TypeScript task detail rendering.
- [x] Update M61 tracking docs and focused tests.

## Tests and Commands

Run focused checks:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-scheduler -p ruse-core -p ruse-ui-bridge -p ruse-cli -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-scheduler
cargo test -p ruse-core background
cargo test -p ruse-ui-bridge --test task_command_bridge
cargo test -p ruse-cli --test task_commands
cargo test -p ruse-desktop --test conversation_tasks
git diff --check
```

## Completion Checklist

- [x] Scheduler contracts are serde-friendly and backward-compatible.
- [x] Approval wait ids are retained on task state and exposed through UI summaries.
- [x] Latest notification/control evidence is bounded, redacted, and typed.
- [x] CLI text and JSON output include wait/control evidence.
- [x] Desktop task monitor/detail models include wait/control evidence.
- [x] Focused checks pass.
- [x] Exact files are staged and committed.
