# M43-01 Client Stdio Codec Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed client-side MCP stdio request/response frame helpers.

**Architecture:** Extend `agent-mcp` with a codec module that reuses existing JSON-RPC contracts.

**Tech Stack:** Rust 2024 edition, serde JSON, agent-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-01-client-stdio-codec.spec.md`
- Read: `crates/agent-mcp/src/server.rs`
- Read: `crates/agent-mcp/src/client_sampling_stdio.rs`

- [x] **Step 1: Review JSON-RPC contracts**

Expected: codec can reuse existing envelope types.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_codec.rs`

- [x] **Step 1: Add codec tests**

Add tests named:

```rust
mcp_client_stdio_codec_renders_initialize_request_frame
mcp_client_stdio_codec_parses_success_response_frame
mcp_client_stdio_codec_parses_failure_response_frame
mcp_client_stdio_codec_rejects_notifications_as_responses
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_codec
```

Expected: tests fail until the codec exists.

### Task 3: Implement Codec

**Files:**
- Create: `crates/agent-mcp/src/client_stdio.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add request frame renderer**
- [x] **Step 2: Add response frame parser**
- [x] **Step 3: Export public contracts**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_codec
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M43-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp client stdio codec"
```
