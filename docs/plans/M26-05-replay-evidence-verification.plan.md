# M26-05 Replay Evidence Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify that observability evidence can be regenerated from replay fixtures and stored event logs.

**Architecture:** `ruse-core` verifies evidence from `ruse-storage` replay fixtures; CLI/UI remain consumers of verified contracts.

**Tech Stack:** Rust 2024 edition, ruse-storage replay fixtures, ruse-core evidence builder, Cargo xtask check.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-05-replay-evidence-verification.spec.md`
- Read: `crates/ruse-storage/tests/fixtures/m04_trace_v1.json`
- Read: `crates/ruse-storage/src/compatibility.rs`

- [x] **Step 1: Review replay fixture contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-core/tests/evidence_replay_verification.rs`

- [x] **Step 1: Add replay evidence verification tests**

### Task 3: Implement Verification

**Files:**
- Modify: `crates/ruse-core/src/observability.rs`

- [x] **Step 1: Add verification report and checker**

### Task 4: Verify

- [x] **Step 1: Run final M26 checks**

```bash
cargo xtask check
```

### Task 5: Update Status and Commit

- [x] **Step 1: Mark M26 complete and commit**

```bash
git add crates docs README.md
git commit -m "feat: complete m26 observability evidence"
```
