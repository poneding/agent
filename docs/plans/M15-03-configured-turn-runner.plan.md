# M15-03 Configured Turn Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run `TurnEngine` from effective config and a provider registry.

**Architecture:** Add configured turn orchestration to `agent-core::model_runtime`. The runner resolves the provider, applies invocation policy through M15-02 contracts, and returns normal runtime events.

**Tech Stack:** Rust 2024 edition, async turn tests, fake providers, existing runtime events, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-03-configured-turn-runner.spec.md`
- Read: `crates/agent-core/src/model_runtime.rs`
- Read: `crates/agent-core/src/turn.rs`
- Read: `crates/agent-core/tests/turn_engine.rs`

- [ ] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M15-03-configured-turn-runner.spec.md
```

Expected: the spec defines configured turn input, outcome, errors, and tests.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/configured_turn_runner.rs`

- [ ] **Step 1: Add focused tests**

Add tests named:

- `configured_turn_runner_uses_effective_provider_and_model`
- `configured_turn_runner_rejects_empty_user_message`
- `configured_turn_runner_resolution_failure_prevents_model_call`
- `configured_turn_runner_success_emits_final_response`

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core --test configured_turn_runner
```

Expected: tests fail because configured turn runner contracts do not exist.

### Task 3: Implement Configured Runner

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`

- [ ] **Step 1: Add configured turn contracts**

Implement:

- `ConfiguredTurnInput`
- `ConfiguredTurnOutcome`
- `ConfiguredTurnRunner`

- [ ] **Step 2: Wire resolver and turn engine**

Rules:

- reject empty prompt before provider resolution
- use selected provider/model from resolved runtime
- return final message, events, and attempt history

- [ ] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-core --test configured_turn_runner
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-03-configured-turn-runner.plan.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

- [ ] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m15 configured turn runner"
```

Expected: commit succeeds.
