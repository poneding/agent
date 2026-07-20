# M19-02 UI Approval Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose pending approvals and approval responses through typed `ruse-ui-bridge` frames and commands.

**Architecture:** `ruse-ui-bridge` consumes `ruse-policy` approval contracts and emits UI-safe event/command payloads. Runtime approval execution remains in `ruse-core` for M19-05.

**Tech Stack:** Rust 2024 edition, serde UI schemas, existing `UiJsonFrame`, snapshot tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-02-ui-approval-bridge.spec.md`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-policy/src/approval.rs`

- [x] **Step 1: Review bridge scope**

Run:

```bash
sed -n '1,220p' docs/specs/M19-02-ui-approval-bridge.spec.md
sed -n '1,220p' crates/ruse-ui-bridge/src/event.rs
sed -n '1,220p' crates/ruse-ui-bridge/src/command.rs
```

Expected: this slice adds UI-safe schemas only.

### Task 2: Write RED Bridge Tests

**Files:**
- Create: `crates/ruse-ui-bridge/tests/approval_bridge.rs`

- [x] **Step 1: Add approval frame and command tests**

Add tests for serializing `ApprovalRequested`, serializing `ResolveApproval`, validating empty ids/reasons, and redacting preview summaries.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-ui-bridge --test approval_bridge
```

Expected: compile fails because approval bridge payloads do not exist.

### Task 3: Implement Bridge Payloads

**Files:**
- Modify: `crates/ruse-ui-bridge/src/event.rs`
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add event payloads**

Add `UiApprovalRequest`, `UiApprovalResolved`, and `UiEventPayload` variants for approval requested/resolved.

- [x] **Step 2: Add response command**

Add `UiApprovalResponseCommand` and `UiCommandPayload::ResolveApproval`.

- [x] **Step 3: Add normalization helpers**

Convert `ruse_policy::PendingApproval` into a UI-safe request payload without raw tool input.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-ui-bridge/tests/approval_bridge.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test approval_bridge
```

Expected: focused tests pass.

- [x] **Step 2: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge
```

Expected: all bridge tests pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M19-02-ui-approval-bridge.plan.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M19-02 complete**

Update plan and milestone checkboxes after verification.
