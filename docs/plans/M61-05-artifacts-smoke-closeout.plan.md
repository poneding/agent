# M61-05 Artifacts, Smoke, Docs, and Closeout Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:executing-plans` to execute this plan task by task. If unavailable, follow this checklist directly.

## Goal

Complete M61 by wiring task review/artifact evidence into plan/todo state, updating smoke docs, and running final milestone gates.

## Files

- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M61-plan-todo-task-ux.plan.md`
- Modify: `crates/ruse-core/src/plan.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify: `crates/ruse-core/tests/plan_todo_state.rs`
- Modify: `crates/ruse-ui-bridge/tests/task_review.rs`
- Modify: `crates/ruse-cli/tests/daily_driver_recipes.rs`
- Modify: `crates/ruse-desktop/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Modify: `docs/cli/daily-driver-recipes.md`
- Create: `docs/desktop/plan-todo-task-smoke.md`

## Implementation Steps

- [x] Add a core helper to append task review, artifact, trace, checkpoint, and diagnostic evidence to a todo item.
- [x] Test revision checks, redaction, missing artifact diagnostics, and terminal todo preservation.
- [x] Extend CLI daily-driver docs with plan/todo/task evidence recipes and keep recipe commands parseable.
- [x] Extend Desktop Rust and Vitest smoke fixtures with M61 task review/artifact evidence.
- [x] Add Desktop plan/todo/task smoke documentation.
- [x] Update roadmap, milestone, and M61 plan closeout status after verification.

## Tests and Commands

Focused checks:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-core -p ruse-ui-bridge -p ruse-cli -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-core --test plan_todo_state
cargo test -p ruse-ui-bridge --test task_review
cargo test -p ruse-cli --test daily_driver_recipes
cargo test -p ruse-desktop --test desktop_daily_driver
npm --prefix apps/desktop test -- desktop-smoke conversation-task-ui
npm --prefix apps/desktop run build
git diff --check
```

Final M61 gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

## Completion Checklist

- [x] Review/artifact evidence links into plan/todo snapshots through core runtime logic.
- [x] CLI and Desktop smoke docs are updated.
- [x] Deterministic smoke coverage proves M61 plan/todo/task evidence without credentials.
- [x] Final M61 gates pass.
- [x] M61 roadmap, milestone, and plan status are marked complete.
- [x] Exact files are staged and committed.

## Verification Evidence

Focused checks passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p ruse-core -p ruse-ui-bridge -p ruse-cli -p ruse-desktop --all-targets -- -D warnings`
- `cargo test -p ruse-core --test plan_todo_state`
- `cargo test -p ruse-ui-bridge --test task_review`
- `cargo test -p ruse-cli --test daily_driver_recipes`
- `cargo test -p ruse-desktop --test desktop_daily_driver`
- `npm --prefix apps/desktop test -- desktop-smoke conversation-task-ui`
- `npm --prefix apps/desktop run build`

Final M61 gates passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix apps/desktop test`
- `npm --prefix apps/desktop run build`
- `cargo xtask eval`
