# MX01-04 Desktop Bridge MCP Management Snapshot Implementation Plan

**Goal:** Add UI-safe MCP management snapshot contracts.

### Task 1: Add MCP Snapshot Contracts

**Files:**
- Create: `crates/agent-mcp/src/management_snapshot.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Define server snapshot, transport, stream, auth, diagnostic, root, sampling, and extension DTOs**
- [x] **Step 2: Add snapshot builder helpers**
- [x] **Step 3: Export contracts**

### Task 2: Add Bridge DTOs

**Files:**
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`

- [x] **Step 1: Define bridge-safe MCP management snapshot DTOs**
- [x] **Step 2: Reuse existing bridge validation/redaction style**

### Task 3: Add Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_management_snapshot.rs`
- Create: `crates/agent-ui-bridge/tests/mcp_management_snapshot.rs`

- [x] **Step 1: Cover MCP snapshot redaction/bounding**
- [x] **Step 2: Cover bridge snapshot serialization**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-mcp mcp_management_snapshot
cargo test -p agent-ui-bridge mcp_management_snapshot
```
