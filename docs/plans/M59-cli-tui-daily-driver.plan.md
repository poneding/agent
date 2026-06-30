# M59 CLI/TUI Daily Driver Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn terminal usage into a daily-driver workflow across CLI, plain REPL, JSON, and TUI without moving runtime logic into presentation crates.

**Architecture:** `agent-core` owns terminal workflow/session orchestration, `agent-storage` owns durable event access, `agent-ui-bridge` owns UI-safe command/snapshot contracts, `agent-cli` owns command parsing and text/JSON rendering, and `agent-tui` owns panel state and rendering.

**Tech Stack:** Rust 2024 edition, serde contracts, existing runtime events, repo session reports, command/test reports, approval/task/diff bridge snapshots, Ratatui TUI state, Cargo fmt/clippy/test, `cargo xtask eval`.

---

### Task 1: Confirm M59 Inputs

**Files:**
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/specs/M20-terminal-ux-tui-repl.spec.md`
- Read: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/src/repl.rs`
- Read: `crates/agent-tui/src/app.rs`
- Read: `crates/agent-tui/src/slash.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Review terminal baseline**

Run:

```bash
cargo test -p agent-tui
cargo test -p agent-cli --test repo_session_bootstrap
cargo test -p agent-ui-bridge repo_session_snapshot
cargo xtask eval
```

Expected: current terminal state, repo session CLI, bridge repo session snapshots, and default eval suite pass before M59 changes.

Baseline verification on 2026-07-01:

- `cargo test -p agent-tui`
- `cargo test -p agent-cli --test repo_session_bootstrap`
- `cargo test -p agent-ui-bridge repo_session_snapshot`
- `cargo xtask eval`

### Task 2: M59-01 Session Restore and Switch

**Files:**
- Execute: `docs/specs/M59-01-session-restore-switch.spec.md`
- Execute: `docs/plans/M59-01-session-restore-switch.plan.md`

- [x] **Step 1: Complete terminal session directory slice**

Added typed session summaries, restore/switch validation, CLI/JSON commands, bridge snapshot support, and TUI session panel state in M59-01.

### Task 3: M59-02 Slash Command Catalog and Status Hub

**Files:**
- Create: `docs/specs/M59-02-slash-command-status-hub.spec.md`
- Create: `docs/plans/M59-02-slash-command-status-hub.plan.md`

- [ ] **Step 1: Plan slash command/status slice**

Define a typed slash command catalog, help/status rendering, aliases, argument validation, plain REPL parity, and tests for discoverability.

### Task 4: M59-03 Approval Queue and Permission Review

**Files:**
- Create: `docs/specs/M59-03-approval-queue-permission-review.spec.md`
- Create: `docs/plans/M59-03-approval-queue-permission-review.plan.md`

- [ ] **Step 1: Plan approval queue slice**

Define queue grouping, focus movement, batch-safe decisions, lease display, preview redaction, CLI/JSON inspection, and TUI actions.

### Task 5: M59-04 Diff Viewer and Command Streaming Controls

**Files:**
- Create: `docs/specs/M59-04-diff-command-streaming.spec.md`
- Create: `docs/plans/M59-04-diff-command-streaming.plan.md`

- [ ] **Step 1: Plan diff and command panel slice**

Define diff navigation, command/test output chunks, waiting/running/completed states, cancellation readiness, and stable rendering across CLI/JSON/TUI.

### Task 6: M59-05 Task Status and JSON/Batch Recipes

**Files:**
- Create: `docs/specs/M59-05-task-status-json-recipes.spec.md`
- Create: `docs/plans/M59-05-task-status-json-recipes.plan.md`
- Create: `docs/cli/daily-driver-recipes.md`

- [ ] **Step 1: Plan task status and recipe slice**

Define task panel improvements, CLI/JSON recipe commands, documented smoke scripts, and deterministic tests for batch-safe daily-driver workflows.

### Task 7: Final Verification and Closeout

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`

- [ ] **Step 1: Run final M59 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all gates pass with terminal daily-driver surfaces redacted and deterministic.

- [ ] **Step 2: Commit M59 closeout**

Run exact-path staging and commit the final docs/status updates.

### Completion Checklist

- [x] Terminal sessions can be listed, restored, and switched through typed contracts.
- [ ] Slash command catalog is discoverable and tested.
- [ ] Approval queue and permission review workflows are actionable and safe.
- [ ] Diff and command/test panels support repeated coding-loop use.
- [ ] Task status is visible and bounded in CLI/JSON/TUI paths.
- [ ] JSON/batch recipes are documented and covered by tests.
- [ ] Final workspace fmt/clippy/tests and `cargo xtask eval` pass.
