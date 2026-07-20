# M20-02 Ratatui Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal `ruse-tui` crate with safe terminal lifecycle, pure state, and renderable shell.

**Architecture:** `ruse-tui` owns terminal UI state and rendering. It receives UI bridge frames later, but this slice only creates the app shell and launcher boundary.

**Tech Stack:** Rust 2024 edition, Ratatui, Crossterm, `tui-textarea`, `ruse-ui-bridge`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-02-ratatui-shell.spec.md`
- Read: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`
- Read: `Cargo.toml`

- [x] **Step 1: Review crate boundary**

Run:

```bash
sed -n '1,220p' docs/specs/M20-02-ratatui-shell.spec.md
```

Expected: `ruse-tui` is UI-only and runtime-agnostic.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-tui/tests/shell_state.rs`
- Create: `crates/ruse-tui/tests/shell_render.rs`

- [x] **Step 1: Add pure reducer tests**

Cover initial state, resize action, help screen switch, and quit request.

- [x] **Step 2: Add render buffer tests**

Assert normal and narrow layouts render transcript placeholder, composer placeholder, and status bar.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-tui
```

Expected: fails because `ruse-tui` does not exist.

### Task 3: Implement Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-tui/Cargo.toml`
- Create: `crates/ruse-tui/src/lib.rs`
- Create: `crates/ruse-tui/src/app.rs`
- Create: `crates/ruse-tui/src/render.rs`
- Create: `crates/ruse-tui/src/terminal.rs`

- [x] **Step 1: Add workspace crate and dependencies**

Add Ratatui, Crossterm, `tui-textarea`, `ruse-ui-bridge`, and `ruse-types`.

- [x] **Step 2: Add app state and reducer**

Implement `TuiApp`, `TuiScreen`, `TuiAction`, and `TuiApp::reduce`.

- [x] **Step 3: Add render shell**

Implement `render_app` with transcript/composer/status layout and narrow fallback.

- [x] **Step 4: Add terminal guard**

Implement terminal enter/exit wrapper with cleanup on drop.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-tui/tests/shell_state.rs`
- Read: `crates/ruse-tui/tests/shell_render.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-tui
```

Expected: tests pass.

- [x] **Step 2: Run workspace check**

Run:

```bash
cargo xtask check
```

Expected: workspace remains green with new crate.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M20-02-ratatui-shell.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-02 complete**

Update status after verification and set focus to M20-03.
