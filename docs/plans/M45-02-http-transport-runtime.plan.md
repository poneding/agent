# M45-02 HTTP Transport Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe, testable HTTP transport boundary for MCP client requests.

**Architecture:** Extend `agent-mcp::client_http` with transport traits and live/fixture implementations. Keep higher-level MCP operations in later slices.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, HTTP client dependency chosen to match workspace patterns, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-02-http-transport-runtime.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/config.rs`

- [x] **Step 1: Review M45-01 contracts and config validation**

### Task 2: Write Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_transport.rs`

- [x] **Step 1: Add fixture transport tests**
- [x] **Step 2: Add config and redaction tests**

### Task 3: Implement Transport

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/Cargo.toml`

- [x] **Step 1: Add transport trait and fixture transport**
- [x] **Step 2: Add live transport constructor and safe diagnostics**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_transport
CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Commit M45-02**

Run:

```bash
git add Cargo.toml Cargo.lock crates/agent-mcp docs
git commit -m "feat: add mcp http client transport"
```
