# M36-02 Prompt Template Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render prompt messages from explicit argument maps.

**Architecture:** `ruse-mcp::server_prompt` validates and renders templates; stdio protocol remains a thin caller.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M36-02 spec and M36-01 contracts**

### Task 2: Write Failing Tests

- [x] **Step 1: Add successful substitution tests**
- [x] **Step 2: Add missing and unknown argument tests**
- [x] **Step 3: Add undeclared template reference tests**
- [x] **Step 4: Add redaction and bounds tests**

### Task 3: Implement Rendering

- [x] **Step 1: Add argument validation path**
- [x] **Step 2: Add template reference scanner**
- [x] **Step 3: Render `McpPromptGetResult` with arguments**
- [x] **Step 4: Route registry get through render path**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_prompt_rendering
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_prompt_rendering
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: render mcp prompt arguments"
```

- [x] **Step 1: Commit M36-02**
