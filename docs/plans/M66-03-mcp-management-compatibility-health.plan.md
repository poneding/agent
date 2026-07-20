# M66-03 MCP Management and Compatibility Health Plan

> **For agentic workers:** Keep M66-03 as product health hardening. Do not add OAuth, new protocol methods, or live startup side effects.

**Goal:** Add a typed MCP status health report and expose it through existing status CLI output.

**Architecture:** `ruse-mcp` owns MCP status health contracts. `ruse-cli` renders the report. Bridge/support integrations remain future slices unless required by tests.

---

### Task 1: Confirm Baseline

**Files:**
- Read: `crates/ruse-mcp/src/server_status.rs`
- Read: `crates/ruse-mcp/src/management_snapshot.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/mcp_server_commands.rs`

- [x] **Step 1: Inspect status and management snapshots**

Existing MCP status summaries and management snapshots are typed, redacted, bounded, and already bridge-safe.

### Task 2: Add MCP Status Health Report

**Files:**
- Modify: `crates/ruse-mcp/src/server_status.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`
- Add/modify tests under `crates/ruse-mcp/tests/`

- [x] **Step 1: Define typed report**

Add report status, diagnostics, and next-action ids derived from `McpServerStatusSummary`.

- [x] **Step 2: Add compatibility diagnostics**

Detect exported tools/resources when matching capabilities are disabled.

- [x] **Step 3: Preserve bounds and redaction**

Limit diagnostics and sanitize secret-like text.

### Task 3: Expose Through CLI Status

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/tests/mcp_server_commands.rs`

- [x] **Step 1: Add JSON health field**

Include the health report in `mcp_server_status` JSON output.

- [x] **Step 2: Add text health line**

Add a concise `health=<status>` line to text output.

### Task 4: Update Docs

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark M66-03 complete after checks pass**

Update milestone status and identify M66-04 as the next slice.

### Task 5: Verification

**Commands:**

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p ruse-mcp mcp_server_status_health
CARGO_TARGET_DIR=target/m66-dev cargo test -p ruse-cli --test mcp_server_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p ruse-mcp -p ruse-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 1: Run focused tests**

Run MCP status health and CLI status tests.

- [x] **Step 2: Run lint and formatting checks**

Run clippy, fmt, and whitespace checks.

### Task 6: Commit

- [x] **Step 1: Commit M66-03**

Suggested message:

```text
feat: add mcp management health report
```
