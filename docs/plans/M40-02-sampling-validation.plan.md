# M40-02 Sampling Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe-subset validation for MCP sampling requests.

**Architecture:** `agent-mcp` validates protocol shape and M40 capability support before any fixture or future model provider executes the request.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M40-02-sampling-validation.spec.md`
- Read: `crates/agent-mcp/src/client_sampling.rs`
- Read: `crates/agent-mcp/src/error.rs`

- [x] **Step 1: Review validation scope and existing error style**

Run:

```bash
sed -n '1,240p' docs/specs/M40-02-sampling-validation.spec.md
sed -n '1,260p' crates/agent-mcp/src/client_sampling.rs
sed -n '1,220p' crates/agent-mcp/src/error.rs
```

Expected: validation stays in `agent-mcp` and produces redacted MCP diagnostics/errors.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_validation.rs`

- [x] **Step 1: Add accepted and rejected request tests**

Add tests named:

```rust
mcp_sampling_validation_accepts_basic_text_request
mcp_sampling_validation_rejects_empty_messages_and_invalid_token_bounds
mcp_sampling_validation_rejects_unsupported_tools_and_context
mcp_sampling_validation_rejects_mixed_tool_result_content
mcp_sampling_validation_rejects_unbalanced_tool_use_results
mcp_sampling_validation_redacts_prompt_content_from_diagnostics
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation
```

Expected: tests fail because validation does not exist yet.

### Task 3: Implement Validation

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add validation policy and diagnostics**

Implement `McpSamplingValidationPolicy::default()` with conservative values:

```rust
max_tokens_limit: 4096
max_text_chars: 16_384
max_media_bytes_base64_chars: 1_048_576
allow_tools: false
allow_context: false
```

- [x] **Step 2: Add `validate_mcp_sampling_request`**

Validate message list, token bounds, context, tools, content sizes, MIME labels, and tool-result balance. Return `McpSamplingValidationReport`; callers reject provider execution when `accepted` is false. Do not include raw prompt text in diagnostic messages.

- [x] **Step 3: Export validation contracts**

Export policy, report, diagnostics, and validation function from `lib.rs`.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M40-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: validate mcp sampling requests"
```

Expected: commit succeeds with source, tests, and docs together.
