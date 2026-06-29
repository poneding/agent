# M33-04 Stdio Tool Call E2E Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify manifest-exported built-in tools work over MCP stdio list and call frames.

**Architecture:** Tests exercise CLI stdio IO and MCP session behavior through public contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review existing stdio command tests and server session tests**

### Task 2: Write E2E Tests

**Files:**
- Modify: `crates/agent-cli/tests/mcp_server_workspace_tool_profile.rs`
- Create: `crates/agent-mcp/tests/mcp_server_builtin_tool_call.rs`

- [ ] **Step 1: Add initialize/list/call fixture**
- [ ] **Step 2: Add non-exported tool call failure fixture**
- [ ] **Step 3: Assert stdout is protocol-only**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-cli mcp_server_workspace_tool_profile
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-mcp mcp_server_builtin_tool_call
```

Expected: tests fail until prior slices are wired.

### Task 3: Implement Minimal Fixes

**Files:**
- Modify only runtime files needed by failing tests.

- [ ] **Step 1: Keep response shape compatible with existing MCP server contracts**
- [ ] **Step 2: Keep policy and trace summaries in the call path**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-mcp mcp_server_builtin_tool_call
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-cli mcp_server_workspace_tool_profile
```

- [ ] **Step 1: Run focused protocol checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: prove mcp stdio builtin tool calls"
```

- [ ] **Step 1: Commit M33-04**
