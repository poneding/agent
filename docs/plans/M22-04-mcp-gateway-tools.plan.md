# M22-04 MCP Gateway Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register and execute MCP tools through the normal local tool registry, policy, audit, and comprehensive turn path.

**Architecture:** `ruse-mcp` owns gateway discovery and remote calls. `ruse-tool` owns local executable tool contracts. `ruse-core` wires gateway tools into comprehensive turn resources.

**Tech Stack:** Rust 2024 edition, async traits, fixture MCP client, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-04-mcp-gateway-tools.spec.md`
- Read: `crates/ruse-mcp/src/client.rs`
- Read: `crates/ruse-mcp/src/tool_mapping.rs`
- Read: `crates/ruse-mcp/src/policy.rs`
- Read: `crates/ruse-core/src/tool_loop.rs`

- [x] **Step 1: Review MCP gateway scope**

Run:

```bash
sed -n '1,280p' docs/specs/M22-04-mcp-gateway-tools.spec.md
```

Expected: gateway discovery, policy, execution, and failure requirements are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_gateway.rs`
- Create: `crates/ruse-core/tests/mcp_gateway_tool_loop.rs`

- [x] **Step 1: Add discovery tests**

Cover disabled, healthy, unhealthy, duplicate local names, timeout, and source metadata.

- [x] **Step 2: Add execution tests**

Cover allowed call, denied call, remote error, oversized response, and redacted input/output traces.

- [x] **Step 3: Add core integration test**

Prove a comprehensive turn can execute an MCP-backed tool through `ToolLoop` and record normal tool events.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-mcp mcp_gateway
cargo test -p ruse-core mcp_gateway_tool_loop
```

Expected: tests fail because gateway contracts are missing.

### Task 3: Implement MCP Gateway

**Files:**
- Create: `crates/ruse-mcp/src/gateway.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`
- Modify: `crates/ruse-mcp/src/client.rs`
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Add gateway client and report contracts**

Implement `McpGatewayClient`, `McpGateway`, `McpGatewayReport`, `McpGatewayTool`, and `McpToolCallRequest`.

- [x] **Step 2: Register local tools**

Convert healthy descriptors into `ruse-tool` entries named `mcp.<server>.<tool>` with source metadata.

- [x] **Step 3: Enforce policy**

Ensure remote calls use existing policy requests and denied calls never reach the MCP client.

- [x] **Step 4: Add execution and observation conversion**

Bound remote responses and convert them to normal tool outputs and observations.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-mcp/tests/mcp_gateway.rs`
- Read: `crates/ruse-core/tests/mcp_gateway_tool_loop.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-mcp mcp_gateway
cargo test -p ruse-core mcp_gateway_tool_loop
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M22-04-mcp-gateway-tools.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M22-04 complete**

Advance active implementation focus to M22-05.

- [x] **Step 2: Commit M22-04**

Run:

```bash
git add crates/ruse-mcp crates/ruse-core docs Cargo.lock
git commit -m "feat: add mcp gateway tools"
```

Expected: commit succeeds.
