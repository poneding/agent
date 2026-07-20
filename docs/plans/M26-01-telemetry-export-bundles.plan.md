# M26-01 Telemetry Export Bundles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted, bounded, filterable telemetry export bundles in `ruse-telemetry`.

**Architecture:** `ruse-telemetry` owns telemetry export contracts and depends only on `ruse-types`, serde, and local telemetry types.

**Tech Stack:** Rust 2024 edition, serde, ruse-types `Timestamp`, existing telemetry span/metric records, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-01-telemetry-export-bundles.spec.md`
- Read: `crates/ruse-telemetry/src/lib.rs`
- Read: `crates/ruse-telemetry/src/sink.rs`
- Read: `crates/ruse-telemetry/tests/telemetry.rs`

- [x] **Step 1: Review the spec and current telemetry contracts**

Expected: export belongs in `ruse-telemetry` and must not depend on core or UI crates.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-telemetry/tests/telemetry_export.rs`

- [x] **Step 1: Add telemetry export tests**

Cover stable serde shape, filtering, redaction, truncation diagnostics, invalid options, and empty exports.

- [x] **Step 2: Run focused tests and verify RED**

Run:

```bash
cargo test -p ruse-telemetry telemetry_export
```

Expected: tests fail before export contracts exist.

### Task 3: Implement Telemetry Export

**Files:**
- Create: `crates/ruse-telemetry/src/export.rs`
- Modify: `crates/ruse-telemetry/src/lib.rs`

- [x] **Step 1: Add export contracts and validation**

Implement bundle structs, options, errors, filtering, generated timestamp injection, and summaries.

- [x] **Step 2: Add redaction and truncation**

Reuse existing telemetry redaction before applying bounded attribute values and diagnostics.

- [x] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test -p ruse-telemetry telemetry_export
```

Expected: focused tests pass.

### Task 4: Verify

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-telemetry --all-targets -- -D warnings
cargo test -p ruse-telemetry
```

Expected: all focused checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M26-01-telemetry-export-bundles.plan.md`
- Modify: `docs/plans/M26-runtime-observability-evidence.plan.md`

- [x] **Step 1: Mark M26-01 complete**

Record M26-02 as the next implementation slice.

- [x] **Step 2: Commit M26-01**

Run:

```bash
git add crates/ruse-telemetry docs
git commit -m "feat: add telemetry export bundles"
```

Expected: commit succeeds.
