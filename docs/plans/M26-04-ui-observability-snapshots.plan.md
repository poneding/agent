# M26-04 UI Observability Snapshots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI-safe observability snapshots for TUI and Desktop.

**Architecture:** `agent-ui-bridge` normalizes core evidence; `agent-tui` and `agent-desktop` consume typed snapshots.

**Tech Stack:** Rust 2024 edition, serde, agent-ui-bridge frames, Desktop/TUI tests, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-04-ui-observability-snapshots.spec.md`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-desktop/src/trace.rs`
- Read: `crates/agent-tui/src/app.rs`

- [ ] **Step 1: Review UI bridge snapshot patterns**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/observability_snapshot.rs`

- [ ] **Step 1: Add UI snapshot tests**

### Task 3: Implement Snapshots

**Files:**
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify if needed: `crates/agent-ui-bridge/src/lib.rs`

- [ ] **Step 1: Add snapshot contracts and reducers**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

```bash
cargo test -p agent-ui-bridge observability
cargo clippy -p agent-ui-bridge --all-targets -- -D warnings
```

### Task 5: Update Status and Commit

- [ ] **Step 1: Mark M26-04 complete and commit**

```bash
git add crates/agent-ui-bridge crates/agent-desktop crates/agent-tui docs
git commit -m "feat: add ui observability snapshots"
```
