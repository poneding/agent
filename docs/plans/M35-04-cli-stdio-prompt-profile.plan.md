# M35-04 CLI Stdio Prompt Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve manifest prompt exports through `ruse-cli mcp server stdio`.

**Architecture:** `ruse-cli` wires profile runtime data into `ruse-mcp` sessions.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M35-04 spec and CLI stdio bootstrap**

### Task 2: Write Failing Tests

- [x] **Step 1: Add prompt profile CLI parse/startup test**
- [x] **Step 2: Add prompts/list e2e test**
- [x] **Step 3: Add prompts/get e2e test**

### Task 3: Implement CLI Wiring

- [x] **Step 1: Pass prompt exports into MCP session fixture**
- [x] **Step 2: Set prompt capability when prompt exports exist**
- [x] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-cli mcp_server_prompt_profile
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-cli mcp_server_prompt_profile
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio prompt profiles"
```

- [x] **Step 1: Commit M35-04**
