# M19-04 CLI Approval Prompt Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add plain terminal approval prompts for pending approvals while preserving batch and JSON safety.

**Architecture:** `ruse-cli` owns terminal rendering and input parsing. It consumes `ruse-ui-bridge` approval frames and produces typed approval response commands; it does not create leases or execute tools.

**Tech Stack:** Rust 2024 edition, existing CLI parser/rendering tests, stdin fixture tests, serde JSON frames, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-04-cli-approval-prompt.spec.md`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/src/lib.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`

- [x] **Step 1: Review CLI boundaries**

Run:

```bash
sed -n '1,220p' docs/specs/M19-04-cli-approval-prompt.spec.md
sed -n '1,260p' crates/ruse-cli/src/command.rs
```

Expected: CLI prompt owns interaction only and delegates approval execution to core.

### Task 2: Write RED CLI Tests

**Files:**
- Create: `crates/ruse-cli/tests/approval_prompt.rs`

- [x] **Step 1: Add prompt parser and rendering tests**

Add tests for allow, deny, abort, invalid input, batch approval-required failure, and JSON frame behavior.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-cli --test approval_prompt
```

Expected: compile fails because approval prompt contracts do not exist in CLI.

### Task 3: Implement CLI Prompt

**Files:**
- Create: `crates/ruse-cli/src/approval.rs`
- Modify: `crates/ruse-cli/src/lib.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add approval input parser**

Implement explicit allow, deny, abort, and invalid input handling.

- [x] **Step 2: Add text renderer**

Render action summary, preview summary, scope, duration, policy reason, and choices.

- [x] **Step 3: Add mode behavior**

Ensure interactive prompts block only in interactive mode; batch and JSON modes remain script-safe.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-cli/tests/approval_prompt.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-cli --test approval_prompt
```

Expected: focused tests pass.

- [x] **Step 2: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli
```

Expected: all CLI tests pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M19-04-cli-approval-prompt.plan.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M19-04 complete**

Update plan and milestone checkboxes after verification.
