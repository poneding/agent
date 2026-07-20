# MX01-02 MCP Progress Logging Task Events Implementation Plan

**Goal:** Add typed MCP protocol event reports with bridge-safe summaries.

### Task 1: Add Event Contracts

**Files:**
- Create: `crates/ruse-mcp/src/client_events.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Define event kind, status, diagnostics, summary, and report DTOs**
- [x] **Step 2: Export event contracts**

### Task 2: Add Parser and Normalizer

**Files:**
- Modify: `crates/ruse-mcp/src/client_events.rs`

- [x] **Step 1: Parse JSON-RPC event frames**
- [x] **Step 2: Normalize progress/logging/task/elicitation methods**
- [x] **Step 3: Bound and redact summaries**

### Task 3: Add Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_protocol_events.rs`

- [x] **Step 1: Cover known event summaries**
- [x] **Step 2: Cover unknown events**
- [x] **Step 3: Cover redaction and bounded output**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p ruse-mcp mcp_client_protocol_events
```
