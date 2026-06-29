# M40-01 Sampling Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP sampling contracts and client sampling capability metadata.

**Architecture:** `agent-mcp` owns protocol contracts in a focused sampling module exported from `lib.rs`; no UI or live model dependency is introduced.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M40-01-sampling-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_roots.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Review the spec and existing client capability patterns**

Run:

```bash
sed -n '1,240p' docs/specs/M40-01-sampling-contracts.spec.md
sed -n '1,140p' crates/agent-mcp/src/client_roots.rs
sed -n '1,120p' crates/agent-mcp/src/lib.rs
```

Expected: roots contracts show the local serde and export style to mirror.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_contracts.rs`

- [x] **Step 1: Add capability, request, result, and content serde tests**

Add tests that assert:

```rust
assert_eq!(
    serde_json::to_value(McpClientCapabilities::with_sampling(
        McpClientSamplingCapability::basic()
    ))
    .expect("json"),
    serde_json::json!({ "sampling": {} })
);
```

Add a basic request assertion with `modelPreferences`, `systemPrompt`, and `maxTokens`, and a result assertion with `role`, `content`, `model`, and `stopReason`.

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
```

Expected: tests fail because sampling contracts do not exist yet.

### Task 3: Implement Contracts

**Files:**
- Create: `crates/agent-mcp/src/client_sampling.rs`
- Modify: `crates/agent-mcp/src/client_roots.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add sampling structs and serde attributes**

Implement the public contracts from the spec with `Serialize`, `Deserialize`, `Clone`, `Debug`, and `PartialEq` where appropriate. Use `#[serde(rename = "mimeType")]`, `#[serde(rename = "modelPreferences")]`, `#[serde(rename = "systemPrompt")]`, `#[serde(rename = "includeContext")]`, `#[serde(rename = "maxTokens")]`, and `#[serde(rename = "stopSequences")]`.

- [x] **Step 2: Extend `McpClientCapabilities`**

Add:

```rust
pub struct McpClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<McpClientRootsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<McpClientSamplingCapability>,
}
```

Add `with_sampling` and `with_roots_and_sampling` constructors without breaking existing root tests.

- [x] **Step 3: Export contracts**

Export the new module and public types from `crates/agent-mcp/src/lib.rs`.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M40-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp sampling contracts"
```

Expected: commit succeeds with source, tests, and docs together.
