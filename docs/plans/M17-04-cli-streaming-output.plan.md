# M17-04 CLI Streaming Output Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render model deltas incrementally in CLI text and JSON modes.

**Architecture:** `agent-core` exposes a streaming configured turn API. `agent-cli` renders events as they arrive while compatibility helpers can still collect for old tests.

**Tech Stack:** Rust 2024 edition, Tokio current-thread runtime, futures streams, `agent-ui-bridge` JSON frames, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Streaming Output Tests

**Files:**
- Create: `crates/agent-core/tests/configured_turn_streaming.rs`
- Modify: `crates/agent-cli/tests/json_mode.rs`
- Modify: `crates/agent-cli/tests/prompt_runtime.rs`

- [x] **Step 1: Add core event stream test**

Add `configured_turn_streaming_yields_deltas_before_final_response`.

Use a delayed fake provider stream and assert delta events are observed before completion.

- [x] **Step 2: Add CLI JSON order test**

Add `json_mode_prompt_runtime_emits_delta_frames_before_done`.

Assert NDJSON frames are parseable and delta frames precede done.

- [x] **Step 3: Add text output test**

Add `cli_prompt_runtime_text_streams_deltas_without_json_frames`.

Assert text output contains model text and not raw JSON frame tags.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-core --test configured_turn_streaming
cargo test -p agent-cli --test json_mode json_mode_prompt_runtime_emits_delta_frames_before_done
cargo test -p agent-cli --test prompt_runtime cli_prompt_runtime_text_streams_deltas_without_json_frames
```

Expected: failures because configured turns are still collected internally.

### Task 2: Add Core Streaming API

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`
- Modify: `crates/agent-core/src/turn.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add streaming outcome type**

Expose a stream of `RuntimeEvent` plus selected provider/model metadata.

- [x] **Step 2: Refactor collected runner**

Implement existing `ConfiguredTurnRunner::run` by collecting the new stream.

- [x] **Step 3: Preserve prompt snapshot first**

The first emitted event remains `PromptSnapshotRecorded`.

### Task 3: Render CLI Streaming

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/tests/json_mode.rs`
- Modify: `crates/agent-cli/tests/prompt_runtime.rs`

- [x] **Step 1: Render JSON frames while consuming events**

Emit `UiJsonFrame::Event` immediately per runtime event, then `Done`.

- [x] **Step 2: Render text deltas**

Text mode prints model delta text and final newline. It does not emit NDJSON.

- [x] **Step 3: Render stream errors**

JSON mode emits `Error` then `Done`; text mode returns non-zero with redacted message.

### Task 4: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core --test configured_turn_streaming
cargo test -p agent-cli --test json_mode
cargo test -p agent-cli --test prompt_runtime
```

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

Run:

```bash
git add crates/agent-core crates/agent-cli docs
git commit -m "feat: stream cli model output"
```
