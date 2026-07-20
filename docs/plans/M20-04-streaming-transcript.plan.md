# M20-04 Streaming Transcript Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert UI bridge event frames into a live transcript rendered by the TUI.

**Architecture:** `ruse-tui` owns transcript state and rendering. It consumes `ruse-ui-bridge` frames and keeps all runtime normalization in bridge/core crates.

**Tech Stack:** Rust 2024 edition, `ruse-tui`, `ruse-ui-bridge`, Ratatui buffer tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-04-streaming-transcript.spec.md`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-tui/src/app.rs`

- [x] **Step 1: Review bridge frame contracts**

Run:

```bash
sed -n '1,260p' crates/ruse-ui-bridge/src/event.rs
sed -n '1,220p' docs/specs/M20-04-streaming-transcript.spec.md
```

Expected: transcript state consumes UI-safe frames only.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-tui/tests/transcript.rs`
- Create: `crates/ruse-tui/tests/transcript_render.rs`

- [x] **Step 1: Add reducer tests**

Cover model delta aggregation, final response closure, tool call/result items, approval requested item, and diagnostic item.

- [x] **Step 2: Add render tests**

Assert transcript renders active stream, completed answer, and diagnostics in stable buffer output.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-tui --test transcript
cargo test -p ruse-tui --test transcript_render
```

Expected: fails because transcript contracts are missing.

### Task 3: Implement Transcript

**Files:**
- Create: `crates/ruse-tui/src/transcript.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/src/lib.rs`

- [x] **Step 1: Add transcript item model**

Implement typed transcript items and stable ordering.

- [x] **Step 2: Add frame reducer**

Handle `UiEventPayload` variants for model, tool, approval, diagnostics, progress, and final response.

- [x] **Step 3: Add render function**

Render transcript rows with bounded width and stable scrolling behavior.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-tui/tests/transcript.rs`
- Read: `crates/ruse-tui/tests/transcript_render.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-tui --test transcript
cargo test -p ruse-tui --test transcript_render
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
- Modify: `docs/plans/M20-04-streaming-transcript.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-04 complete**

Update status and set focus to M20-05.
