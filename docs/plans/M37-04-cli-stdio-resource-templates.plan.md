# M37-04 CLI Stdio Resource Templates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify resource templates through `ruse-cli mcp server stdio`.

**Architecture:** `ruse-cli` stays an IO/bootstrap layer; `ruse-mcp` owns protocol behavior.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M37-04 spec and CLI stdio tests**

### Task 2: Write Failing Tests

- [x] **Step 1: Add `resources/templates/list` e2e test**
- [x] **Step 2: Add template-backed `resources/read` e2e test**
- [x] **Step 3: Add invalid concrete URI and traversal e2e tests**
- [x] **Step 4: Add protocol-only stdout binary test**

### Task 3: Implement CLI Wiring

- [x] **Step 1: Route `resources/templates/list` in stdio session**
- [x] **Step 2: Ensure profile runtime passes templates to sessions**
- [x] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p ruse-cli mcp_server_resource_templates
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p ruse-cli mcp_server_resource_templates
CARGO_TARGET_DIR=target/codex-m37 cargo test -p ruse-mcp mcp_server_contracts
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio resource templates"
```

- [x] **Step 1: Commit M37-04**
