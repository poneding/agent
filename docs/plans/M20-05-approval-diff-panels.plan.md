# M20-05 Approval and Diff Panels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add TUI side panels for pending approvals and diff previews with explicit allow/deny command effects.

**Architecture:** `agent-tui` owns panel state and rendering. Approval decisions are emitted as `agent-ui-bridge` commands; core remains responsible for validation and resume.

**Tech Stack:** Rust 2024 edition, `agent-tui`, `agent-ui-bridge`, Ratatui buffer tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-05-approval-diff-panels.spec.md`
- Read: `crates/agent-cli/src/approval.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/event.rs`

- [x] **Step 1: Review approval command boundary**

Run:

```bash
sed -n '1,220p' docs/specs/M20-05-approval-diff-panels.spec.md
sed -n '1,220p' crates/agent-ui-bridge/src/command.rs
```

Expected: TUI emits commands and does not create approval leases.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-tui/tests/approval_panel.rs`
- Create: `crates/agent-tui/tests/diff_panel.rs`

- [x] **Step 1: Add approval reducer tests**

Cover pending approval add, focus movement, allow command, deny command, abort, and resolved removal.

- [x] **Step 2: Add render tests**

Cover redacted approval preview, command resource, and truncated diff preview.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p agent-tui --test approval_panel
cargo test -p agent-tui --test diff_panel
```

Expected: fails because panel contracts are missing.

### Task 3: Implement Panels

**Files:**
- Create: `crates/agent-tui/src/approval_panel.rs`
- Create: `crates/agent-tui/src/diff_panel.rs`
- Modify: `crates/agent-tui/src/app.rs`
- Modify: `crates/agent-tui/src/render.rs`
- Modify: `crates/agent-tui/src/lib.rs`

- [x] **Step 1: Add approval panel state**

Track pending approvals, focused item, and local denial reason.

- [x] **Step 2: Add approval actions**

Emit `UiCommand::ResolveApproval` for allow and deny, and local abort effect for abort.

- [x] **Step 3: Add diff panel state**

Render file edit summaries and truncation metadata from approval previews.

- [x] **Step 4: Wire panel rendering**

Display side panel when focused approval or diff state exists.

### Task 4: Verify

**Files:**
- Read: `crates/agent-tui/tests/approval_panel.rs`
- Read: `crates/agent-tui/tests/diff_panel.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-tui --test approval_panel
cargo test -p agent-tui --test diff_panel
```

Expected: focused tests pass.

- [x] **Step 2: Run TUI tests**

Run:

```bash
cargo test -p agent-tui
```

Expected: all TUI tests pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M20-05-approval-diff-panels.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-05 complete**

Update status and set focus to M20-06.
