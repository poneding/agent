# M27-04 CLI Eval Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI commands for listing, validating, and running evaluation scenarios.

**Architecture:** `agent-cli` handles argument parsing and rendering. `agent-eval` owns runtime evaluation logic.

**Tech Stack:** Rust 2024 edition, clap or existing CLI parsing style, serde_json, assert_cmd, predicates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-04-cli-eval-commands.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-eval/src/lib.rs`

- [x] **Step 1: Review CLI patterns**

Expected: eval commands follow existing text/JSON command style.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-cli/tests/eval_commands.rs`

- [x] **Step 1: Add CLI eval command tests**

Cover list, validate, run, JSON output, invalid manifest, and exit codes.

### Task 3: Implement Commands

**Files:**
- Modify: `crates/agent-cli/Cargo.toml`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/output.rs`

- [x] **Step 1: Add eval subcommands**
- [x] **Step 2: Delegate to `agent-eval`**
- [x] **Step 3: Render text and JSON reports**

### Task 4: Verify and Commit

- [x] **Step 1: Run checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli --all-targets -- -D warnings
cargo test -p agent-cli eval_commands
```

- [x] **Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock crates/agent-cli crates/agent-eval docs
git commit -m "feat: add cli eval commands"
```
