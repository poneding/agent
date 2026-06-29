# M43-04 Client Stdio McpClient Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `McpClient` over M43 stdio sessions for initialize and tools/list.

**Architecture:** Keep typed MCP operation mapping in `agent-mcp`; keep CLI/Desktop unaware of JSON-RPC frames.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-04-client-stdio-mcp-client.spec.md`
- Read: `crates/agent-mcp/src/client.rs`
- Read: `crates/agent-mcp/src/tool.rs`
- Read: `crates/agent-mcp/src/client_stdio.rs`

- [ ] **Step 1: Review typed client contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_mcp_client.rs`

- [ ] **Step 1: Add `McpClient` integration tests**

Add tests for initialize, tools/list, health, and invalid response shapes.

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_mcp_client
```

### Task 3: Implement Stdio McpClient

**Files:**
- Modify: `crates/agent-mcp/src/client_stdio.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add stdio client wrapper**
- [ ] **Step 2: Add initialize mapping**
- [ ] **Step 3: Add tools/list mapping**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_mcp_client
```

### Task 5: Commit

- [ ] **Step 1: Commit M43-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: implement mcp client over stdio"
```
