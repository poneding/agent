# M45-03 HTTP McpClient Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `McpClient` for HTTP-backed MCP servers.

**Architecture:** Add `McpHttpClient<T>` in `agent-mcp::client_http`, reusing existing MCP typed result contracts and health check flows.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-03-http-mcp-client-discovery.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/client.rs`

- [x] **Step 1: Review stdio `McpClient` mapping**

### Task 2: Write Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_mcp_client.rs`

- [x] **Step 1: Add initialize/list_tools fixture tests**
- [x] **Step 2: Add health and redaction tests**

### Task 3: Implement Client

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add `McpHttpClient<T>` session state**
- [x] **Step 2: Implement `McpClient` and root provider**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_mcp_client
CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Commit M45-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http client discovery"
```
