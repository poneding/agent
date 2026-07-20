# M18-04 Loop Guards and Idempotency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce comprehensive turn loop budgets and make completed tool effects replay-safe.

**Architecture:** `ComprehensiveTurnRunner` owns guard accounting and delegates replay-safe execution to the existing `IdempotentToolExecutor`. Guard failures return typed failed outcomes and emit runtime error events.

**Tech Stack:** Rust 2024 edition, Tokio tests, serde_json, existing `ruse-core` idempotency module, `ruse-storage` idempotency store, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Guard Tests

**Files:**
- Create: `crates/ruse-core/tests/comprehensive_turn_guards.rs`
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Add model-turn checkpoint test**

Add `comprehensive_turn_continues_after_model_turn_soft_limit`. The test sets `model_turn_checkpoint_interval = 1`, proves the second model call runs, and requires a `model_turn_budget_extended` warning instead of a failed outcome. This supersedes the original hard `max_model_turns` design so long tasks are not interrupted solely because they needed more model/tool rounds.

- [x] **Step 2: Add max tool calls test**

Add `comprehensive_turn_fails_before_executing_tool_over_budget`.

```rust
#[tokio::test]
async fn comprehensive_turn_fails_before_executing_tool_over_budget() {
    let (mut input, registry, resources) = fixture_two_tool_calls();
    input.loop_budget.max_tool_calls = 1;

    let outcome = ComprehensiveTurnRunner::run_with_resources(input, &registry, resources)
        .await
        .expect("guarded turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Failed);
    assert_eq!(
        outcome.failure_reason,
        Some(AgentLoopFailureReason::MaxToolCallsExceeded)
    );
    let started = outcome
        .events
        .iter()
        .filter(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
        .count();
    assert_eq!(started, 1);
}
```

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-core --test comprehensive_turn_guards
```

Expected: fails because `AgentLoopFailureReason` and guard enforcement do not exist.

### Task 2: Add Guard Contracts

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Extend `AgentLoopBudget`**

Add `max_wall_time_ms` and `max_consecutive_invalid_tool_calls` to `AgentLoopBudget`.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoopBudget {
    pub model_turn_checkpoint_interval: u32,
    pub max_tool_calls: u32,
    pub max_wall_time_ms: u64,
    pub max_observation_bytes: u64,
    pub max_consecutive_invalid_tool_calls: u32,
}
```

- [x] **Step 2: Add failure reason**

Add `AgentLoopFailureReason` and extend `AgentLoopOutcome`.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentLoopFailureReason {
    MaxToolCallsExceeded,
    MaxWallTimeExceeded,
    MaxObservationBytesExceeded,
    MaxConsecutiveInvalidToolCallsExceeded,
    InvalidBudget,
    Runtime(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentLoopOutcome {
    pub terminal_state: AgentLoopTerminalState,
    pub final_message: Option<String>,
    pub failure_reason: Option<AgentLoopFailureReason>,
    pub events: Vec<RuntimeEvent>,
}
```

- [x] **Step 3: Export failure reason**

Export `AgentLoopFailureReason` from `crates/ruse-core/src/lib.rs`.

```rust
pub use model_runtime::{
    AgentLoopBudget, AgentLoopFailureReason, AgentLoopOutcome, AgentLoopTerminalState,
    ComprehensiveTurnInput, ComprehensiveTurnRunner,
};
```

### Task 3: Enforce Guards

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Add internal guard state**

Add an internal `AgentLoopGuardState` that tracks `model_turns`, `tool_calls`, `observation_bytes`, `consecutive_invalid_tool_calls`, and `started_at`.

```rust
#[derive(Debug)]
struct AgentLoopGuardState {
    model_turns: u32,
    tool_calls: u32,
    observation_bytes: u64,
    consecutive_invalid_tool_calls: u32,
    started_at: std::time::Instant,
}
```

- [x] **Step 2: Validate budget before model invocation**

Return `AgentLoopTerminalState::Failed` with `AgentLoopFailureReason::InvalidBudget` when any budget value is zero.

- [x] **Step 3: Checkpoint long model loops**

Before each model stream invocation, count model turns and emit `model_turn_budget_extended` at each `model_turn_checkpoint_interval`. Continue while wall-time, tool-call, observation, invalid-call, and cancellation guards permit it.

- [x] **Step 4: Check tool call budget**

Before executing an accumulated ready tool call, increment the tool-call count and fail with `MaxToolCallsExceeded` when the call would exceed the budget.

- [x] **Step 5: Check observation and invalid-call budgets**

After creating each observation, add its byte length to `observation_bytes`. Increment `consecutive_invalid_tool_calls` for invalid, unknown, duplicate, or malformed calls, and reset it after a completed executable tool call.

### Task 4: Write RED Idempotency Tests

**Files:**
- Create: `crates/ruse-core/tests/comprehensive_turn_idempotency.rs`

- [x] **Step 1: Add replay test**

Add `comprehensive_turn_replays_completed_tool_effect_without_reexecution`.

```rust
#[tokio::test]
async fn comprehensive_turn_replays_completed_tool_effect_without_reexecution() {
    let (input, registry, resources, store, counter) = fixture_counting_side_effect_tool();

    let first = ComprehensiveTurnRunner::run_with_idempotency(
        input.clone(),
        &registry,
        resources.clone(),
        &store,
    )
    .await
    .expect("first turn");
    assert_eq!(first.terminal_state, AgentLoopTerminalState::Done);

    let second = ComprehensiveTurnRunner::run_with_idempotency(input, &registry, resources, &store)
        .await
        .expect("second turn");
    assert_eq!(second.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(second.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::ToolObservationRecorded(_)
    )));
}
```

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-core --test comprehensive_turn_idempotency
```

Expected: fails because comprehensive runner idempotency integration does not exist.

### Task 5: Integrate Idempotent Execution

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-core/src/idempotency.rs` if the existing executor needs a narrower public helper

- [x] **Step 1: Add idempotency-aware entrypoint**

Add `ComprehensiveTurnRunner::run_with_idempotency` that accepts an idempotency store and uses `IdempotentToolExecutor` for executable tool calls.

- [x] **Step 2: Derive stable idempotency keys**

Build idempotency keys from workspace id, conversation id, turn id, model turn index, call id, tool name, and canonical JSON input hash.

- [x] **Step 3: Continue from replayed observations**

When `IdempotentToolExecutor` returns a replayed completed effect, convert its observation event into the same continuation message shape used for freshly completed tools.

### Task 6: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-core --test comprehensive_turn_guards
cargo test -p ruse-core --test comprehensive_turn_idempotency
cargo test -p ruse-core --test comprehensive_turn_observations
```

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

Run:

```bash
git add crates/ruse-core docs
git commit -m "feat: enforce comprehensive turn guards"
```
