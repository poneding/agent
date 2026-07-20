# M20-06 Trace and Task Panels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add inspectable trace and task panels to the TUI.

**Architecture:** `ruse-tui` derives trace and task panel state from UI bridge frames. Storage-backed trace loading remains outside this slice.

**Tech Stack:** Rust 2024 edition, `ruse-tui`, `ruse-ui-bridge`, Ratatui buffer tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-06-trace-task-panels.spec.md`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-tui/src/slash.rs`
- Read: `crates/ruse-tui/src/transcript.rs`

- [x] **Step 1: Review panel data sources**

Run:

```bash
sed -n '1,220p' docs/specs/M20-06-trace-task-panels.spec.md
```

Expected: panels consume UI-safe bridge events only.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-tui/tests/trace_panel.rs`
- Create: `crates/ruse-tui/tests/task_panel.rs`

- [x] **Step 1: Add trace panel tests**

Cover entry ordering, filters, diagnostics retention, empty state, and `/trace` focus.

- [x] **Step 2: Add task panel tests**

Cover task notifications, bounded list behavior, empty state, and `/tasks` focus.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-tui --test trace_panel
cargo test -p ruse-tui --test task_panel
```

Expected: fails because trace/task panel contracts are missing.

### Task 3: Implement Panels

**Files:**
- Create: `crates/ruse-tui/src/trace_panel.rs`
- Create: `crates/ruse-tui/src/task_panel.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/src/slash.rs`
- Modify: `crates/ruse-tui/src/lib.rs`

- [x] **Step 1: Add trace state**

Track ordered trace entries and filters from UI event payloads.

- [x] **Step 2: Add task state**

Track task notifications and status summaries.

- [x] **Step 3: Wire slash commands**

Open trace/task panels from `/trace` and `/tasks`.

- [x] **Step 4: Render panels**

Render empty, populated, and narrow states with stable dimensions.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-tui/tests/trace_panel.rs`
- Read: `crates/ruse-tui/tests/task_panel.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-tui --test trace_panel
cargo test -p ruse-tui --test task_panel
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
- Modify: `docs/plans/M20-06-trace-task-panels.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-06 complete**

Update status and set focus to M20-07.
