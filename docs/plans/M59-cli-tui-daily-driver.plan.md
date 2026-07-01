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

- [x] **Step 1: Plan slash command/status slice**

Defined a typed slash command catalog, help/status rendering, aliases, argument validation, plain REPL parity, and tests for discoverability in M59-02.

### Task 4: M59-03 Approval Queue and Permission Review

**Files:**
- Create: `docs/specs/M59-03-approval-queue-permission-review.spec.md`
- Create: `docs/plans/M59-03-approval-queue-permission-review.plan.md`

- [x] **Step 1: Plan approval queue slice**

Defined queue grouping, focus movement, batch-safe decisions, lease display, preview redaction, CLI/JSON inspection, and TUI actions in M59-03.

- [x] **Step 2: Implement approval queue slice**

Completed typed approval queue snapshots, CLI text/JSON inspection, TUI queue metadata rendering, focus-preserving approval actions, and focused/standard verification.

### Task 5: M59-04 Diff Viewer and Command Streaming Controls

**Files:**
- Create: `docs/specs/M59-04-diff-command-streaming.spec.md`
- Create: `docs/plans/M59-04-diff-command-streaming.plan.md`

- [x] **Step 1: Plan diff and command panel slice**

Defined typed diff and command/test daily-driver rendering, cancellation readiness, TUI navigation/state, CLI/JSON stability, and focused verification in M59-04.

- [x] **Step 2: Implement diff and command panel slice**

Completed CLI text/JSON diff and command evidence, bridge snapshot regression coverage, TUI diff navigation, TUI command output controls, cancellation readiness rendering, and focused/standard verification.

### Task 6: M59-05 Task Status and JSON/Batch Recipes

**Files:**
- Create: `docs/specs/M59-05-task-status-json-recipes.spec.md`
- Create: `docs/plans/M59-05-task-status-json-recipes.plan.md`
- Create: `docs/cli/daily-driver-recipes.md`

- [x] **Step 1: Plan task status and recipe slice**

Defined task status polish, CLI/JSON recipe tests, deterministic docs, and final M59 verification/closeout gates in M59-05.

- [x] **Step 2: Implement task status and recipe slice**

Completed task status panel polish, bridge/CLI task coverage, deterministic daily-driver recipes, recipe parseability tests, and final M59 verification.

### Task 7: Final Verification and Closeout

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`

- [x] **Step 1: Run final M59 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all gates pass with terminal daily-driver surfaces redacted and deterministic.

- [x] **Step 2: Commit M59 closeout**

Run exact-path staging and commit the final docs/status updates.

### Completion Checklist

- [x] Terminal sessions can be listed, restored, and switched through typed contracts.
- [x] Slash command catalog is discoverable and tested.
- [x] Approval queue and permission review workflows are actionable and safe.
- [x] Diff and command/test panels support repeated coding-loop use.
- [x] Task status is visible and bounded in CLI/JSON/TUI paths.
- [x] JSON/batch recipes are documented and covered by tests.
- [x] Final workspace fmt/clippy/tests and `cargo xtask eval` pass.
