# M20 Terminal UX and TUI REPL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the terminal product entrypoint where `agent` opens a Ratatui-powered REPL in interactive terminals while preserving batch, JSON, and plain modes.

**Architecture:** `ruse-cli` owns the installed binary, mode parsing, and TTY dispatch. `ruse-tui` owns Ratatui/Crossterm rendering, terminal lifecycle, and UI state. Runtime communication goes through `ruse-ui-bridge`.

**Tech Stack:** Rust 2024 edition, Ratatui, Crossterm, tui-textarea, serde UI bridge frames, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M20 Technical Design

**Files:**
- Create: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`
- Create: `docs/plans/M20-00-terminal-ux-tui-technical-design.plan.md`

- [x] **Step 1: Write the M20 technical design baseline**

Define binary naming, TTY detection, `ruse-tui` crate ownership, app loop, terminal cleanup, keyboard model, panels, and fallback behavior.

- [x] **Step 2: Write the M20 design execution plan**

Document slice order and testing strategy for pure state and rendered buffers.

### Task 2: Decompose M20 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M20-01-agent-binary-entrypoint.spec.md`
- Create: `docs/plans/M20-01-agent-binary-entrypoint.plan.md`
- Create: `docs/specs/M20-02-ratatui-shell.spec.md`
- Create: `docs/plans/M20-02-ratatui-shell.plan.md`
- Create: `docs/specs/M20-03-tui-composer-repl.spec.md`
- Create: `docs/plans/M20-03-tui-composer-repl.plan.md`
- Create: `docs/specs/M20-04-streaming-transcript.spec.md`
- Create: `docs/plans/M20-04-streaming-transcript.plan.md`
- Create: `docs/specs/M20-05-approval-diff-panels.spec.md`
- Create: `docs/plans/M20-05-approval-diff-panels.plan.md`
- Create: `docs/specs/M20-06-trace-task-panels.spec.md`
- Create: `docs/plans/M20-06-trace-task-panels.plan.md`
- Create: `docs/specs/M20-07-plain-repl-fallback.spec.md`
- Create: `docs/plans/M20-07-plain-repl-fallback.plan.md`

- [x] **Step 1: Create slice specs**

Each slice specifies state models, input events, rendering behavior, cleanup behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan starts with pure state tests before rendering or terminal integration.

### Task 3: Implement M20 Slices in Order

**Files:**
- Execute: `docs/plans/M20-01-agent-binary-entrypoint.plan.md`
- Execute: `docs/plans/M20-02-ratatui-shell.plan.md`
- Execute: `docs/plans/M20-03-tui-composer-repl.plan.md`
- Execute: `docs/plans/M20-04-streaming-transcript.plan.md`
- Execute: `docs/plans/M20-05-approval-diff-panels.plan.md`
- Execute: `docs/plans/M20-06-trace-task-panels.plan.md`
- Execute: `docs/plans/M20-07-plain-repl-fallback.plan.md`

- [x] **Step 1: Add `agent` binary dispatch**

Configure the installed binary name and mode parsing so `agent` opens TUI in interactive terminals.

- [x] **Step 2: Add Ratatui shell**

Create `ruse-tui` with app state, terminal guard, layout, and exit behavior.

- [x] **Step 3: Add composer and REPL input**

Implement multi-line composer, slash command routing, history, and submission events.

- [x] **Step 4: Render streaming transcript**

Display model deltas, tool calls, observations, and final responses from bridge events.

- [x] **Step 5: Add approval and diff panels**

Render pending approvals and diff previews with explicit keyboard actions.

- [x] **Step 6: Add trace and task panels**

Display runtime trace timeline and background task status.

- [x] **Step 7: Add plain REPL fallback**

Support constrained terminals and scripts without entering raw-mode TUI.

### Task 4: Complete M20

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Mark M20 complete**

Update milestone status and advance active focus to M21.

- [ ] **Step 3: Commit M20 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m20 terminal ux"
```

Expected: commit succeeds.
