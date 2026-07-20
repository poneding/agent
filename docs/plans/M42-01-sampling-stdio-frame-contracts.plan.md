# M42-01 Sampling Stdio Frame Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed sampling stdio frame contract helpers in `ruse-mcp`.

**Architecture:** Add an `ruse-mcp` module that reuses existing JSON-RPC envelope types and sampling request contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, ruse-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-01-sampling-stdio-frame-contracts.spec.md`
- Read: `crates/ruse-mcp/src/server.rs`
- Read: `crates/ruse-mcp/src/client_sampling.rs`

- [x] **Step 1: Review frame and sampling contracts**

Run:

```bash
sed -n '1,220p' crates/ruse-mcp/src/server.rs
sed -n '360,520p' crates/ruse-mcp/src/client_sampling.rs
```

Expected: existing JSON-RPC envelopes and sampling params can be reused.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_sampling_stdio_contracts.rs`

- [x] **Step 1: Add contract tests**

Add tests named:

```rust
mcp_sampling_stdio_contracts_decode_create_message_request
mcp_sampling_stdio_contracts_notification_requires_no_response
mcp_sampling_stdio_contracts_reject_unsupported_method_safely
mcp_sampling_stdio_contracts_reject_invalid_params
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_contracts
```

Expected: tests fail because sampling stdio contracts do not exist.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/ruse-mcp/src/lib.rs`
- Create: `crates/ruse-mcp/src/client_sampling_stdio.rs`

- [x] **Step 1: Add method constant**
- [x] **Step 2: Add request parser helper**
- [x] **Step 3: Export public contracts**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_contracts
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M42-01**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp sampling stdio frame contracts"
```
