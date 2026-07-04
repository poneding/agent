# M18-03 Tool Observation Context Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute ready tool calls and pass bounded observations into continuation model turns.

**Architecture:** `ComprehensiveTurnRunner` uses `ToolCallAccumulator`, `ToolLoop`, `ToolRegistry`, and `PolicyEngine`. Observation shaping stays in `agent-core`; tool implementations stay in `agent-tool`.

**Tech Stack:** Rust 2024 edition, Tokio tests, serde_json, existing tool and policy crates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Observation Tests

**Files:**
- Create: `crates/agent-core/tests/comprehensive_turn_observations.rs`

- [x] **Step 1: Add tool observation continuation test**

Add `comprehensive_turn_feeds_tool_observation_to_next_model_turn`.

```rust
let outcome = ComprehensiveTurnRunner::run_with_resources(input, &models, resources)
    .await
    .expect("turn");
assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
assert!(recorded_model_requests()[1].messages.iter().any(|message| message.contains("tool observation")));
```

- [x] **Step 2: Add ask suspension test**

Add `comprehensive_turn_suspends_when_policy_asks`.

```rust
assert_eq!(outcome.terminal_state, AgentLoopTerminalState::WaitingApproval);
assert!(outcome.events.iter().any(|event| matches!(
    event.payload,
    RuntimeEventPayload::PermissionDecisionRecorded(_)
)));
```

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p agent-core --test comprehensive_turn_observations
```

Expected: fails because resource-aware comprehensive runner does not exist.

### Task 2: Add Resource-Aware Runner API

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add resource contract**

Add `ComprehensiveTurnResources<'a, P>` with `tools: &'a ToolRegistry` and `policy: &'a P`.

- [x] **Step 2: Add `run_with_resources`**

Implement a runner entrypoint that executes tool calls and can continue model streaming.

### Task 3: Convert Tool Outcomes To Observations

**Files:**
- Modify: `crates/agent-core/src/tool_loop.rs`
- Modify: `crates/agent-core/src/model_runtime.rs`

- [x] **Step 1: Build observation messages**

Convert completed, denied, failed, and invalid tool results into `ToolObservationMessage`.

- [x] **Step 2: Enforce observation byte limit**

Use `AgentLoopBudget.max_observation_bytes`; truncate on UTF-8 char boundaries.

- [x] **Step 3: Redact secret-like text**

Apply a local redaction helper for `sk-`, `api_key`, `token`, and `password`.

### Task 4: Continue Model Loop

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`

- [x] **Step 1: Append observation messages**

Add observation text to continuation messages for the next model request.

- [x] **Step 2: Continue until final response**

Loop model turns until final response, waiting approval, failure, or budget exhaustion.

### Task 5: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core --test comprehensive_turn_observations
cargo test -p agent-core --test tool_loop
```

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

Run:

```bash
git add crates/agent-core docs
git commit -m "feat: feed tool observations into model loop"
```
