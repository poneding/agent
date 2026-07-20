# M33-03 CLI Workspace Tool Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit workspace-backed built-in tool catalog wiring to `ruse-cli mcp server stdio --profile`.

**Architecture:** `ruse-cli` parses flags and passes path inputs to `ruse-mcp`; runtime tool selection stays in `ruse-mcp`.

**Tech Stack:** Rust 2024 edition, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review CLI stdio profile code and M33 catalog helper**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-cli/tests/mcp_server_workspace_tool_profile.rs`
- Modify: `crates/ruse-cli/tests/mcp_server_profile_command.rs`

- [x] **Step 1: Test `--workspace` parsing**
- [x] **Step 2: Test missing workspace fails for tool profiles**
- [x] **Step 3: Test resource-only profile still starts without workspace**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_workspace_tool_profile
```

Expected: tests fail before implementation.

### Task 3: Implement CLI Wiring

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add `workspace_root` to stdio command/config**
- [x] **Step 2: Parse `--workspace <path>`**
- [x] **Step 3: Build catalog only when selected profile declares tools**
- [x] **Step 4: Preserve protocol-only stdout**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_workspace_tool_profile
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_profile_command
```

- [x] **Step 1: Run focused CLI checks**

Verified:

```text
fmt-check passed
clippy -p ruse-cli passed
ruse-cli mcp_server_workspace_tool_profile: 3 passed
ruse-cli mcp_server_profile_command: 4 passed
ruse-cli mcp_server_stdio_command: 5 passed
ruse-cli full crate tests passed
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio workspace tool profiles"
```

- [x] **Step 1: Commit M33-03**
