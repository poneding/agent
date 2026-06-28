# M27-02 Scenario Runner Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute validated evaluation scenarios and produce typed reports.

**Architecture:** `agent-eval::runner` consumes manifest contracts and calls runtime/replay/evidence crates. UI crates stay consumers only.

**Tech Stack:** Rust 2024 edition, serde, existing replay/evidence contracts, tempfile, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-02-scenario-runner-core.spec.md`
- Read: `crates/agent-eval/src/manifest.rs`

- [ ] **Step 1: Review manifest contracts**

Expected: runner consumes validated scenarios only.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-eval/tests/runner.rs`

- [ ] **Step 1: Add deterministic runner tests**

Cover replay-trace, evidence-only, strict unsupported kind, warn skipped kind, and interrupted run reporting.

### Task 3: Implement Runner

**Files:**
- Create: `crates/agent-eval/src/runner.rs`
- Create: `crates/agent-eval/src/report.rs`
- Modify: `crates/agent-eval/src/lib.rs`

- [ ] **Step 1: Add report contracts**
- [ ] **Step 2: Add replay/evidence scenario execution**
- [ ] **Step 3: Add strictness and interruption handling**

### Task 4: Verify and Commit

- [ ] **Step 1: Run checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-eval --all-targets -- -D warnings
cargo test -p agent-eval runner
```

- [ ] **Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock crates/agent-eval docs
git commit -m "feat: add eval scenario runner"
```
