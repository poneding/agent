# M36-01 Prompt Argument Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed prompt argument contracts.

**Architecture:** `agent-mcp::server_prompt` owns MCP prompt argument contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M36-01 spec and existing prompt export contracts**

### Task 2: Write Failing Tests

- [x] **Step 1: Add argument descriptor serialization tests**
- [x] **Step 2: Add get params serialization tests**
- [x] **Step 3: Add invalid argument metadata tests**

### Task 3: Implement Contracts

- [x] **Step 1: Add `McpPromptArgument`**
- [x] **Step 2: Add descriptor arguments field**
- [x] **Step 3: Add `McpPromptGetParams`**
- [x] **Step 4: Export public contracts**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p agent-mcp mcp_server_prompt_arguments
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p agent-mcp mcp_server_prompt_arguments
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp prompt argument contracts"
```

- [x] **Step 1: Commit M36-01**
