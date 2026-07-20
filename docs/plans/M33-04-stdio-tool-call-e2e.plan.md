# M33-04 Stdio Tool Call E2E Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify manifest-exported built-in tools work over MCP stdio list and call frames.

**Architecture:** Tests exercise CLI stdio IO and MCP session behavior through public contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review existing stdio command tests and server session tests**

### Task 2: Write E2E Tests

**Files:**
- Modify: `crates/ruse-cli/tests/mcp_server_workspace_tool_profile.rs`
- Create: `crates/ruse-mcp/tests/mcp_server_builtin_tool_call.rs`

- [x] **Step 1: Add initialize/list/call fixture**
- [x] **Step 2: Add non-exported tool call failure fixture**
- [x] **Step 3: Assert stdout is protocol-only**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_workspace_tool_profile
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_call
```

Expected: tests fail until prior slices are wired.

### Task 3: Implement Minimal Fixes

**Files:**
- Modify only runtime files needed by failing tests.

- [x] **Step 1: Keep response shape compatible with existing MCP server contracts**
- [x] **Step 2: Keep policy and trace summaries in the call path**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_call
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_workspace_tool_profile
```

- [x] **Step 1: Run focused protocol checks**

Verified:

```text
fmt-check passed
clippy -p ruse-cli passed
clippy -p ruse-mcp passed
ruse-cli mcp_server_workspace_tool_profile: 5 passed
ruse-mcp mcp_server_builtin_tool_call: 1 passed
ruse-cli full crate tests passed
ruse-mcp full crate tests passed
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: prove mcp stdio builtin tool calls"
```

- [x] **Step 1: Commit M33-04**
