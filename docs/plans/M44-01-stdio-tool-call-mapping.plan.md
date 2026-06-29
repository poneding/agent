# M44-01 Stdio Tool Call Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `McpStdioClient` `tools/call` mapping through `McpGatewayClient`.

**Architecture:** Extend `agent-mcp::client_stdio`; keep JSON-RPC mapping out of CLI/Desktop.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-01-stdio-tool-call-mapping.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/client.rs`
- Read: `crates/agent-mcp/src/tool.rs`

- [ ] **Step 1: Review typed call-tool contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_tool_call.rs`

- [ ] **Step 1: Add fixture stdio tool-call tests**

Add tests for request shape/result mapping and uninitialized or mismatched server calls.

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call
```

### Task 3: Implement Mapping

**Files:**
- Modify: `crates/agent-mcp/src/client_stdio.rs`

- [ ] **Step 1: Implement `McpGatewayClient` for `McpStdioClient<T>`**
- [ ] **Step 2: Add `tools/call` params and result mapping**
- [ ] **Step 3: Keep errors bounded and redacted**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call
```

### Task 5: Commit

- [ ] **Step 1: Commit M44-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: map mcp stdio tool calls"
```
