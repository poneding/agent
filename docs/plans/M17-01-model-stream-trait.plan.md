# M17-01 Model Stream Trait Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate model providers from collected vectors to true async event streams.

**Architecture:** `ruse-model` defines `ModelEventStream` and collection helpers. Providers return streams; `ruse-core` collects only where compatibility outcomes still require a final vector.

**Tech Stack:** Rust 2024 edition, `futures-core`, `futures-util`, `async-stream` or `futures_util::stream`, Tokio tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Stream Contract Tests

**Files:**
- Create: `crates/ruse-model/tests/model_stream_trait.rs`
- Modify: `crates/ruse-core/tests/configured_turn_runner.rs`

- [x] **Step 1: Add fake stream tests**

Add tests:

- `model_event_stream_collects_fake_events_in_order`
- `model_event_stream_propagates_item_errors`

Expected contract:

```rust
let stream = provider.stream(model_request()).await.expect("stream");
let events = collect_model_event_stream(stream).await.expect("collect");
assert_eq!(events, expected_events);
```

- [x] **Step 2: Add configured turn compatibility test**

Add test:

- `configured_turn_runner_collects_streamed_model_events`

Assert existing final response behavior remains unchanged after the trait migration.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-model --test model_stream_trait
cargo test -p ruse-core --test configured_turn_runner configured_turn_runner_collects_streamed_model_events
```

Expected: compile or test failures because `ModelEventStream` and collection helper do not exist.

### Task 2: Add Stream Types And Helpers

**Files:**
- Modify: `crates/ruse-model/Cargo.toml`
- Modify: `crates/ruse-model/src/stream.rs`
- Modify: `crates/ruse-model/src/lib.rs`

- [x] **Step 1: Add stream dependencies**

Add `futures-core` and `futures-util` to `ruse-model`.

- [x] **Step 2: Define `ModelEventStream`**

Add:

```rust
pub type ModelEventStream =
    Pin<Box<dyn Stream<Item = Result<ModelStreamEvent, ModelError>> + Send>>;
```

- [x] **Step 3: Add collection helper**

Add:

```rust
pub async fn collect_model_event_stream(
    stream: ModelEventStream,
) -> Result<Vec<ModelStreamEvent>, ModelError>
```

The helper preserves order and returns the first item error.

### Task 3: Migrate Providers

**Files:**
- Modify: `crates/ruse-model/src/provider.rs`
- Modify: `crates/ruse-model/src/fake.rs`
- Modify: `crates/ruse-model/src/openai_compatible.rs`
- Modify: `crates/ruse-model/tests/fake_model.rs`
- Modify: `crates/ruse-model/tests/fake_tool_call.rs`
- Modify: `crates/ruse-model/tests/openai_compatible.rs`

- [x] **Step 1: Change `ModelProvider::stream` return type**

Return `Result<ModelEventStream, ModelError>`.

- [x] **Step 2: Wrap fake scripts in streams**

Fake provider returns stream items in scripted order.

- [x] **Step 3: Wrap OpenAI-compatible fixture events in streams**

Keep fixture behavior deterministic while returning stream items.

- [x] **Step 4: Update tests to collect explicitly**

Replace direct vector assertions with `collect_model_event_stream`.

### Task 4: Migrate Core Collection Boundary

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify: `crates/ruse-core/src/turn.rs`
- Modify: `crates/ruse-core/tests/configured_turn_runner.rs`
- Modify: `crates/ruse-core/tests/model_invocation_policy_runner.rs`

- [x] **Step 1: Collect streams only in core compatibility wrappers**

Use `collect_model_event_stream` where existing return types still require `Vec<ModelStreamEvent>`.

- [x] **Step 2: Preserve model error mapping**

Stream item errors must reach existing `TurnError::Model` and configured turn errors.

### Task 5: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-model --test model_stream_trait
cargo test -p ruse-model
cargo test -p ruse-core --test configured_turn_runner
cargo test -p ruse-core --test model_invocation_policy_runner
```

Expected: all focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: workspace passes.

- [ ] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates/ruse-model crates/ruse-core docs
git commit -m "feat: add m17 model event stream trait"
```
