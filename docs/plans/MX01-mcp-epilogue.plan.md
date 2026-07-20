# MX01 MCP Epilogue Runtime Implementation Plan

**Goal:** Finish P1-P3 MCP epilogue hardening without entering the P0 OAuth/Auth product loop or disturbing the M55-M66 roadmap.

**Architecture:** `ruse-mcp` owns MCP contracts, reports, redaction, sampling review semantics, management snapshots, and export guardrails. `ruse-ui-bridge` owns bridge-consumable DTOs without taking a direct dependency on `ruse-mcp`.

**Tech Stack:** Rust 2024 edition, serde, async-trait fixture providers, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish MX01 Milestone Documents

**Files:**
- Create: `docs/specs/MX01-mcp-epilogue.spec.md`
- Create: `docs/plans/MX01-mcp-epilogue.plan.md`

- [x] **Step 1: Write milestone-level spec**
- [x] **Step 2: Write milestone-level plan**

### Task 2: Decompose MX01 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/MX01-01-reentrant-callbacks.spec.md`
- Create: `docs/plans/MX01-01-reentrant-callbacks.plan.md`
- Create: `docs/specs/MX01-02-progress-logging-task-events.spec.md`
- Create: `docs/plans/MX01-02-progress-logging-task-events.plan.md`
- Create: `docs/specs/MX01-03-sampling-productization.spec.md`
- Create: `docs/plans/MX01-03-sampling-productization.plan.md`
- Create: `docs/specs/MX01-04-mcp-management-snapshot.spec.md`
- Create: `docs/plans/MX01-04-mcp-management-snapshot.plan.md`
- Create: `docs/specs/MX01-05-server-export-guardrails.spec.md`
- Create: `docs/plans/MX01-05-server-export-guardrails.plan.md`
- Create: `docs/specs/MX01-06-hardening-docs-verification.spec.md`
- Create: `docs/plans/MX01-06-hardening-docs-verification.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement MX01 Slices in Order

**Files:**
- Execute: `docs/plans/MX01-01-reentrant-callbacks.plan.md`
- Execute: `docs/plans/MX01-02-progress-logging-task-events.plan.md`
- Execute: `docs/plans/MX01-03-sampling-productization.plan.md`
- Execute: `docs/plans/MX01-04-mcp-management-snapshot.plan.md`
- Execute: `docs/plans/MX01-05-server-export-guardrails.plan.md`
- Execute: `docs/plans/MX01-06-hardening-docs-verification.plan.md`

- [x] **Step 1: Add reentrant callback contracts and fixture tests**
- [x] **Step 2: Add MCP protocol event reports and bridge-safe summaries**
- [x] **Step 3: Add sampling approval/productization contracts and guardrail tests**
- [x] **Step 4: Add MCP management snapshots and bridge DTO tests**
- [x] **Step 5: Add server export guardrails and policy preview tests**
- [x] **Step 6: Update docs, status, and verification evidence**

### Task 4: Complete MX01

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/MX01-mcp-epilogue.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark MX01 complete**

Update MX01 status and verification evidence. Do not mark M55-M66 complete or move MX01 into the main roadmap.

Verification evidence recorded on 2026-06-30:

- Focused MX01 tests passed:
  - `cargo test -p ruse-mcp mcp_client_reentrant_callbacks`
  - `cargo test -p ruse-mcp mcp_client_protocol_events`
  - `cargo test -p ruse-mcp mcp_sampling_productization`
  - `cargo test -p ruse-mcp mcp_management_snapshot`
  - `cargo test -p ruse-mcp mcp_server_export_guardrails`
  - `cargo test -p ruse-ui-bridge mcp_management_snapshot`
- Final workspace gates passed:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo xtask ci`
- `target/ruse-ci/quality-gate-summary.txt` reports `total=4 passed=4 failed=0 skipped=0`.

MX01 is complete as an MCP epilogue. It remains separate from M55-M66 and does not implement P0 OAuth/Auth product closure.
