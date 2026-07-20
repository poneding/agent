# MX01-01 Reentrant Client Callbacks Implementation Plan

**Goal:** Add fixture-grade typed callback contracts and safe callback dispatch reports.

### Task 1: Add Contracts

**Files:**
- Create: `crates/ruse-mcp/src/client_callbacks.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Define callback transport, phase, outcome, diagnostics, and report DTOs**
- [x] **Step 2: Export callback contracts**

### Task 2: Add Dispatcher

**Files:**
- Modify: `crates/ruse-mcp/src/client_callbacks.rs`

- [x] **Step 1: Parse JSON-RPC callback frames**
- [x] **Step 2: Handle `roots/list` through the root provider**
- [x] **Step 3: Handle `sampling/createMessage` through sampling validation/provider**
- [x] **Step 4: Reject unknown callbacks safely**

### Task 3: Add Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_reentrant_callbacks.rs`

- [x] **Step 1: Cover roots/list callback**
- [x] **Step 2: Cover sampling callback**
- [x] **Step 3: Cover unknown callback**
- [x] **Step 4: Cover disabled/unsafe rejection and redaction**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p ruse-mcp mcp_client_reentrant_callbacks
```
