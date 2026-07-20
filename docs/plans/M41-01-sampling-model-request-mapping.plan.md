# M41-01 Sampling Model Request Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure MCP sampling request to `ruse-model::ModelRequest` mapping contracts.

**Architecture:** Add `ruse-mcp` as a normal `ruse-core` dependency, create `crates/ruse-core/src/mcp_sampling.rs`, and export contracts from `ruse-core/src/lib.rs`.

**Tech Stack:** Rust 2024 edition, ruse-core, ruse-mcp, ruse-model, ruse-types, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-01-sampling-model-request-mapping.spec.md`
- Read: `crates/ruse-model/src/request.rs`
- Read: `crates/ruse-mcp/src/client_sampling.rs`

- [x] **Step 1: Review mapping contracts**

Run:

```bash
sed -n '1,220p' docs/specs/M41-01-sampling-model-request-mapping.spec.md
sed -n '1,120p' crates/ruse-model/src/request.rs
```

Expected: mapping is pure and `ModelRequest.messages` is `Vec<String>`.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-core/tests/mcp_sampling_model_mapping.rs`

- [x] **Step 1: Add mapping tests**

Add tests named:

```rust
mcp_sampling_model_mapping_builds_text_model_request
mcp_sampling_model_mapping_includes_system_prompt
mcp_sampling_model_mapping_uses_local_model_over_server_hints
mcp_sampling_model_mapping_rejects_media_without_prompt_leak
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_mapping
```

Expected: tests fail because the bridge module and mapping contracts do not exist.

### Task 3: Implement Mapping Contracts

**Files:**
- Modify: `crates/ruse-core/Cargo.toml`
- Create: `crates/ruse-core/src/mcp_sampling.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add `ruse-mcp` as an `ruse-core` dependency**

Move the existing `ruse-mcp` dev-dependency into `[dependencies]`.

- [x] **Step 2: Add bridge config, IDs, and error contracts**

Implement:

```rust
pub struct McpSamplingModelBridgeConfig {
    pub provider: String,
    pub model: String,
}

pub struct McpSamplingModelBridgeIds {
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
}

pub enum McpSamplingModelBridgeError {
    InvalidRequest(String),
    UnsupportedContent(String),
    Model(String),
}
```

- [x] **Step 3: Add `build_mcp_sampling_model_request`**

Validate with `McpSamplingValidationPolicy::default()`, reject unsupported content, and create role-prefixed text messages using local provider/model config.

- [x] **Step 4: Export contracts**

Export the new module and public contracts from `ruse-core/src/lib.rs`.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p ruse-core --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_mapping
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M41-01**

Run:

```bash
git add Cargo.toml Cargo.lock crates/ruse-core docs
git commit -m "feat: map mcp sampling requests to model requests"
```
