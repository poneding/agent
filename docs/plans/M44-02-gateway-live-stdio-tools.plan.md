# M44-02 Gateway Live Stdio Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify gateway discover/register/execute works with stdio-backed MCP clients.

**Architecture:** Reuse `McpGateway` and `McpStdioClient`; avoid new gateway protocol branches.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-02-gateway-live-stdio-tools.spec.md`
- Read: `crates/ruse-mcp/src/gateway.rs`
- Read: `crates/ruse-mcp/src/client_stdio.rs`

- [x] **Step 1: Review gateway registration and execution path**

### Task 2: Write Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_gateway_stdio_tool_call.rs`

- [x] **Step 1: Add gateway stdio execution tests**

### Task 3: Implement Fixes If Needed

**Files:**
- Modify: `crates/ruse-mcp/src/gateway.rs`
- Modify: `crates/ruse-mcp/src/client_stdio.rs`

- [x] **Step 1: Keep gateway compatible with stdio client**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_gateway_stdio_tool_call
```

### Task 5: Commit

- [x] **Step 1: Commit M44-02**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: execute gateway tools over mcp stdio"
```
