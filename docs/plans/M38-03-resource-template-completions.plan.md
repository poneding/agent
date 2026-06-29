# M38-03 Resource Template Completions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add static completion lookup for declared MCP resource template variables.

**Architecture:** `agent-mcp` extends resource export registry behavior while keeping completion side-effect-free.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M38-03 spec and resource template registry code**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add resource template variable completion lookup tests**
- [ ] **Step 2: Add undeclared variable and disabled template tests**
- [ ] **Step 3: Add empty completion and redaction tests**

### Task 3: Implement Resource Template Completion Sources

- [ ] **Step 1: Add resource template completion source contract**
- [ ] **Step 2: Validate completion sources against URI template variables**
- [ ] **Step 3: Add resource registry completion lookup API**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m38 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m38 cargo test -p agent-mcp mcp_server_resource_template_completions
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp resource template completions"
```

- [ ] **Step 1: Commit M38-03**
