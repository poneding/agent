# M45-04 HTTP Gateway Tool Call Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute HTTP-backed MCP tools through the existing gateway and local registry path.

**Architecture:** Implement `McpGatewayClient` for `McpHttpClient<T>` without changing gateway ownership.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-04-http-gateway-tool-call.spec.md`
- Read: `crates/agent-mcp/src/gateway.rs`
- Read: `crates/agent-mcp/tests/mcp_gateway_stdio_tool_call.rs`

- [ ] **Step 1: Review stdio gateway execution coverage**

### Task 2: Write Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_gateway_http_tool_call.rs`

- [ ] **Step 1: Add gateway discover/register/execute test**
- [ ] **Step 2: Add redaction and output-bound assertions**

### Task 3: Implement Tool Calls

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Implement `McpGatewayClient` for `McpHttpClient<T>`**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_gateway_http_tool_call
CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Commit M45-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: execute gateway tools over mcp http"
```
