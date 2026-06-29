# M41-02 Sampling Model Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute safe MCP sampling requests through an `agent-model::ModelProvider`.

**Architecture:** Extend `agent-core::mcp_sampling` with `McpModelSamplingProvider`, backed by `Arc<dyn ModelProvider>`, and implement `agent_mcp::McpSamplingProvider`.

**Tech Stack:** Rust 2024 edition, async-trait, agent-core, agent-model, agent-mcp, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-02-sampling-model-provider.spec.md`
- Read: `crates/agent-model/src/stream.rs`
- Read: `crates/agent-model/src/fake.rs`
- Read: `crates/agent-core/src/mcp_sampling.rs`

- [x] **Step 1: Review stream and mapping contracts**

Run:

```bash
sed -n '1,180p' crates/agent-model/src/stream.rs
sed -n '1,220p' crates/agent-core/src/mcp_sampling.rs
```

Expected: model stream events can be collected and mapping is already available.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-core/tests/mcp_sampling_model_provider.rs`

- [x] **Step 1: Add provider tests**

Add tests named:

```rust
mcp_sampling_model_provider_returns_fake_model_text
mcp_sampling_model_provider_returns_empty_text_for_empty_deltas
mcp_sampling_model_provider_rejects_invalid_request_before_streaming
mcp_sampling_model_provider_debug_does_not_retain_prompt_text
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_provider
```

Expected: tests fail because `McpModelSamplingProvider` does not exist.

### Task 3: Implement Provider

**Files:**
- Modify: `crates/agent-core/src/mcp_sampling.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add provider struct and constructor**

Store `Arc<dyn ModelProvider>`, bridge config, and deterministic IDs.

- [x] **Step 2: Implement `McpSamplingProvider`**

Build a `ModelRequest`, stream the provider, concatenate delta text, and return `McpCreateMessageResult`.

- [x] **Step 3: Convert failures safely**

Map bridge errors to `McpError::InvalidServerRequest` or `McpError::Client` without prompt text.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p agent-core --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_provider
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M41-02**

Run:

```bash
git add crates/agent-core docs
git commit -m "feat: execute mcp sampling through model provider"
```
