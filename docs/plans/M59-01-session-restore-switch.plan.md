# M59-01 Session Restore and Switch Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed terminal session directory, restore validation, CLI/JSON commands, bridge snapshots, and TUI session panel state for daily-driver session continuity.

**Architecture:** `ruse-core` owns session directory derivation and validation. `ruse-ui-bridge` maps reports to UI-safe snapshots and commands. `ruse-cli` renders list/resume commands. `ruse-tui` renders panel state and emits typed switch effects. Runtime logic stays out of presentation crates.

**Tech Stack:** Rust 2024 edition, serde contracts, `ruse-types` ids, existing repo session reports and runtime events, CLI text/JSON tests, Ratatui state tests, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M59-01-session-restore-switch.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `crates/ruse-core/src/repo_session.rs`
- Read: `crates/ruse-storage/src/event_store.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-tui/src/app.rs`
- Read: `crates/ruse-tui/src/slash.rs`

- [x] **Step 1: Review session baseline**

Run:

```bash
cargo test -p ruse-core --test repo_session_bootstrap
cargo test -p ruse-cli --test repo_session_bootstrap
cargo test -p ruse-ui-bridge repo_session_snapshot
cargo test -p ruse-tui --test slash_commands
```

Expected: existing repo session bootstrap, bridge snapshot, and slash command tests pass before session directory changes.

Baseline verification on 2026-07-01:

- `cargo test -p ruse-core --test repo_session_bootstrap`
- `cargo test -p ruse-cli --test repo_session_bootstrap`
- `cargo test -p ruse-ui-bridge repo_session_snapshot`
- `cargo test -p ruse-tui`

### Task 2: Add Core Session Directory Tests

**Files:**
- Create: `crates/ruse-core/tests/terminal_session_directory.rs`

- [x] **Step 1: Add failing core tests**

Cover:

- empty directory returns a successful empty report
- session summaries are sorted by latest event descending
- duplicate workspace/conversation records collapse to the latest record
- pending approvals, running commands, and tasks derive a stable status
- invalid restore requests fail before UI focus changes
- workspace display, title, provider/model ids, and diagnostics are bounded and redacted

- [x] **Step 2: Run core tests and verify failure**

Run:

```bash
cargo test -p ruse-core --test terminal_session_directory
```

Expected: tests fail before terminal session contracts exist.

### Task 3: Implement Core Session Contracts

**Files:**
- Create: `crates/ruse-core/src/terminal_session.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add typed contracts**

Implement terminal session record, summary, status, diagnostics, directory report, and restore validation contracts with serde-friendly public shapes where durable or UI-facing.

- [x] **Step 2: Add deterministic directory derivation**

Derive summaries from typed records or existing repo session reports/events. Keep sorting, duplicate collapse, status classification, and redaction in shared runtime code.

- [x] **Step 3: Run core tests**

Run:

```bash
cargo test -p ruse-core --test terminal_session_directory
```

Expected: core session directory tests pass.

### Task 4: Add UI Bridge Session Snapshot and Switch Command

**Files:**
- Create: `crates/ruse-ui-bridge/tests/terminal_session_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add bridge tests**

Cover stable JSON snapshot shape, command validation for switch requests, bounded diagnostics, redaction, and single-line JSON frame encoding.

- [x] **Step 2: Implement bridge mapping**

Add `UiTerminalSessionSnapshot`, summaries, diagnostics, `UiJsonFrame` variant, and switch command validation. Reuse core reports; do not derive session status in the bridge.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test terminal_session_snapshot
```

Expected: bridge session snapshot tests pass.

### Task 5: Add CLI Session List and Resume Commands

**Files:**
- Create: `crates/ruse-cli/tests/terminal_session_commands.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo session list` text output for empty and populated directories
- `--json repo session list` emits a session snapshot and `Done`
- `repo session resume --conversation-id <id>` returns a stable summary
- invalid ids exit `2`
- default `repo session` bootstrap behavior remains unchanged
- output is bounded and redacted

- [x] **Step 2: Implement CLI routing and rendering**

Extend existing repo session commands without breaking `repo session --workspace`. Use core session helpers and bridge snapshots for JSON.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli --test terminal_session_commands
cargo test -p ruse-cli --test repo_session_bootstrap
```

Expected: session command tests and existing repo session bootstrap tests pass.

### Task 6: Add TUI Session Panel State

**Files:**
- Create: `crates/ruse-tui/src/session_panel.rs`
- Create: `crates/ruse-tui/tests/session_panel.rs`
- Modify: `crates/ruse-tui/src/lib.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/slash.rs`
- Modify: `crates/ruse-tui/tests/slash_commands.rs`

- [x] **Step 1: Add TUI state tests**

Cover snapshot reduction, focus next/previous, empty state rendering, switch effect emission, and `/sessions` slash command parsing.

- [x] **Step 2: Implement session panel state**

Add panel state derived from bridge snapshots and a typed effect for switch requests. Keep rendering bounded and side-effect free.

- [x] **Step 3: Run TUI tests**

Run:

```bash
cargo test -p ruse-tui --test session_panel
cargo test -p ruse-tui --test slash_commands
```

Expected: TUI session panel and slash command tests pass.

### Task 7: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M59-01-session-restore-switch.plan.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`
- Modify: `docs/milestones/README.md`
- Modify if needed: `README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-core --test terminal_session_directory
cargo test -p ruse-cli --test terminal_session_commands
cargo test -p ruse-ui-bridge --test terminal_session_snapshot
cargo test -p ruse-tui --test session_panel
cargo test -p ruse-tui --test slash_commands
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M59-01 source, tests, and docs, then:

```bash
git commit -m "feat: add m59 terminal session restore"
```

Expected: commit contains only M59-01 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Core terminal session contracts are typed, serde-friendly, and tested.
- [x] Session directory sorting, duplicate collapse, status classification, and restore validation are deterministic.
- [x] CLI text/JSON list and resume commands are stable and redacted.
- [x] UI bridge session snapshot and switch command are stable and safe.
- [x] TUI session panel can display sessions and emit switch effects.
- [x] Existing repo session bootstrap behavior remains unchanged.
- [x] Focused fmt/clippy/tests pass.
