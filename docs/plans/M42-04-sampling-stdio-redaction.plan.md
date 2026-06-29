# M42-04 Sampling Stdio Redaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redaction regressions and helpers for sampling stdio dispatch errors.

**Architecture:** Keep redaction inside `agent-mcp` sampling stdio dispatch so the public JSON-RPC surface is safe regardless of provider implementation.

**Tech Stack:** Rust 2024 edition, serde JSON, agent-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-04-sampling-stdio-redaction.spec.md`
- Read: `crates/agent-mcp/src/client_sampling_stdio.rs`
- Read: `crates/agent-mcp/src/server.rs`

- [ ] **Step 1: Review current JSON-RPC error redaction**

Expected: error rendering is centralized enough to harden.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_stdio_redaction.rs`

- [ ] **Step 1: Add redaction tests**

Add tests named:

```rust
mcp_sampling_stdio_redaction_hides_prompt_text_in_invalid_request_errors
mcp_sampling_stdio_redaction_hides_provider_error_secrets
mcp_sampling_stdio_redaction_bounds_error_messages
mcp_sampling_stdio_redaction_rejects_secret_like_method_names_safely
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_redaction
```

Expected: tests fail until redaction is explicit.

### Task 3: Implement Redaction

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [ ] **Step 1: Add bounded public error helper**
- [ ] **Step 2: Redact provider and invalid frame diagnostics**
- [ ] **Step 3: Preserve JSON-RPC error codes**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_redaction
```

Expected: focused checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit M42-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: redact mcp sampling stdio dispatch errors"
```
