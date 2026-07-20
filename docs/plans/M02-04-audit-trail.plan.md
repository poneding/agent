# M02-04 Audit Trail Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed runtime event payloads for tool proposals, permission decisions, executions, failures, and observations.

**Architecture:** Durable audit contracts live in `ruse-types`. `ruse-storage` replay tests prove the events are append-only observations rather than side-effect re-execution.

**Tech Stack:** Rust 2024 edition, `ruse-types`, `ruse-storage`, `serde_json`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Failing Audit Event Tests

**Files:**
- Create: `crates/ruse-types/tests/tool_audit_json.rs`
- Create: `crates/ruse-storage/tests/tool_audit_replay.rs`

- [x] **Step 1: Add JSON shape tests**

Tests must prove new audit payloads serialize with stable `snake_case` variant names and preserve input/output JSON values.

- [x] **Step 2: Add replay ordering test**

Persist proposed, decision, started, completed, and observed events, then replay and assert ordering.

- [x] **Step 3: Verify RED**

```bash
cargo test -p ruse-types tool_audit_json
cargo test -p ruse-storage tool_audit_replay
```

Expected: tests fail before payload variants exist.

### Task 2: Implement Audit Payloads

**Files:**
- Modify: `crates/ruse-types/Cargo.toml`
- Modify: `crates/ruse-types/src/lib.rs`
- Modify: `crates/ruse-types/src/event.rs`
- Create: `crates/ruse-types/src/tool.rs`

- [x] **Step 1: Promote `serde_json` to normal dependency**

Audit payloads carry JSON tool input/output values.

- [x] **Step 2: Add payload structs**

Implement `ToolCallProposed`, `PermissionDecisionRecorded`, `ToolExecutionStarted`, `ToolExecutionCompleted`, `ToolExecutionFailed`, and `ToolObservationRecorded`.

- [x] **Step 3: Extend `RuntimeEventPayload`**

Add variants for every audit payload.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p ruse-types tool_audit_json
cargo test -p ruse-storage tool_audit_replay
```

Expected: focused audit tests pass.

### Task 3: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M02-04-audit-trail.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M02-04-audit-trail` to `M02-05-tool-loop-integration`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m02-04 audit trail"
```
