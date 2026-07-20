# M26-04 UI Observability Snapshots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI-safe observability snapshots for TUI and Desktop.

**Architecture:** `ruse-ui-bridge` normalizes core evidence; `ruse-tui` and `ruse-desktop` consume typed snapshots.

**Tech Stack:** Rust 2024 edition, serde, ruse-ui-bridge frames, Desktop/TUI tests, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-04-ui-observability-snapshots.spec.md`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-desktop/src/trace.rs`
- Read: `crates/ruse-tui/src/app.rs`

- [x] **Step 1: Review UI bridge snapshot patterns**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-ui-bridge/tests/observability_snapshot.rs`

- [x] **Step 1: Add UI snapshot tests**

### Task 3: Implement Snapshots

**Files:**
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify if needed: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add snapshot contracts and reducers**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

```bash
cargo test -p ruse-ui-bridge observability
cargo clippy -p ruse-ui-bridge --all-targets -- -D warnings
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M26-04 complete and commit**

```bash
git add crates/ruse-ui-bridge crates/ruse-desktop crates/ruse-tui docs
git commit -m "feat: add ui observability snapshots"
```
