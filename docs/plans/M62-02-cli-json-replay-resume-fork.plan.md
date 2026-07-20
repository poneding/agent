# M62-02 CLI/JSON Replay, Resume, and Fork Commands Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose M62-01 recovery snapshots and dry-run reports through stable UI bridge contracts and CLI text/JSON commands for recovery status, resume dry-run, replay inspect, and replay fork planning.

**Architecture:** `ruse-core` remains recovery truth. `ruse-ui-bridge` owns typed UI command/frame contracts. `ruse-cli` parses and renders typed reports without duplicating recovery logic. This slice is read-only and dry-run oriented.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `RecoverySnapshot` and `ResumeDryRunReport`, UI bridge commands/frames, CLI parser/rendering patterns, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62-02-cli-json-replay-resume-fork.spec.md`
- Read: `docs/specs/M62-01-recovery-snapshot-resume.spec.md`
- Read: `docs/plans/M62-01-recovery-snapshot-resume.plan.md`
- Read: `crates/ruse-types/src/recovery.rs`
- Read: `crates/ruse-core/src/recovery.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/plan_todo_commands.rs`
- Read: `crates/ruse-cli/tests/json_mode.rs`

- [x] **Step 1: Review CLI and bridge patterns**

Identify existing command parsing, text rendering, JSON frame rendering, fixture helpers, and stable serde test style.

### Task 2: Add Bridge Contracts

**Files:**
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Create: `crates/ruse-ui-bridge/tests/recovery_bridge.rs`

- [x] **Step 1: Add UI command variants**

Add typed command variants for:

- `RecoveryStatus`
- `RecoveryResumeDryRun`
- `ReplayInspect`
- `ReplayFork`

Use existing ID types and snake_case serde tagging.

- [x] **Step 2: Add UI recovery frame/report types**

Add typed report/frame wrappers for:

- recovery snapshot
- resume dry-run report
- replay inspect report
- replay fork report

Wrap existing `ruse-types` recovery contracts where possible.

- [x] **Step 3: Add bridge serde tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test recovery_bridge
```

Expected initially: tests fail before bridge contracts exist, then pass after implementation.

### Task 3: Add CLI Commands

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/src/lib.rs`
- Modify: `crates/ruse-tui/src/trace_panel.rs`
- Modify: `crates/ruse-tui/src/transcript.rs`
- Create: `crates/ruse-cli/tests/recovery_commands.rs`

- [x] **Step 1: Add parser coverage**

Support command forms:

```text
ruse-cli recovery status --conversation <id> [--turn <id>]
ruse-cli recovery resume --conversation <id> [--turn <id>] [--checkpoint <id>] --dry-run
ruse-cli replay inspect --conversation <id> [--from-checkpoint <id>]
ruse-cli replay fork --conversation <id> [--turn <id>] [--checkpoint <id>] --label <name> --dry-run
```

- [x] **Step 2: Add deterministic fixture-backed reports**

If the CLI does not yet have arbitrary persisted session lookup by conversation id, use deterministic fixture helpers for M62-02 tests while preserving the real command shape and returning typed diagnostics when live lookup is unavailable.

- [x] **Step 3: Render stable text and JSON**

Text output should include readiness, replay_from, replayed_event_count, pending tool count, idempotent effect count, pending approval count, and diagnostics. JSON output should emit typed data frames suitable for automation.

- [x] **Step 4: Add CLI tests**

Run:

```bash
cargo test -p ruse-cli --test recovery_commands
```

Expected: text and JSON commands are stable, bounded, redacted, and reject invalid arguments safely.

### Task 4: Verification and Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md` if status text needs refreshing
- Modify: `docs/plans/M62-02-cli-json-replay-resume-fork.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-ui-bridge -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-ui-bridge --test recovery_bridge
cargo test -p ruse-cli --test recovery_commands
```

Expected: all focused checks pass.

- [x] **Step 2: Update status docs**

Mark M62-02 complete only after command contracts, CLI output, tests, and checks pass.

- [x] **Step 3: Commit planning docs first**

Before implementation, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-02-cli-json-replay-resume-fork.spec.md docs/plans/M62-02-cli-json-replay-resume-fork.plan.md docs/milestones/README.md
git commit -m "docs: add m62-02 recovery cli plan"
```

Expected: M62-02 slice planning is committed before code changes.

- [x] **Step 4: Commit implementation after verification**

After focused verification passes, exact-stage only M62-02 files and commit:

```bash
git status --short --branch
git diff --check
git add crates/ruse-ui-bridge/src/command.rs crates/ruse-ui-bridge/src/frame.rs crates/ruse-ui-bridge/src/lib.rs crates/ruse-ui-bridge/tests/recovery_bridge.rs crates/ruse-cli/src/command.rs crates/ruse-cli/src/lib.rs crates/ruse-cli/tests/recovery_commands.rs crates/ruse-tui/src/trace_panel.rs crates/ruse-tui/src/transcript.rs docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md docs/plans/M62-02-cli-json-replay-resume-fork.plan.md
git commit -m "feat: add m62-02 recovery cli commands"
```

Expected: implementation commit contains only M62-02 bridge/CLI contracts, tests, and status docs.
