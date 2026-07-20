# M44-03 Tool Call Redaction and Limits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden MCP stdio tool-call diagnostics and local argument redaction.

**Architecture:** Keep redaction in `ruse-mcp`; preserve typed errors and gateway output bounding.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo focused tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-03-tool-call-redaction-limits.spec.md`
- Read: `crates/ruse-mcp/src/client_stdio.rs`
- Read: `crates/ruse-mcp/src/gateway.rs`

- [x] **Step 1: Review existing redaction helpers**

### Task 2: Add Negative Tests

**Files:**
- Modify: `crates/ruse-mcp/tests/mcp_client_stdio_tool_call.rs`
- Modify: `crates/ruse-mcp/tests/mcp_gateway_stdio_tool_call.rs`

- [x] **Step 1: Add failure and redaction regressions**

### Task 3: Implement Hardening

**Files:**
- Modify: `crates/ruse-mcp/src/client_stdio.rs`
- Modify: `crates/ruse-mcp/src/gateway.rs`

- [x] **Step 1: Fix any leaking diagnostics**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_client_stdio_tool_call
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_gateway_stdio_tool_call
```

### Task 5: Commit

- [x] **Step 1: Commit M44-03**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: harden mcp stdio tool call diagnostics"
```
