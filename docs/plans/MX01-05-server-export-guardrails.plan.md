# MX01-05 Server Export Expansion Guardrails Implementation Plan

**Goal:** Add manifest guardrails and policy previews for MCP server tool export expansion.

### Task 1: Add Manifest Capability Declarations

**Files:**
- Modify: `crates/agent-mcp/src/server_manifest.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add tool export capability enum and side-effecting declaration field**
- [x] **Step 2: Validate enabled dangerous declarations as default-deny**
- [x] **Step 3: Preserve read-only tool behavior**

### Task 2: Add Policy Preview Contracts

**Files:**
- Modify: `crates/agent-mcp/src/server_manifest.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add policy preview DTOs**
- [x] **Step 2: Add preview builder for selected profiles**
- [x] **Step 3: Redact and bound preview diagnostics**

### Task 3: Add Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_server_export_guardrails.rs`
- Modify: existing manifest tests as needed for new struct fields

- [x] **Step 1: Cover read-only export preview**
- [x] **Step 2: Cover dangerous command/edit/git-write/browser/secret/network rejection**
- [x] **Step 3: Cover disabled dangerous declaration preview**
- [x] **Step 4: Cover redaction**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-mcp mcp_server_export_guardrails
```
