# M37-03 Manifest Resource Templates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load MCP resource templates from export manifests and expose safe inspection metadata.

**Architecture:** `agent-mcp` manifest code owns parsing, validation, profile construction, and inspection.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M37-03 spec and manifest resource code**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add manifest parse and serialization tests**
- [ ] **Step 2: Add runtime builder tests**
- [ ] **Step 3: Add duplicate, disabled, and inspection tests**

### Task 3: Implement Manifest Templates

- [ ] **Step 1: Add manifest config structs**
- [ ] **Step 2: Build resource templates into profile runtime**
- [ ] **Step 3: Include template summaries in inspection**
- [ ] **Step 4: Add preflight diagnostics for unsafe path templates**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p agent-mcp mcp_server_manifest_resource_templates
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp manifest resource templates"
```

- [ ] **Step 1: Commit M37-03**
