# M38-01 Completion Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP completion request/result contracts and deterministic value bounding.

**Architecture:** `agent-mcp` owns contracts in a completion module or server-adjacent module, exported from `lib.rs`.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M38-01 spec and existing MCP server contracts**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add request and result serialization tests**
- [ ] **Step 2: Add prefix, dedupe, ordering, and bound tests**
- [ ] **Step 3: Add invalid and secret-like metadata tests**

### Task 3: Implement Contracts

- [ ] **Step 1: Add completion request/ref/argument/result contracts**
- [ ] **Step 2: Add deterministic completion value builder**
- [ ] **Step 3: Export contracts from `agent-mcp`**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m38 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m38 cargo test -p agent-mcp mcp_server_completion_contracts
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp completion contracts"
```

- [ ] **Step 1: Commit M38-01**
