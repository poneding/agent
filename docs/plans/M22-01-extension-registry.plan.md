# M22-01 Extension Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed extension registry contracts in `agent-core` for plugin, skill root, and MCP server extension state.

**Architecture:** `agent-core` owns cross-kind registry state. Domain crates keep native validation. Bridge/UI crates consume snapshots later and do not own registry mutation.

**Tech Stack:** Rust 2024 edition, serde, agent-policy capability leases, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-01-extension-registry.spec.md`
- Read: `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md`
- Read: `docs/architecture/crate-boundaries.md`

- [ ] **Step 1: Review registry scope**

Run:

```bash
sed -n '1,260p' docs/specs/M22-01-extension-registry.spec.md
```

Expected: registry state, failure behavior, privacy, and test expectations are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/extension_registry.rs`

- [ ] **Step 1: Add registry construction tests**

Cover valid plugin/skill/MCP records, duplicate id rejection, deterministic snapshot ordering, and stable serialization.

- [ ] **Step 2: Add state transition tests**

Cover upsert, disable, remove, unsupported compatibility, and lease revocation.

- [ ] **Step 3: Add redaction tests**

Cover health/source messages containing `token`, `password`, `api_key`, `secret`, and `sk-`.

- [ ] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-core extension_registry
```

Expected: tests fail because registry contracts are missing.

### Task 3: Implement Registry Contracts

**Files:**
- Create: `crates/agent-core/src/extension.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Add typed records and validation**

Implement `ExtensionId`, `ExtensionKind`, `ExtensionSource`, `ExtensionCompatibility`, `ExtensionHealth`, `ExtensionCapabilitySummary`, `ExtensionCapabilityLease`, and `ExtensionRecord`.

- [ ] **Step 2: Add registry operations**

Implement `ExtensionRegistry::new`, `upsert`, `disable`, `remove`, `get`, `list`, and `snapshot`.

- [ ] **Step 3: Add safe summaries**

Redact health and source display fields before serialization or snapshot construction.

- [ ] **Step 4: Export contracts**

Re-export public types from `agent-core`.

### Task 4: Verify

**Files:**
- Read: `crates/agent-core/tests/extension_registry.rs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core extension_registry
```

Expected: focused registry tests pass.

- [ ] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M22-01-extension-registry.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Mark M22-01 complete**

Update M22 progress so the active implementation focus advances to M22-02.

- [ ] **Step 2: Commit M22-01**

Run:

```bash
git add crates/agent-core docs
git commit -m "feat: add extension registry"
```

Expected: commit succeeds.
