# M42-02 Sampling Stdio Dispatcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dispatch valid sampling stdio frames through `McpSamplingProvider`.

**Architecture:** Extend the M42-01 `agent-mcp` module with an async dispatcher that calls `create_mcp_sampling_message` and renders JSON-RPC frames.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-02-sampling-stdio-dispatcher.spec.md`
- Read: `crates/agent-mcp/src/client.rs`
- Read: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [ ] **Step 1: Review provider helper and M42-01 contracts**

Expected: dispatcher can reuse `create_mcp_sampling_message`.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_stdio_dispatch.rs`

- [ ] **Step 1: Add dispatcher tests**

Add tests named:

```rust
mcp_sampling_stdio_dispatch_returns_success_frame
mcp_sampling_stdio_dispatch_preserves_numeric_ids
mcp_sampling_stdio_dispatch_returns_empty_for_notification
mcp_sampling_stdio_dispatch_rejects_disabled_config_before_provider
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_dispatch
```

Expected: tests fail until the dispatcher exists.

### Task 3: Implement Dispatcher

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling_stdio.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add async frame dispatcher**
- [ ] **Step 2: Render success and failure frames**
- [ ] **Step 3: Export dispatcher**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_dispatch
```

Expected: focused checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit M42-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: dispatch mcp sampling stdio frames"
```
