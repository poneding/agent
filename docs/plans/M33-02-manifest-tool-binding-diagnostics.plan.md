# M33-02 Manifest Tool Binding Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make manifest tool binding fail early with safe, useful diagnostics.

**Architecture:** `agent-mcp` validates manifest tool names against `McpServerToolExportCatalog` before building the profile.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M32 manifest validation and M33-01 catalog helper**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_server_manifest_tool_diagnostics.rs`

- [x] **Step 1: Add duplicate tool declaration test**
- [x] **Step 2: Add unknown tool diagnostic test**
- [x] **Step 3: Add disabled tool visibility test if not already covered**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-mcp mcp_server_manifest_tool_diagnostics
```

Expected: tests fail before validation hardening.

### Task 3: Implement Validation

**Files:**
- Modify: `crates/agent-mcp/src/server_manifest.rs`

- [x] **Step 1: Validate duplicate tool declarations per profile**
- [x] **Step 2: Include bounded known-name diagnostics for unknown tools**
- [x] **Step 3: Preserve existing redaction behavior**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-mcp mcp_server_manifest_tool_diagnostics
CARGO_TARGET_DIR=target/codex-m33 cargo test -p agent-mcp
```

- [x] **Step 1: Run focused and crate tests**

Verified:

```text
fmt-check passed
clippy -p agent-mcp passed
agent-mcp mcp_server_manifest_tool_diagnostics: 3 passed
agent-mcp full crate tests passed
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: harden mcp manifest tool diagnostics"
```

- [x] **Step 1: Commit M33-02**
