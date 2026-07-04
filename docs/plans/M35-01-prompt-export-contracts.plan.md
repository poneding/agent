# M35-01 Prompt Export Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP prompt export contracts in `agent-mcp`.

**Architecture:** `agent-mcp` owns prompt export contracts; no CLI logic in this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M35-01 spec and current MCP server contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_server_prompt_export.rs`

- [x] **Step 1: Add prompt descriptor and result serialization tests**
- [x] **Step 2: Add invalid prompt contract tests**
- [x] **Step 3: Add redaction and bounds tests**

### Task 3: Implement Contracts

**Files:**
- Create: `crates/agent-mcp/src/server_prompt.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add prompt export data types**
- [x] **Step 2: Add constructors and validation**
- [x] **Step 3: Export public contracts**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_export
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_export
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp prompt export contracts"
```

- [x] **Step 1: Commit M35-01**
