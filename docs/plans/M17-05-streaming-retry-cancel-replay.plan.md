# M17-05 Streaming Retry, Cancellation, and Replay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make streaming model execution retryable, cancellable, and replayable.

**Architecture:** `ruse-core` owns policy and event sourcing. Providers classify errors; core decides retry, fallback, cancellation, and replay behavior.

**Tech Stack:** Rust 2024 edition, Tokio cancellation primitives, futures streams, existing event store/replay contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Policy Tests

**Files:**
- Create: `crates/ruse-core/tests/streaming_retry_cancel_replay.rs`
- Modify: `crates/ruse-core/tests/model_invocation_policy_runner.rs`
- Modify: `crates/ruse-storage/tests/replay_integration.rs`

- [x] **Step 1: Add retryable stream failure test**

Add `streaming_retry_retries_retryable_error_before_delta`.

- [x] **Step 2: Add non-retryable stream failure test**

Add `streaming_retry_does_not_retry_auth_error`.

- [x] **Step 3: Add fallback test**

Add `streaming_retry_uses_fallback_provider_after_stream_failure`.

- [x] **Step 4: Add cancellation test**

Add `streaming_cancellation_emits_cancelled_state`.

- [x] **Step 5: Add replay test**

Add `streaming_replay_reconstructs_final_response_from_events`.

- [x] **Step 6: Verify RED**

Run:

```bash
cargo test -p ruse-core --test streaming_retry_cancel_replay
cargo test -p ruse-core --test model_invocation_policy_runner
cargo test -p ruse-storage --test replay_integration
```

Expected: failures because stream item retry/cancel/replay behavior is incomplete.

### Task 2: Extend Invocation Policy Over Streams

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-model/src/provider.rs`
- Modify: `crates/ruse-model/src/stream.rs`

- [x] **Step 1: Track attempt state**

Record provider id, model, attempt index, and whether any accepted side-effect boundary was crossed.

- [x] **Step 2: Retry setup and early item failures**

Retry only retryable failures before side-effect boundaries.

- [x] **Step 3: Apply fallback providers**

Fallback attempts preserve attempt indices and model override behavior.

### Task 3: Add Cancellation

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-types/src/event.rs`
- Modify: `crates/ruse-ui-bridge/src/event.rs`

- [x] **Step 1: Add cancellation input**

Accept a cancellation token or equivalent cancellation handle in streaming runner options.

- [x] **Step 2: Emit cancelled runtime event**

Use existing turn state events where possible; add a typed payload only if current events cannot represent cancellation.

- [x] **Step 3: Stop consuming stream**

Dropping the stream is sufficient for M17 model cancellation.

### Task 4: Add Replay Coverage

**Files:**
- Modify: `crates/ruse-core/src/trace.rs`
- Modify: `crates/ruse-storage/tests/replay_integration.rs`

- [x] **Step 1: Reconstruct final text from persisted deltas**

Replay uses events only, not providers.

- [x] **Step 2: Preserve redaction in replay diagnostics**

Replay output must not include raw provider secrets or request bodies.

### Task 5: Verify And Complete M17

**Files:**
- Modify: `docs/plans/M17-05-streaming-retry-cancel-replay.plan.md`
- Modify: `docs/plans/M17-live-model-runtime-streaming.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-core --test streaming_retry_cancel_replay
cargo test -p ruse-core --test model_invocation_policy_runner
cargo test -p ruse-storage --test replay_integration
```

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [x] **Step 3: Mark M17 complete**

Update README and milestone docs so active focus advances to M18.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m17 live model streaming"
```
