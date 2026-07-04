# M02-05 Tool Loop Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove a fake model tool call can be validated, authorized, executed, audited, observed, denied, or held for approval.

**Architecture:** `agent-core` orchestrates the tool loop using `agent-model`, `agent-tool`, `agent-policy`, and `agent-types`. Tool and policy implementations remain trait-backed and replaceable.

**Tech Stack:** Rust 2024 edition, M02 crates, `serde_json`, `tokio` async tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Extend Fake Model Stream

**Files:**
- Modify: `crates/agent-model/src/stream.rs`
- Modify: `crates/agent-model/src/fake.rs`
- Create: `crates/agent-model/tests/fake_tool_call.rs`

- [x] **Step 1: Write failing fake tool-call test**

Test that a fake model script emits `ModelStreamEvent::ToolCall` with call ID, tool name, and JSON input.

- [x] **Step 2: Implement model stream event**

Add `ToolCall { call_id, tool_name, input_json }` and script support.

- [x] **Step 3: Verify focused test**

```bash
cargo test -p agent-model fake_tool_call
```

Expected: fake tool-call tests pass.

### Task 2: Add Tool Loop Tests

**Files:**
- Create: `crates/agent-core/tests/tool_loop.rs`

- [x] **Step 1: Write failing integration tests**

Tests must prove:

- allow decision executes read-only tool
- deny decision emits no execution event
- ask decision reaches `TurnState::WaitingApproval`
- audit event order is proposal, decision, execution, observation

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-core tool_loop
```

Expected: tests fail before `ToolLoop` exists.

### Task 3: Implement Tool Loop

**Files:**
- Modify: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/src/tool_loop.rs`

- [x] **Step 1: Define outcome types**

Implement `ToolLoopOutcome`, `ToolLoopStatus`, and `ToolLoopError`.

- [x] **Step 2: Implement allow path**

Validate tool lookup, request policy decision, execute tool on allow, emit audit events, and return observation.

- [x] **Step 3: Implement deny and ask paths**

Deny emits decision and terminal denied outcome. Ask emits decision and waiting-approval outcome without execution.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p agent-core tool_loop
```

Expected: tool loop tests pass.

### Task 4: Run M02 Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M02-05-tool-loop-integration.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Mark M02 complete**

Update milestone status so M02 is complete and active focus advances to `M03-local-coding-loop`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m02 tool policy audit"
```
