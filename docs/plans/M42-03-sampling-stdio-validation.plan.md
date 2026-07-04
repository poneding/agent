# M42-03 Sampling Stdio Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add validation-before-provider coverage for sampling stdio dispatch.

**Architecture:** Use the M42 dispatcher with fixture providers whose call count can be asserted.

**Tech Stack:** Rust 2024 edition, agent-mcp tests, serde JSON.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-03-sampling-stdio-validation.spec.md`
- Read: `crates/agent-mcp/src/client_sampling.rs`
- Read: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [x] **Step 1: Review validation behavior**

Expected: M40 default validation is still the source of truth.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_stdio_validation.rs`

- [x] **Step 1: Add validation tests**

Add tests named:

```rust
mcp_sampling_stdio_validation_rejects_tools_before_provider
mcp_sampling_stdio_validation_rejects_context_before_provider
mcp_sampling_stdio_validation_rejects_invalid_tokens_before_provider
mcp_sampling_stdio_validation_rejects_malformed_params_before_provider
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_validation
```

Expected: tests fail until dispatcher validation behavior is explicit and covered.

### Task 3: Implement or Tighten Validation

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [x] **Step 1: Ensure invalid params fail before provider**
- [x] **Step 2: Ensure sampling validation failures fail before provider**
- [x] **Step 3: Keep JSON-RPC error shape stable**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_validation
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M42-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: validate mcp sampling stdio dispatch"
```
