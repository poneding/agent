# M62-02 CLI/JSON Replay, Resume, and Fork Commands Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose M62-01 recovery snapshots and dry-run reports through stable UI bridge contracts and CLI text/JSON commands for recovery status, resume dry-run, replay inspect, and replay fork planning.

**Architecture:** `agent-core` remains recovery truth. `agent-ui-bridge` owns typed UI command/frame contracts. `agent-cli` parses and renders typed reports without duplicating recovery logic. This slice is read-only and dry-run oriented.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `RecoverySnapshot` and `ResumeDryRunReport`, UI bridge commands/frames, CLI parser/rendering patterns, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62-02-cli-json-replay-resume-fork.spec.md`
- Read: `docs/specs/M62-01-recovery-snapshot-resume.spec.md`
- Read: `docs/plans/M62-01-recovery-snapshot-resume.plan.md`
- Read: `crates/agent-types/src/recovery.rs`
- Read: `crates/agent-core/src/recovery.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/plan_todo_commands.rs`
- Read: `crates/agent-cli/tests/json_mode.rs`

- [ ] **Step 1: Review CLI and bridge patterns**

Identify existing command parsing, text rendering, JSON frame rendering, fixture helpers, and stable serde test style.

### Task 2: Add Bridge Contracts

**Files:**
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Create: `crates/agent-ui-bridge/tests/recovery_bridge.rs`

- [ ] **Step 1: Add UI command variants**

Add typed command variants for:

- `RecoveryStatus`
- `RecoveryResumeDryRun`
- `ReplayInspect`
- `ReplayFork`

Use existing ID types and snake_case serde tagging.

- [ ] **Step 2: Add UI recovery frame/report types**

Add typed report/frame wrappers for:

- recovery snapshot
- resume dry-run report
- replay inspect report
- replay fork report

Wrap existing `agent-types` recovery contracts where possible.

- [ ] **Step 3: Add bridge serde tests**

Run:

```bash
cargo test -p agent-ui-bridge --test recovery_bridge
```

Expected initially: tests fail before bridge contracts exist, then pass after implementation.

### Task 3: Add CLI Commands

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Create: `crates/agent-cli/tests/recovery_commands.rs`

- [ ] **Step 1: Add parser coverage**

Support command forms:

```text
agent-cli recovery status --conversation <id> [--turn <id>]
agent-cli recovery resume --conversation <id> [--turn <id>] [--checkpoint <id>] --dry-run
agent-cli replay inspect --conversation <id> [--from-checkpoint <id>]
agent-cli replay fork --conversation <id> [--turn <id>] [--checkpoint <id>] --label <name> --dry-run
```

- [ ] **Step 2: Add deterministic fixture-backed reports**

If the CLI does not yet have arbitrary persisted session lookup by conversation id, use deterministic fixture helpers for M62-02 tests while preserving the real command shape and returning typed diagnostics when live lookup is unavailable.

- [ ] **Step 3: Render stable text and JSON**

Text output should include readiness, replay_from, replayed_event_count, pending tool count, idempotent effect count, pending approval count, and diagnostics. JSON output should emit typed data frames suitable for automation.

- [ ] **Step 4: Add CLI tests**

Run:

```bash
cargo test -p agent-cli --test recovery_commands
```

Expected: text and JSON commands are stable, bounded, redacted, and reject invalid arguments safely.

### Task 4: Verification and Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md` if status text needs refreshing
- Modify: `docs/plans/M62-02-cli-json-replay-resume-fork.plan.md`

- [ ] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-ui-bridge -p agent-cli --all-targets -- -D warnings
cargo test -p agent-ui-bridge --test recovery_bridge
cargo test -p agent-cli --test recovery_commands
```

Expected: all focused checks pass.

- [ ] **Step 2: Update status docs**

Mark M62-02 complete only after command contracts, CLI output, tests, and checks pass.

- [ ] **Step 3: Commit planning docs first**

Before implementation, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-02-cli-json-replay-resume-fork.spec.md docs/plans/M62-02-cli-json-replay-resume-fork.plan.md docs/milestones/README.md
git commit -m "docs: add m62-02 recovery cli plan"
```

Expected: M62-02 slice planning is committed before code changes.

- [ ] **Step 4: Commit implementation after verification**

After focused verification passes, exact-stage only M62-02 files and commit:

```bash
git status --short --branch
git diff --check
git add crates/agent-ui-bridge/src/command.rs crates/agent-ui-bridge/src/frame.rs crates/agent-ui-bridge/tests/recovery_bridge.rs crates/agent-cli/src/command.rs crates/agent-cli/tests/recovery_commands.rs docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md docs/plans/M62-02-cli-json-replay-resume-fork.plan.md
git commit -m "feat: add m62-02 recovery cli commands"
```

Expected: implementation commit contains only M62-02 bridge/CLI contracts, tests, and status docs.
