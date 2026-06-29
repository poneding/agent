# M35-04 CLI Stdio Prompt Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve manifest prompt exports through `agent-cli mcp server stdio`.

**Architecture:** `agent-cli` wires profile runtime data into `agent-mcp` sessions.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M35-04 spec and CLI stdio bootstrap**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add prompt profile CLI parse/startup test**
- [ ] **Step 2: Add prompts/list e2e test**
- [ ] **Step 3: Add prompts/get e2e test**

### Task 3: Implement CLI Wiring

- [ ] **Step 1: Pass prompt exports into MCP session fixture**
- [ ] **Step 2: Set prompt capability when prompt exports exist**
- [ ] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-cli mcp_server_prompt_profile
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio prompt profiles"
```

- [ ] **Step 1: Commit M35-04**
