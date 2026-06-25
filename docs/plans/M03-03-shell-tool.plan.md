# M03-03 Shell Tool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a controlled shell command tool with cwd confinement, timeout handling, and captured output.

**Architecture:** `agent-tool` implements `ShellCommandTool`. `agent-sandbox` validates cwd, and command execution uses program plus args rather than shell strings.

**Tech Stack:** Rust 2024 edition, `agent-tool`, `agent-sandbox`, `serde_json`, `tempfile`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Shell Tool Tests

**Files:**
- Create: `crates/agent-tool/tests/shell_command_tool.rs`

- [x] **Step 1: Add failing tests**

Tests must prove successful command capture, non-zero status capture, timeout, stderr capture, and cwd traversal rejection.

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-tool shell_command_tool
```

Expected: tests fail because `ShellCommandTool` does not exist.

### Task 2: Implement Shell Tool

**Files:**
- Create: `crates/agent-tool/src/builtins/shell_command.rs`
- Modify: `crates/agent-tool/src/builtins/mod.rs`

- [x] **Step 1: Implement input parsing**

Accept `program`, `args`, `cwd`, `timeout_ms`, and `env` JSON fields.

- [x] **Step 2: Implement execution and timeout**

Spawn the program with explicit args and kill it on timeout.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-tool shell_command_tool
```

Expected: focused tests pass.

### Task 3: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M03-03-shell-tool.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M03-03-shell-tool` to `M03-04-git-tools`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m03-03 shell tool"
```
