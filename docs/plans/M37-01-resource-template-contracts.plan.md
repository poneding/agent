# M37-01 Resource Template Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP resource template descriptors and registry listing.

**Architecture:** `agent-mcp` owns contracts in the resource export module.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M37-01 spec and existing resource export code**

### Task 2: Write Failing Tests

- [x] **Step 1: Add descriptor serialization test**
- [x] **Step 2: Add duplicate and disabled listing tests**
- [x] **Step 3: Add metadata validation tests**

### Task 3: Implement Contracts

- [x] **Step 1: Add `McpResourceTemplateExport`**
- [x] **Step 2: Add registry insertion and listing methods**
- [x] **Step 3: Reuse existing redaction and bounds helpers**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p agent-mcp mcp_server_resource_template_contracts
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p agent-mcp mcp_server_resource_template_contracts
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp resource template contracts"
```

- [x] **Step 1: Commit M37-01**
