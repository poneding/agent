# M38-02 Prompt Argument Completions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add static completion lookup for declared MCP prompt arguments.

**Architecture:** `agent-mcp` extends prompt export registry behavior while keeping completion contracts provider-agnostic.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M38-02 spec and prompt export registry code**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add prompt argument completion lookup tests**
- [ ] **Step 2: Add undeclared argument and disabled prompt tests**
- [ ] **Step 3: Add empty completion and redaction tests**

### Task 3: Implement Prompt Completion Sources

- [ ] **Step 1: Add prompt argument completion source contract**
- [ ] **Step 2: Validate completion sources against declared arguments**
- [ ] **Step 3: Add prompt registry completion lookup API**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m38 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m38 cargo test -p agent-mcp mcp_server_prompt_completions
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp prompt argument completions"
```

- [ ] **Step 1: Commit M38-02**
