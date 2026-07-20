# M20-03 TUI Composer REPL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add prompt composition, multiline input, history, slash commands, and submit effects to `ruse-tui`.

**Architecture:** Composer state is pure and emits effects. The app loop maps submit effects to UI bridge commands but does not run runtime logic directly.

**Tech Stack:** Rust 2024 edition, `ruse-tui`, `ruse-ui-bridge`, Ratatui buffer tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-03-tui-composer-repl.spec.md`
- Read: `crates/ruse-tui/src/app.rs`
- Read: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Review composer boundary**

Run:

```bash
sed -n '1,220p' docs/specs/M20-03-tui-composer-repl.spec.md
```

Expected: composer emits effects and does not call runtime crates.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-tui/tests/composer.rs`
- Create: `crates/ruse-tui/tests/slash_commands.rs`

- [x] **Step 1: Add composer reducer tests**

Cover text input, submit, multiline newline, history up/down, and empty submit.

- [x] **Step 2: Add slash command tests**

Cover `/help`, `/status`, `/trace`, `/tasks`, `/settings`, `/clear`, `/quit`, and unknown commands.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-tui --test composer
cargo test -p ruse-tui --test slash_commands
```

Expected: fails because composer contracts are missing.

### Task 3: Implement Composer

**Files:**
- Create: `crates/ruse-tui/src/composer.rs`
- Create: `crates/ruse-tui/src/slash.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/src/lib.rs`

- [x] **Step 1: Add composer state**

Implement `ComposerState`, history cursor, multiline flag, and submit state.

- [x] **Step 2: Add composer actions**

Implement reducer actions for text input, enter, newline, history, clear, and escape.

- [x] **Step 3: Add slash command parser**

Implement typed slash commands and diagnostics for unknown commands.

- [x] **Step 4: Render composer**

Render composer content with stable height and truncation for narrow widths.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-tui/tests/composer.rs`
- Read: `crates/ruse-tui/tests/slash_commands.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-tui --test composer
cargo test -p ruse-tui --test slash_commands
```

Expected: focused tests pass.

- [x] **Step 2: Run TUI tests**

Run:

```bash
cargo test -p ruse-tui
```

Expected: all TUI tests pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M20-03-tui-composer-repl.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-03 complete**

Update status and set focus to M20-04.
