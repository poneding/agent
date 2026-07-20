# M62-03 Idempotent Recovery and Failed Command Retry UX Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make idempotent side-effect recovery visible and failed command/test retry planning explicit, typed, redacted, and evidence-linked.

**Architecture:** `ruse-core` owns idempotent recovery summaries and command retry planning. `ruse-ui-bridge` owns UI-safe commands/frames. `ruse-cli` parses and renders typed dry-run retry plans. Real retry execution must reuse the existing command/test loop, policy, sandbox, audit, checkpoint, and trace paths.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `CommandTestLoopReport`, existing idempotency store/effects, recovery snapshots, UI bridge frames, CLI text/JSON commands, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62-03-idempotent-retry-ux.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `docs/plans/M62-reliability-recovery-productization.plan.md`
- Read: `crates/ruse-types/src/recovery.rs`
- Read: `crates/ruse-core/src/recovery.rs`
- Read: `crates/ruse-core/src/idempotency.rs`
- Read: `crates/ruse-core/src/command_test.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/recovery_commands.rs`

- [x] **Step 1: Review retry and idempotency foundations**

Identify existing command report fields, failure classifications, redaction helpers, idempotency records, and recovery snapshot output.

### Task 2: Add Runtime Contracts and Retry Planner

**Files:**
- Modify: `crates/ruse-types/src/recovery.rs`
- Create: `crates/ruse-types/src/command_retry.rs`
- Modify: `crates/ruse-types/src/lib.rs`
- Create: `crates/ruse-core/src/command_retry.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify: `crates/ruse-core/src/recovery.rs`
- Create: `crates/ruse-core/tests/failed_command_retry.rs`
- Modify: `crates/ruse-core/tests/recovery_snapshot.rs`

- [x] **Step 1: Define typed retry contracts**

Add serializable contracts for:

- `CommandRetryRequest`
- `CommandRetryPlan`
- `CommandRetryReadiness`
- `CommandRetryDiagnostic`
- `CommandRetryAttemptLink`

Use stable ids, bounded strings, and existing `CommandFailureClassification`/`CommandIntentSummary` where possible.

- [x] **Step 2: Define idempotent recovery disposition contracts**

Add serializable summaries for reusable/skipped/pending/conflicting side effects. Preserve existing `RecoveredToolEffect` compatibility, but add enough status/disposition information for product UI.

- [x] **Step 3: Implement `CommandRetryPlanner`**

Create a runtime planner that builds retry plans from typed command evidence. It must:

- mark command/test/tool failure and timeout as retryable when command intent is present
- preserve `classify_as_test`
- reject successful commands
- block waiting approval and policy-denied commands
- block missing/stale/invalid targets
- create a new attempt id linked to the previous attempt id
- redact reason and diagnostic text

- [x] **Step 4: Extend recovery summaries**

Thread idempotent effect disposition into recovery snapshot or adjacent recovery summary output. Do not expose raw tool input/output.

- [x] **Step 5: Add core tests**

Run:

```bash
cargo test -p ruse-core --test failed_command_retry
cargo test -p ruse-core --test recovery_snapshot
cargo test -p ruse-core --test comprehensive_turn_idempotency
```

Expected: retry plans are typed and evidence-linked, idempotent recovery summaries are safe and redacted, and existing idempotency behavior still passes.

### Task 3: Add Bridge Contracts

**Files:**
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify: `crates/ruse-ui-bridge/tests/recovery_bridge.rs`

- [x] **Step 1: Add retry UI command**

Add a typed command equivalent to:

```rust
CommandRetryPlan {
    command_run_id: String,
    reason: String,
    dry_run: bool,
}
```

Validate non-empty command id, non-empty reason, and planning-only `dry_run` until real execution is wired.

- [x] **Step 2: Add retry/idempotent frames**

Add typed frames for command retry plans and idempotent recovery summaries.

- [x] **Step 3: Add bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test recovery_bridge
```

Expected: command/frame JSON shapes are stable, bounded, and redacted.

### Task 4: Add CLI Retry Command

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/src/lib.rs`
- Modify: `crates/ruse-cli/tests/recovery_commands.rs`

- [x] **Step 1: Add parser coverage**

Support:

```text
ruse-cli command retry <command-run-id> --reason <text> --dry-run
```

If `command` is already used for repo shell execution, keep the existing `repo command` path unchanged and add a top-level `command retry` parser only for retry planning.

- [x] **Step 2: Add deterministic fixture-backed retry plans**

If durable command evidence lookup is not available yet, use deterministic command report fixtures for:

- command failure
- test failure
- timeout
- success not retryable
- waiting approval blocked

Return a typed diagnostic explaining fixture mode.

- [x] **Step 3: Render text and JSON**

Text output should include readiness, classification, command display, previous attempt id, next attempt id, dry-run state, and diagnostics. JSON output should emit the typed retry frame.

- [x] **Step 4: Add CLI tests**

Run:

```bash
cargo test -p ruse-cli --test recovery_commands
```

Expected: retry dry-run text/JSON is stable and rejects missing reason, missing dry-run, invalid target, and secret-like values.

### Task 5: Verification and Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M62-03-idempotent-retry-ux.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-core -p ruse-ui-bridge -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-core --test comprehensive_turn_idempotency
cargo test -p ruse-core --test failed_command_retry
cargo test -p ruse-ui-bridge --test recovery_bridge
cargo test -p ruse-cli --test recovery_commands
```

Expected: all focused checks pass.

- [x] **Step 2: Update status docs**

Mark M62-03 complete only after runtime contracts, bridge frames, CLI output, tests, and focused checks pass.

- [x] **Step 3: Commit planning docs first**

Before implementation, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-03-idempotent-retry-ux.spec.md docs/plans/M62-03-idempotent-retry-ux.plan.md docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md
git commit -m "docs: add m62-03 retry recovery plan"
```

Expected: M62-03 slice planning is committed before code changes.

- [x] **Step 4: Commit implementation after verification**

After focused verification passes, exact-stage only M62-03 files and commit:

```bash
git status --short --branch
git diff --check
git add crates/ruse-types/src/recovery.rs crates/ruse-types/src/command_retry.rs crates/ruse-types/src/lib.rs crates/ruse-core/src/command_retry.rs crates/ruse-core/src/lib.rs crates/ruse-core/src/recovery.rs crates/ruse-core/tests/failed_command_retry.rs crates/ruse-core/tests/recovery_snapshot.rs crates/ruse-ui-bridge/src/command.rs crates/ruse-ui-bridge/src/frame.rs crates/ruse-ui-bridge/src/lib.rs crates/ruse-ui-bridge/tests/recovery_bridge.rs crates/ruse-cli/src/command.rs crates/ruse-cli/src/lib.rs crates/ruse-cli/tests/recovery_commands.rs crates/ruse-tui/src/trace_panel.rs crates/ruse-tui/src/transcript.rs docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md docs/plans/M62-03-idempotent-retry-ux.plan.md
git commit -m "feat: add m62-03 command retry planning"
```

Expected: implementation commit contains only M62-03 runtime, bridge, CLI, tests, and status docs.
