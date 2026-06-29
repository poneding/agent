# M42-01 Sampling Stdio Frame Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed sampling stdio frame contract helpers in `agent-mcp`.

**Architecture:** Add an `agent-mcp` module that reuses existing JSON-RPC envelope types and sampling request contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, agent-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-01-sampling-stdio-frame-contracts.spec.md`
- Read: `crates/agent-mcp/src/server.rs`
- Read: `crates/agent-mcp/src/client_sampling.rs`

- [ ] **Step 1: Review frame and sampling contracts**

Run:

```bash
sed -n '1,220p' crates/agent-mcp/src/server.rs
sed -n '360,520p' crates/agent-mcp/src/client_sampling.rs
```

Expected: existing JSON-RPC envelopes and sampling params can be reused.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_stdio_contracts.rs`

- [ ] **Step 1: Add contract tests**

Add tests named:

```rust
mcp_sampling_stdio_contracts_decode_create_message_request
mcp_sampling_stdio_contracts_notification_requires_no_response
mcp_sampling_stdio_contracts_reject_unsupported_method_safely
mcp_sampling_stdio_contracts_reject_invalid_params
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_contracts
```

Expected: tests fail because sampling stdio contracts do not exist.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-mcp/src/lib.rs`
- Create: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [ ] **Step 1: Add method constant**
- [ ] **Step 2: Add request parser helper**
- [ ] **Step 3: Export public contracts**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_contracts
```

Expected: focused checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit M42-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp sampling stdio frame contracts"
```
