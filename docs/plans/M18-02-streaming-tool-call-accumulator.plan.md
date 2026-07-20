# M18-02 Streaming Tool Call Accumulator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed accumulator that converts model tool-call stream events into executable requests or recoverable observations.

**Architecture:** `ruse-core` owns accumulation and validation. It creates `ToolCallRequest` values for valid calls and `ToolObservationMessage` values for invalid or duplicate calls.

**Tech Stack:** Rust 2024 edition, serde_json, `ruse-model`, `ruse-tool`, `ruse-core`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Accumulator Tests

**Files:**
- Create: `crates/ruse-core/tests/tool_call_accumulator.rs`

- [x] **Step 1: Add valid call test**

Add `tool_call_accumulator_accepts_valid_complete_call`.

```rust
let result = ToolCallAccumulator::default().push_tool_call(
    "call-1",
    "workspace.read_file",
    serde_json::json!({ "path": "README.md" }),
);
assert!(matches!(result, AccumulatedToolCall::Ready(_)));
```

- [x] **Step 2: Add duplicate test**

Add `tool_call_accumulator_reports_duplicate_call_id`.

```rust
let mut accumulator = ToolCallAccumulator::default();
let _ = accumulator.push_tool_call("call-1", "workspace.read_file", serde_json::json!({}));
let duplicate = accumulator.push_tool_call("call-1", "workspace.read_file", serde_json::json!({}));
assert!(matches!(duplicate, AccumulatedToolCall::Observation(_)));
```

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-core --test tool_call_accumulator
```

Expected: fails because accumulator contracts do not exist.

### Task 2: Implement Accumulator Contracts

**Files:**
- Modify: `crates/ruse-core/src/tool_loop.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add `ToolObservationMessage`**

Add a serializable observation type with `call_id`, `tool_name`, `observation`, and `truncated`.

- [x] **Step 2: Add `AccumulatedToolCall`**

Add an enum with `Ready(ToolCallRequest)` and `Observation(ToolObservationMessage)`.

- [x] **Step 3: Add `ToolCallAccumulator`**

Track seen call ids in a `BTreeSet<String>` and expose `push_tool_call`.

### Task 3: Validate Tool Names and Inputs

**Files:**
- Modify: `crates/ruse-core/src/tool_loop.rs`

- [x] **Step 1: Validate tool names**

Use `ToolName::new(tool_name)` before returning `Ready`.

- [x] **Step 2: Validate JSON object shape**

If input is not a JSON object, return an observation explaining the invalid input shape.

- [x] **Step 3: Redact observation text**

Ensure observation strings do not include `sk-`, `api_key`, `token`, or `password` values.

### Task 4: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-core --test tool_call_accumulator
cargo test -p ruse-core --test comprehensive_turn_runner
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
git commit -m "feat: accumulate streaming tool calls"
```
