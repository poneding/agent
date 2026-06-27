# M20-07 Plain REPL Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit and fallback plain REPL behavior without entering raw terminal mode.

**Architecture:** `agent-cli` owns the plain line-oriented loop. It reuses existing prompt runtime, slash command, and approval prompt helpers.

**Tech Stack:** Rust 2024 edition, `agent-cli`, stdin/stdout fixtures, existing approval helpers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-07-plain-repl-fallback.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/src/approval.rs`
- Read: `crates/agent-cli/tests/slash_commands.rs`

- [x] **Step 1: Review plain REPL boundary**

Run:

```bash
sed -n '1,220p' docs/specs/M20-07-plain-repl-fallback.spec.md
```

Expected: plain REPL stays in `agent-cli` and does not depend on Ratatui.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-cli/tests/plain_repl.rs`

- [x] **Step 1: Add command parsing tests**

Cover `agent repl --plain`, non-TTY no-arg fallback failure, and script-safe JSON/batch modes.

- [x] **Step 2: Add fixture loop tests**

Cover prompt submission, slash command, empty line ignore, EOF exit, and recoverable runtime diagnostic.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p agent-cli --test plain_repl
```

Expected: fails because plain REPL contracts are missing.

### Task 3: Implement Plain REPL

**Files:**
- Create: `crates/agent-cli/src/repl.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/lib.rs`
- Modify: `crates/agent-cli/src/main.rs`

- [x] **Step 1: Add REPL contracts**

Implement `PlainReplEvent`, prompt rendering, and line parsing.

- [x] **Step 2: Add fixture runner**

Implement a testable loop over injected input/output buffers.

- [x] **Step 3: Wire CLI command**

Route `repl --plain` and constrained terminal fallback to the plain runner.

- [x] **Step 4: Reuse approval helpers**

When approval frames are encountered, use M19-04 prompt rendering and command conversion.

### Task 4: Verify

**Files:**
- Read: `crates/agent-cli/tests/plain_repl.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-cli --test plain_repl
```

Expected: focused tests pass.

- [x] **Step 2: Run CLI and TUI tests**

Run:

```bash
cargo test -p agent-cli
cargo test -p agent-tui
```

Expected: terminal UX crates remain green.

### Task 5: Complete M20

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`

- [x] **Step 1: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: all workspace checks pass.

- [x] **Step 2: Mark M20 complete**

Update status and set next focus to M21.
