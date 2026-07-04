# M66-01 Extension Health Baseline and Smoke Plan

> **For agentic workers:** Reuse existing extension records, CLI commands, and bridge snapshots. Do not introduce a new extension subsystem.

**Goal:** Add a typed, deterministic, redacted extension health smoke report that covers plugin, skill root, and MCP server records.

**Architecture:** `agent-core` owns the health aggregation. CLI/Desktop may render typed snapshots, but no presentation layer owns extension health policy.

**Tech Stack:** Rust 2024 edition, `agent-core`, existing CLI extension health command tests, serde JSON, Cargo fmt/clippy/test.

---

### Task 1: Confirm Existing Baseline

**Files:**
- Read: `crates/agent-core/src/extension.rs`
- Read: `crates/agent-core/tests/extension_registry.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/extension_health_commands.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Verify existing extension records and snapshots**

Existing `ExtensionRegistry`, `ExtensionRecord`, health statuses, CLI `extension health`, UI extension snapshots, and redaction helpers are present.

### Task 2: Add Core Health Smoke Report

**Files:**
- Modify: `crates/agent-core/src/extension.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Modify: `crates/agent-core/tests/extension_registry.rs`

- [x] **Step 1: Define report types**

Add a serializable report with schema version, total count, counts by kind, counts by status, diagnostics, and next actions.

- [x] **Step 2: Generate reports from registry snapshots**

Add a deterministic builder over `ExtensionRegistrySnapshot` or `ExtensionRegistry` that sorts records and caps diagnostics.

- [x] **Step 3: Preserve redaction and bounds**

Reuse existing redaction helpers and cap message length and diagnostic count.

### Task 3: Prove Plugin, Skill, and MCP Baseline

**Files:**
- Modify: `crates/agent-core/tests/extension_registry.rs`

- [x] **Step 1: Add all-kind smoke fixture**

Create plugin, skill root, and MCP server records with healthy, degraded, disabled, unhealthy, unsupported, and missing-review states.

- [x] **Step 2: Assert deterministic JSON**

Verify sorted counts, stable diagnostic codes, stable next-action ids, and no secret-like text in JSON.

- [x] **Step 3: Assert bounded diagnostics**

Generate more diagnostics than the cap and prove truncation evidence is emitted without leaking raw messages.

### Task 4: Update Status Docs

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark planning complete and M66-01 in progress**

Ensure M66 appears in milestone index and active work docs.

- [x] **Step 2: Mark M66-01 complete after tests pass**

Only mark complete after focused checks pass.

### Task 5: Verification

**Commands:**

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-core extension
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-cli extension_health_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p agent-core -p agent-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 1: Run focused tests**

Run the focused Rust tests covering the new report and existing CLI health command.

- [x] **Step 2: Run formatting and lint gates**

Run fmt, clippy, and whitespace checks before committing.

### Task 6: Commit

**Files:**
- Stage only M66-01 docs and touched Rust files.
- Do not stage `tmp/`.

- [x] **Step 1: Commit M66-01**

Suggested message:

```text
feat: add extension health baseline smoke
```
