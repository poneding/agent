# M59-05 Task Status and JSON/Batch Recipes Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish M59 by polishing task status surfaces and documenting/test-covering stable terminal daily-driver JSON and batch recipes.

**Architecture:** `agent-scheduler` and `agent-core` own task state and review semantics. `agent-ui-bridge` owns UI-safe task/review snapshots. `agent-cli` owns task command rendering and recipe smoke tests. `agent-tui` owns task panel state/rendering. Recipe docs use deterministic fixture/local commands and avoid provider credentials.

**Tech Stack:** Rust 2024 edition, serde task snapshots, existing scheduler task command service, Ratatui task panel state, Markdown recipe docs, Cargo fmt/clippy/test, `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M59-05-task-status-json-recipes.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-ui-bridge/tests/task_command_bridge.rs`
- Read: `crates/agent-ui-bridge/tests/task_review.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/task_commands.rs`
- Read: `crates/agent-tui/src/task_panel.rs`
- Read: `crates/agent-tui/tests/task_panel.rs`

- [x] **Step 1: Review current task and recipe baseline**

Run:

```bash
cargo test -p agent-ui-bridge --test task_command_bridge
cargo test -p agent-ui-bridge --test task_review
cargo test -p agent-cli --test task_commands
cargo test -p agent-tui --test task_panel
```

Expected: existing task snapshots, CLI task commands, task reviews, and TUI task panel pass before M59-05 changes.

### Task 2: Improve Task Snapshot and Review Coverage

**Files:**
- Modify: `crates/agent-ui-bridge/tests/task_command_bridge.rs`
- Modify if needed: `crates/agent-ui-bridge/tests/task_review.rs`
- Modify if needed: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Add task snapshot expectations**

Cover:

- empty task snapshot succeeds
- queued/running/waiting-approval/paused/succeeded/failed/cancelled states serialize safely where representable by fixtures
- prompt summaries and errors are bounded and redacted
- approval ids are preserved when present
- task JSON frame shape is stable

- [x] **Step 2: Add task review expectations**

Cover final state, trace id, checkpoint id, artifact count, missing artifacts, diagnostics, redaction, and JSON frame shape.

- [x] **Step 3: Implement bridge adjustments only if needed**

Keep existing task contracts if they already satisfy the spec; otherwise add serde-friendly, bounded, redacted metadata with defaults where compatibility requires.

### Task 3: Improve CLI Task Status and Recipes Tests

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/tests/task_commands.rs`
- Create: `crates/agent-cli/tests/daily_driver_recipes.rs`

- [x] **Step 1: Add CLI task output expectations**

Cover:

- task list empty state and non-empty state
- create/list/inspect/pause/resume/cancel text output with state, priority, schedule, and redacted prompt/reason
- JSON task create/list/inspect/error snapshots remain parseable and bounded
- task review rendering includes final state, artifacts, trace/checkpoint ids, diagnostics, and redaction

- [x] **Step 2: Add recipe tests**

Cover documented command lines for:

- `repo session bootstrap`
- slash/status or startup/config checks
- `approval queue --fixture sample`
- `repo edit --preview` against a temp workspace
- `repo command` or `repo test` against a deterministic helper
- `task create/list/inspect`
- `eval suite run` or `cargo xtask eval` reference as the full smoke gate

Recipe tests may parse commands, invoke renderer fixtures, or run small temp-workspace commands; they must not require provider credentials or network access.

- [x] **Step 3: Implement CLI rendering changes**

Improve text output only where tests show daily-driver gaps. Keep JSON status codes and error shapes stable.

### Task 4: Improve TUI Task Panel

**Files:**
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify: `crates/agent-tui/tests/task_panel.rs`

- [x] **Step 1: Add task panel tests**

Cover task snapshot reduction, grouped state/count metadata if added, waiting-approval approval id rendering, review rendering, notification bounds, empty state, and redaction.

- [x] **Step 2: Implement task panel updates**

Render task states, prompt summaries, priority/schedule, approval ids, review artifacts, trace/checkpoint ids, diagnostics, and notification history in a bounded deterministic layout.

- [x] **Step 3: Run TUI task tests**

Run:

```bash
cargo test -p agent-tui --test task_panel
```

Expected: TUI task panel tests pass.

### Task 5: Add Daily-Driver Recipe Docs

**Files:**
- Create: `docs/cli/daily-driver-recipes.md`
- Modify if needed: `README.md`
- Modify if needed: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [x] **Step 1: Write recipes**

Document credential-free command sequences for session/status, approval queue, edit preview, command/test evidence, task lifecycle, JSON forms, and final eval.

- [x] **Step 2: Keep examples stable and redacted**

Use placeholders and temp-workspace-safe examples. Avoid raw secrets, live provider calls, and destructive operations outside temp/example workspaces.

### Task 6: Verify, Close M59, and Commit

**Files:**
- Modify: `docs/plans/M59-05-task-status-json-recipes.plan.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify if needed: `README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli -p agent-ui-bridge -p agent-tui --all-targets -- -D warnings
cargo test -p agent-ui-bridge --test task_command_bridge
cargo test -p agent-ui-bridge --test task_review
cargo test -p agent-cli --test task_commands
cargo test -p agent-cli --test daily_driver_recipes
cargo test -p agent-tui --test task_panel
```

Expected: focused checks pass.

- [x] **Step 2: Run final M59 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: workspace checks and default eval pass.

- [x] **Step 3: Commit M59-05 implementation**

Run exact-path staging for M59-05 source, tests, and docs, then:

```bash
git commit -m "feat: add m59 task status recipes"
```

Expected: commit contains only M59-05 source, tests, recipe docs, and documentation/status updates.

- [x] **Step 4: Commit M59 closeout if needed**

If final milestone status updates are separate from the implementation commit, exact-stage docs and commit:

```bash
git commit -m "docs: close m59 daily driver"
```

Expected: M59 status is complete and next planning direction points to M60.

### Completion Checklist

- [x] Task status surfaces expose stable state, priority, schedule, prompt summary, approval, and review evidence.
- [x] CLI task text/JSON output is stable, bounded, and redacted.
- [x] TUI task panel renders snapshots, reviews, notifications, approval waits, and empty state clearly.
- [x] Daily-driver recipes are documented and covered by deterministic tests.
- [x] M59 final workspace fmt/clippy/tests and `cargo xtask eval` pass.
- [x] M59 status docs are updated for closeout and next milestone direction.
