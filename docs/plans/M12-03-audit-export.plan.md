# M12-03 Audit Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted audit export bundles derived from `ruse-types` runtime events.

**Architecture:** Implement export composition in `ruse-core`; use typed `ruse-types` events as input and produce serializable export bundles for future CLI/Desktop surfaces.

**Tech Stack:** Rust 2024 edition, serde, serde_json, existing `ruse-core` and `ruse-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-03-audit-export.spec.md`
- Read: `crates/ruse-core/src/lib.rs`
- Read: `crates/ruse-types/src/event.rs`

- [x] **Step 1: Review event payloads**

Run:

```bash
sed -n '1,220p' crates/ruse-types/src/event.rs
sed -n '1,240p' docs/specs/M12-03-audit-export.spec.md
```

Expected: audit export derives from existing runtime events.

### Task 2: Write RED Audit Export Tests

**Files:**
- Create: `crates/ruse-core/tests/audit_export.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `audit_export_serializes_stable_bundle_shape`
- `audit_export_maps_permission_and_tool_events`
- `audit_export_redacts_secret_like_payload_fields`
- `audit_export_allows_empty_event_list`
- `audit_export_rejects_invalid_options`

The tests define the public API for `AuditExportOptions`, `AuditExportBundle`, `AuditExportRecord`, `TraceExportSummary`, `AuditEventCategory`, `AuditExportError`, and `export_audit_bundle_at`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-core audit_export
```

Expected: tests fail because audit export contracts do not exist.

### Task 3: Implement Audit Export

**Files:**
- Modify: `crates/ruse-core/src/lib.rs`
- Create: `crates/ruse-core/src/audit_export.rs`

- [x] **Step 1: Add export contracts**

Implement `AuditExportOptions`, `AuditExportBundle`, `AuditExportRecord`, `TraceExportSummary`, and `AuditExportError`.

- [x] **Step 2: Add event conversion**

Convert permission, tool, model, lifecycle, external trace, warning, and error payloads into bounded audit summaries.

- [x] **Step 3: Add redaction and truncation**

Redact secret-like keys and values and truncate payload summaries above `max_payload_bytes`.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/ruse-core/tests/audit_export.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-core audit_export
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-03-audit-export.plan.md`

- [x] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m12 audit export"
```

Expected: commit succeeds.
