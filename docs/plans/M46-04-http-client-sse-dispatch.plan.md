# M46-04 HTTP Client SSE Dispatch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `McpHttpClient` consume matching JSON-RPC frames from SSE response mode.

**Architecture:** Existing `McpClient` and `McpGatewayClient` implementations remain the public boundary.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review transport and response-mode contracts**

### Task 2: Write Tests

- [x] **Step 1: Add SSE dispatch tests for initialize/list/call**

### Task 3: Implement Dispatch

- [x] **Step 1: Route response modes through matching-id extraction**

### Task 4: Verify

- [x] **Step 1: Run M46 and M45 focused HTTP tests**

### Task 5: Commit

- [x] **Step 1: Commit M46-04**

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_dispatch` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
