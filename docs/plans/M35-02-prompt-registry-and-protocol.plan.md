# M35-02 Prompt Registry and Protocol Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add MCP prompt registry behavior and `prompts/list`/`prompts/get` JSON-RPC handling.

**Architecture:** `ruse-mcp` owns the registry and protocol methods.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M35-02 spec and current server fixture/session code**

### Task 2: Write Failing Tests

- [x] **Step 1: Add registry visibility tests**
- [x] **Step 2: Add prompts/list stdio test**
- [x] **Step 3: Add prompts/get stdio test**

### Task 3: Implement Registry and Protocol

- [x] **Step 1: Add prompt export registry**
- [x] **Step 2: Extend server fixture and method parser**
- [x] **Step 3: Route prompt JSON-RPC frames**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp mcp_server_prompt_protocol
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp mcp_server_prompt_protocol
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp prompt protocol runtime"
```

- [x] **Step 1: Commit M35-02**
