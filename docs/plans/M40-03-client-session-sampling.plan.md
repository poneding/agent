# M40-03 Client Session Sampling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic fixture and provider behavior for MCP client sampling requests.

**Architecture:** `agent-mcp` extends client fixture boundaries with `McpSamplingProvider`; sampling remains separate from gateway tool calls.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M40-03-client-session-sampling.spec.md`
- Read: `crates/agent-mcp/src/client.rs`
- Read: `crates/agent-mcp/src/client_sampling.rs`

- [x] **Step 1: Review fixture client patterns**

Run:

```bash
sed -n '1,260p' docs/specs/M40-03-client-session-sampling.spec.md
sed -n '1,280p' crates/agent-mcp/src/client.rs
```

Expected: fixture client already has roots and tool result patterns to mirror.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_session_sampling.rs`

- [x] **Step 1: Add fixture sampling behavior tests**

Add tests named:

```rust
mcp_client_session_sampling_fixture_returns_configured_result
mcp_client_session_sampling_fixture_rejects_unknown_server
mcp_client_session_sampling_fixture_returns_configured_error
mcp_client_session_sampling_disabled_config_rejects_before_provider
mcp_client_session_sampling_validates_request_before_provider
mcp_client_session_sampling_tracks_calls_without_prompt_text
mcp_client_session_sampling_capabilities_combine_roots_and_sampling
```

- [x] **Step 2: Run focused tests and verify failure**

Verified failing before implementation:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
```

Expected: tests fail because the provider trait and fixture methods do not exist yet.

### Task 3: Implement Provider and Fixture Behavior

**Files:**
- Modify: `crates/agent-mcp/src/client.rs`
- Modify: `crates/agent-mcp/src/client_sampling.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add `McpSamplingProvider` and helper function**

Add the async trait and `create_mcp_sampling_message` helper. The helper rejects disabled configs before invoking the provider and validates the request with `McpSamplingValidationPolicy::default()`.

- [x] **Step 2: Extend `FixtureMcpClient`**

Add sampling result/error maps keyed by server ID, sampling call counts keyed by server ID, and builder methods:

```rust
pub fn with_sampling_result(
    self,
    server_id: McpServerId,
    result: McpCreateMessageResult,
) -> Self

pub fn with_sampling_error(
    self,
    server_id: McpServerId,
    error: McpError,
) -> Self
```

- [x] **Step 3: Implement sampling provider for the fixture**

Return configured results, configured errors, or `McpError::ServerNotFound` for missing servers. Increment call counts without storing prompt content.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M40-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp client sampling fixture"
```

Expected: commit succeeds with source, tests, and docs together.
