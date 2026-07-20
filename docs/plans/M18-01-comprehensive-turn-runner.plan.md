# M18-01 Comprehensive Turn Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first comprehensive turn runner contracts and model-stream orchestration.

**Architecture:** `ruse-core` owns `ComprehensiveTurnRunner` and reuses M17 model streaming. This slice records final responses and tool proposals but does not execute tools.

**Tech Stack:** Rust 2024 edition, Tokio tests, `ruse-core`, `ruse-model`, `ruse-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Runner Tests

**Files:**
- Create: `crates/ruse-core/tests/comprehensive_turn_runner.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add final response test**

Add `comprehensive_turn_runner_finishes_on_final_response`.

```rust
#[tokio::test]
async fn comprehensive_turn_runner_finishes_on_final_response() {
    let (input, registry) = fixture_with_fake_text("done");
    let outcome = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect("comprehensive turn");
    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(outcome.final_message.as_deref(), Some("done"));
}
```

- [x] **Step 2: Add tool proposal suspension test**

Add `comprehensive_turn_runner_waits_on_tool_proposal`.

```rust
#[tokio::test]
async fn comprehensive_turn_runner_waits_on_tool_proposal() {
    let (input, registry) = fixture_with_fake_tool_call("call-1", "workspace.read_file");
    let outcome = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect("comprehensive turn");
    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::WaitingApproval);
    assert!(outcome.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::ToolCallProposed(_)
    )));
}
```

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-core --test comprehensive_turn_runner
```

Expected: fails because `ComprehensiveTurnRunner` contracts do not exist.

### Task 2: Add Core Contracts

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add public structs and enums**

Add `ComprehensiveTurnInput`, `AgentLoopBudget`, `AgentLoopTerminalState`, and `AgentLoopOutcome` near configured turn contracts.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoopBudget {
    pub model_turn_checkpoint_interval: u32,
    pub max_tool_calls: u32,
    pub max_observation_bytes: u64,
}
```

- [x] **Step 2: Export contracts**

Export the new types from `crates/ruse-core/src/lib.rs`.

```rust
pub use model_runtime::{
    AgentLoopBudget, AgentLoopOutcome, AgentLoopTerminalState, ComprehensiveTurnInput,
    ComprehensiveTurnRunner,
};
```

### Task 3: Implement Minimal Runner

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Stream one model turn**

Implement `ComprehensiveTurnRunner::run` by building `ConfiguredTurnInput`, consuming `ConfiguredTurnRunner::run_stream`, and collecting events.

- [x] **Step 2: Stop on tool proposal**

When `RuntimeEventPayload::ToolCallProposed(_)` appears, return `AgentLoopTerminalState::WaitingApproval` with collected events.

- [x] **Step 3: Stop on final response**

When the stream completes with `FinalResponseProduced`, return `AgentLoopTerminalState::Done`.

### Task 4: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-core --test comprehensive_turn_runner
cargo test -p ruse-core --test configured_turn_runner
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
git commit -m "feat: add comprehensive turn runner"
```
