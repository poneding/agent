# M40-04 Gateway Sampling Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted gateway diagnostics for MCP sampling availability.

**Architecture:** `agent-mcp::gateway` reports sampling capability metadata alongside tools and roots without invoking sampling during discovery.

**Tech Stack:** Rust 2024 edition, serde JSON, async-trait, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M40-04-gateway-sampling-diagnostics.spec.md`
- Read: `crates/agent-mcp/src/gateway.rs`
- Read: `crates/agent-mcp/tests/mcp_gateway_roots.rs`

- [ ] **Step 1: Review gateway report patterns**

Run:

```bash
sed -n '1,240p' docs/specs/M40-04-gateway-sampling-diagnostics.spec.md
sed -n '1,260p' crates/agent-mcp/src/gateway.rs
sed -n '1,220p' crates/agent-mcp/tests/mcp_gateway_roots.rs
```

Expected: roots diagnostics show the non-blocking discovery pattern to mirror.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_gateway_sampling.rs`

- [ ] **Step 1: Add gateway sampling report tests**

Add tests named:

```rust
mcp_gateway_sampling_reports_availability_for_healthy_servers
mcp_gateway_sampling_excludes_disabled_and_unhealthy_servers
mcp_gateway_sampling_errors_are_redacted_and_do_not_block_tools
mcp_gateway_sampling_report_serializes_stable_shape
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_gateway_sampling
```

Expected: tests fail because gateway sampling descriptors do not exist yet.

### Task 3: Implement Gateway Diagnostics

**Files:**
- Modify: `crates/agent-mcp/src/gateway.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add `McpGatewaySamplingDescriptor`**

Add a serializable descriptor with server ID, availability, `tools_supported`, `context_supported`, and a bounded redacted message.

- [ ] **Step 2: Populate report descriptors**

During gateway discovery, add descriptors only for enabled healthy servers whose client capabilities include sampling. Sort descriptors by server ID and do not call `create_message`.

- [ ] **Step 3: Add redacted diagnostics for unavailable paths**

When a sampling capability is absent or rejected by safe-subset policy, add a diagnostic code without raw prompt content.

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_gateway_sampling
```

Expected: focused checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit M40-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp gateway sampling diagnostics"
```

Expected: commit succeeds with source, tests, and docs together.
