# MX01-03 Sampling Productization Implementation Plan

**Goal:** Add sampling approval/review contracts and default-deny tool-loop semantics.

### Task 1: Add Approval Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add task metadata, review summary, approval request, and decision DTOs**
- [x] **Step 2: Add approval report builder**
- [x] **Step 3: Export contracts**

### Task 2: Add Guardrails

**Files:**
- Modify: `crates/agent-mcp/src/client_sampling.rs`

- [x] **Step 1: Add default-deny tool-loop policy contract**
- [x] **Step 2: Keep request validation and approval diagnostics redacted**

### Task 3: Add Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_sampling_productization.rs`

- [x] **Step 1: Cover serde shape**
- [x] **Step 2: Cover task metadata**
- [x] **Step 3: Cover default-deny tool-loop behavior**
- [x] **Step 4: Cover redaction**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-mcp mcp_sampling_productization
```
