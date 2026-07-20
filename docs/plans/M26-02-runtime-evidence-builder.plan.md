# M26-02 Runtime Evidence Builder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a core observability evidence bundle from runtime events and telemetry records.

**Architecture:** `ruse-core` composes evidence using existing trace and audit exporters plus `ruse-telemetry` export bundles.

**Tech Stack:** Rust 2024 edition, serde, ruse-types runtime events, ruse-core trace/audit modules, ruse-telemetry export, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-02-runtime-evidence-builder.spec.md`
- Read: `crates/ruse-core/src/trace.rs`
- Read: `crates/ruse-core/src/audit_export.rs`
- Read: `crates/ruse-core/tests/local_coding_e2e.rs`

- [x] **Step 1: Review evidence composition boundaries**

Expected: core composes but does not render UI.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-core/tests/observability_evidence.rs`

- [x] **Step 1: Add evidence builder tests**

Cover serde shape, local coding summary, invalid options, and secret leakage prevention.

### Task 3: Implement Evidence Builder

**Files:**
- Create: `crates/ruse-core/src/observability.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add bundle contracts**

Implement options, summaries, replay evidence metadata, errors, and export function.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

```bash
cargo test -p ruse-core observability_evidence
cargo clippy -p ruse-core --all-targets -- -D warnings
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M26-02 complete and commit**

```bash
git add crates/ruse-core docs
git commit -m "feat: add runtime observability evidence"
```
