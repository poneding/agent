# M37-02 Resource Template Matching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Match concrete resource URIs to declared templates and read resolved files safely.

**Architecture:** `agent-mcp` keeps matching and file resolution in the resource export path.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M37-02 spec and static resource read path**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add URI template parser and matcher tests**
- [ ] **Step 2: Add template-backed read tests**
- [ ] **Step 3: Add traversal, ambiguity, and policy denial tests**

### Task 3: Implement Matching

- [ ] **Step 1: Parse `{variable}` URI templates**
- [ ] **Step 2: Match concrete URIs deterministically**
- [ ] **Step 3: Render path templates and confine resolved paths**
- [ ] **Step 4: Reuse static resource policy and content bounding**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m37 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m37 cargo test -p agent-mcp mcp_server_resource_template_matching
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: resolve mcp resource templates"
```

- [ ] **Step 1: Commit M37-02**
