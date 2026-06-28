# M26-02 Runtime Evidence Builder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a core observability evidence bundle from runtime events and telemetry records.

**Architecture:** `agent-core` composes evidence using existing trace and audit exporters plus `agent-telemetry` export bundles.

**Tech Stack:** Rust 2024 edition, serde, agent-types runtime events, agent-core trace/audit modules, agent-telemetry export, Cargo checks.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-02-runtime-evidence-builder.spec.md`
- Read: `crates/agent-core/src/trace.rs`
- Read: `crates/agent-core/src/audit_export.rs`
- Read: `crates/agent-core/tests/local_coding_e2e.rs`

- [ ] **Step 1: Review evidence composition boundaries**

Expected: core composes but does not render UI.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-core/tests/observability_evidence.rs`

- [ ] **Step 1: Add evidence builder tests**

Cover serde shape, local coding summary, invalid options, and secret leakage prevention.

### Task 3: Implement Evidence Builder

**Files:**
- Create: `crates/agent-core/src/observability.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Add bundle contracts**

Implement options, summaries, replay evidence metadata, errors, and export function.

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

```bash
cargo test -p agent-core observability_evidence
cargo clippy -p agent-core --all-targets -- -D warnings
```

### Task 5: Update Status and Commit

- [ ] **Step 1: Mark M26-02 complete and commit**

```bash
git add crates/agent-core docs
git commit -m "feat: add runtime observability evidence"
```
