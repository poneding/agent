# M41-03 Sampling Bridge Redaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable redacted diagnostics for MCP sampling model bridge failures.

**Architecture:** Keep redaction helpers inside `agent-core::mcp_sampling` and expose only bounded diagnostic contracts.

**Tech Stack:** Rust 2024 edition, serde, agent-core tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-03-sampling-bridge-redaction.spec.md`
- Read: `crates/agent-core/src/mcp_sampling.rs`

- [ ] **Step 1: Review current bridge errors**

Run:

```bash
sed -n '1,260p' crates/agent-core/src/mcp_sampling.rs
```

Expected: bridge errors exist and need stable public diagnostics.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-core/tests/mcp_sampling_model_redaction.rs`

- [ ] **Step 1: Add redaction tests**

Add tests named:

```rust
mcp_sampling_model_redaction_hides_prompt_text_in_mapping_errors
mcp_sampling_model_redaction_hides_provider_error_secrets
mcp_sampling_model_redaction_bounds_public_messages
mcp_sampling_model_redaction_serializes_stable_diagnostic_shape
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_redaction
```

Expected: tests fail because diagnostics are not yet explicit.

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/agent-core/src/mcp_sampling.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Add `McpSamplingBridgeDiagnostic`**

Add serializable diagnostic struct with `code` and `message`.

- [ ] **Step 2: Add diagnostic conversion helpers**

Convert bridge errors and model errors into stable codes and bounded redacted messages.

- [ ] **Step 3: Export diagnostic contract**

Export from `agent-core`.

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p agent-core --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_redaction
```

Expected: focused checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit M41-03**

Run:

```bash
git add crates/agent-core docs
git commit -m "feat: redact mcp sampling model bridge diagnostics"
```
