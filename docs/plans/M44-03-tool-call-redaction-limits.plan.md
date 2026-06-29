# M44-03 Tool Call Redaction and Limits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden MCP stdio tool-call diagnostics and local argument redaction.

**Architecture:** Keep redaction in `agent-mcp`; preserve typed errors and gateway output bounding.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo focused tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-03-tool-call-redaction-limits.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/gateway.rs`

- [ ] **Step 1: Review existing redaction helpers**

### Task 2: Add Negative Tests

**Files:**
- Modify: `crates/agent-mcp/tests/mcp_client_stdio_tool_call.rs`
- Modify: `crates/agent-mcp/tests/mcp_gateway_stdio_tool_call.rs`

- [ ] **Step 1: Add failure and redaction regressions**

### Task 3: Implement Hardening

**Files:**
- Modify: `crates/agent-mcp/src/client_stdio.rs`
- Modify: `crates/agent-mcp/src/gateway.rs`

- [ ] **Step 1: Fix any leaking diagnostics**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call
CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_gateway_stdio_tool_call
```

### Task 5: Commit

- [ ] **Step 1: Commit M44-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: harden mcp stdio tool call diagnostics"
```
