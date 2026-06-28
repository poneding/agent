# M26-03 CLI Observability Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add script-safe CLI observability summary and JSON evidence commands.

**Architecture:** `agent-cli` renders evidence built by `agent-core`; runtime logic stays out of CLI.

**Tech Stack:** Rust 2024 edition, existing CLI command parser, serde_json, agent-core evidence APIs, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-03-cli-observability-commands.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/trace_commands.rs`

- [x] **Step 1: Review existing CLI trace command patterns**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-cli/tests/observability_commands.rs`

- [x] **Step 1: Add CLI observability tests**

### Task 3: Implement Commands

**Files:**
- Modify: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Add summary and JSON output commands**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

```bash
cargo test -p agent-cli observability
cargo clippy -p agent-cli --all-targets -- -D warnings
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M26-03 complete and commit**

```bash
git add crates/agent-cli docs
git commit -m "feat: add cli observability commands"
```
