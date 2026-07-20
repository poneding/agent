# M59-04 Diff Viewer and Command Streaming Controls Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve terminal diff review and command/test monitoring with typed, bounded, redacted CLI/JSON/TUI surfaces.

**Architecture:** `ruse-core` continues to own edit/diff and command/test report semantics. `ruse-ui-bridge` owns UI-safe snapshot contracts and any derived display metadata. `ruse-cli` renders text/JSON inspection. `ruse-tui` owns panel state, navigation, and rendering. Cancellation is surfaced as readiness through existing typed snapshot fields unless a later runtime slice adds active cancellation commands.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `EditDiffFlowReport` and `CommandTestLoopReport`, existing CLI repo commands, Ratatui panel state, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M59-04-diff-command-streaming.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/ruse-core/src/edit_diff.rs`
- Read: `crates/ruse-core/src/command_test.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-tui/src/diff_panel.rs`
- Read: `crates/ruse-tui/src/command_panel.rs`
- Read: `crates/ruse-tui/src/app.rs`
- Read: `crates/ruse-tui/src/render.rs`

- [x] **Step 1: Review current diff and command baselines**

Run:

```bash
cargo test -p ruse-ui-bridge --test edit_diff_snapshot
cargo test -p ruse-ui-bridge --test command_test_snapshot
cargo test -p ruse-cli --test edit_diff_flow
cargo test -p ruse-cli --test command_test_loop
cargo test -p ruse-tui --test diff_panel
cargo test -p ruse-tui --test command_panel
```

Expected: existing snapshots, CLI repo edit/command flows, and TUI panels pass before M59-04 changes.

### Task 2: Extend Bridge Snapshot Tests

**Files:**
- Modify: `crates/ruse-ui-bridge/tests/edit_diff_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/tests/command_test_snapshot.rs`

- [x] **Step 1: Add diff snapshot expectations**

Cover:

- preview-only versus applied diff state is preserved
- changed file count and file list are stable
- empty/no-change diff renders explicit metadata
- large diff previews are bounded and marked truncated
- secret-like diff text is redacted
- JSON frame remains single-line

- [x] **Step 2: Add command snapshot expectations**

Cover:

- status/classification fields distinguish success, failure, timeout, and running/waiting-like fixture states when represented
- stdout/stderr byte counts and bounded preview text are stable
- cancellation readiness and reason are preserved
- output chunks or derived display metadata, if added, are deterministic
- secret-like stdout/stderr/observation text is redacted

### Task 3: Implement Bridge Snapshot Metadata

**Files:**
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify if needed: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add typed display metadata only where needed**

Extend `UiEditDiffSnapshot` and `UiCommandTestSnapshot` with serde-friendly fields for daily-driver rendering only if existing fields are insufficient. Keep additions backward-compatible when possible through serde defaults.

- [x] **Step 2: Preserve bounding and redaction**

Use existing bridge helpers for public text, diff text, stdout/stderr, cancellation reasons, and diagnostics. Do not move runtime classification logic into the bridge.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test edit_diff_snapshot
cargo test -p ruse-ui-bridge --test command_test_snapshot
```

Expected: bridge snapshot tests pass.

### Task 4: Improve CLI Diff and Command Rendering

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/tests/edit_diff_flow.rs`
- Modify: `crates/ruse-cli/tests/command_test_loop.rs`

- [x] **Step 1: Add CLI text expectations**

Cover:

- repo edit text includes diff state, changed-file count, line counts, applied status, and redaction/truncation indicators
- repo command text includes status, classification, exit/timed-out state, cancellation readiness, stdout/stderr byte counts, and redaction/truncation indicators
- no-change and failure cases remain stable and bounded

- [x] **Step 2: Add CLI JSON expectations**

Cover:

- `--json repo edit ...` emits `EditDiffSnapshot` with daily-driver fields before events and `Done`
- `--json repo command ...` emits `CommandTestSnapshot` with cancellation readiness before events and `Done`
- JSON output excludes secret-like values

- [x] **Step 3: Implement rendering changes**

Update text renderers to use existing reports or bridge snapshots consistently. Keep exit codes and error frame behavior unchanged.

- [x] **Step 4: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli --test edit_diff_flow
cargo test -p ruse-cli --test command_test_loop
```

Expected: CLI tests pass.

### Task 5: Improve TUI Diff and Command Panels

**Files:**
- Modify: `crates/ruse-tui/src/diff_panel.rs`
- Modify: `crates/ruse-tui/src/command_panel.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/tests/diff_panel.rs`
- Modify: `crates/ruse-tui/tests/command_panel.rs`

- [x] **Step 1: Add TUI diff tests**

Cover snapshot reduction, changed-file metadata, applied/preview/no-change state, line-count rendering, next/previous navigation if added, truncation, redaction, and stable empty-state behavior.

- [x] **Step 2: Add TUI command tests**

Cover snapshot reduction, waiting/running/succeeded/failed/timed-out rendering where fixtures support it, cancellation readiness/reason display, stdout/stderr truncation, redaction, and stable empty-state behavior.

- [x] **Step 3: Implement panel updates**

Derive panel state from `UiJsonFrame::EditDiffSnapshot` and `UiJsonFrame::CommandTestSnapshot`, keep navigation local to TUI state, and keep side effects behind typed `TuiEffect`/`UiCommand` boundaries.

- [x] **Step 4: Run TUI tests**

Run:

```bash
cargo test -p ruse-tui --test diff_panel
cargo test -p ruse-tui --test command_panel
```

Expected: TUI diff and command panel tests pass.

### Task 6: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M59-04-diff-command-streaming.plan.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-ui-bridge --test edit_diff_snapshot
cargo test -p ruse-ui-bridge --test command_test_snapshot
cargo test -p ruse-cli --test edit_diff_flow
cargo test -p ruse-cli --test command_test_loop
cargo test -p ruse-tui --test diff_panel
cargo test -p ruse-tui --test command_panel
```

Expected: focused checks pass.

- [x] **Step 2: Run standard checks**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: workspace checks pass.

- [x] **Step 3: Commit**

Run exact-path staging for M59-04 source, tests, and docs, then:

```bash
git commit -m "feat: add m59 diff and command panels"
```

Expected: commit contains only M59-04 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Diff viewer exposes stable changed-file, line-count, state, preview, truncation, and redaction evidence.
- [x] Command panel exposes stable status, classification, output, timed-out, cancellation readiness, truncation, and redaction evidence.
- [x] CLI text/JSON repo edit and command outputs are stable, bounded, and redacted.
- [x] TUI diff and command panels reduce bridge snapshots and support repeated coding-loop review.
- [x] Presentation crates do not own runtime policy, command execution, diff application, or cancellation semantics.
- [x] Focused and standard checks pass.
