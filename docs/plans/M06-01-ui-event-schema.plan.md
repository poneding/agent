# M06-01 UI Event Schema Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `ruse-ui-bridge` and expose a stable, UI-safe event schema normalized from `ruse-types::RuntimeEvent`.

**Architecture:** `ruse-ui-bridge` depends on `ruse-types` only for runtime contracts and on serde libraries for wire shape. It must not depend on `ruse-core`, `ruse-cli`, storage, model, tool execution, or Desktop crates.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `serde_json`, `thiserror`, `time` test macros, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold UI Bridge Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-ui-bridge/Cargo.toml`
- Create: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add empty crate scaffold**

Add `crates/ruse-ui-bridge` to workspace members.

Create `crates/ruse-ui-bridge/Cargo.toml`:

```toml
[package]
name = "ruse-ui-bridge"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-types = { path = "../ruse-types" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
time = { version = "0.3", features = ["macros"] }

[lints]
workspace = true
```

Create `src/lib.rs`:

```rust
#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.
```

- [x] **Step 2: Confirm scaffold compiles**

Run:

```bash
cargo test -p ruse-ui-bridge
```

Expected: crate compiles with zero tests.

### Task 2: Write UI Event RED Tests

**Files:**
- Create: `crates/ruse-ui-bridge/tests/ui_event_schema.rs`

- [x] **Step 1: Add failing schema tests**

Create tests covering:

- `ui_event_schema_serializes_model_delta_shape`
- `ui_event_schema_normalizes_final_response`
- `ui_event_schema_excludes_raw_tool_input`
- `ui_event_schema_normalizes_runtime_error_as_diagnostic`

Use deterministic IDs by parsing UUID strings and deterministic timestamps with `time::macros::datetime`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-ui-bridge ui_event_schema
```

Expected: fails because `UiEventEnvelope`, `UiEventPayload`, `UiDiagnosticLevel`, and `normalize_runtime_event` do not exist.

### Task 3: Implement UI Event Contracts

**Files:**
- Create: `crates/ruse-ui-bridge/src/event.rs`
- Create: `crates/ruse-ui-bridge/src/error.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add event module**

Define `UiEventEnvelope`, `UiEventPayload`, `UiDiagnosticLevel`, and `normalize_runtime_event` exactly as specified in `docs/specs/M06-01-ui-event-schema.spec.md`.

Mapping rules:

- `WorkspaceOpened` -> `Workspace`
- `SessionStarted` -> `Session`
- `UserInputReceived` -> `UserMessage`
- `TurnStateChanged` -> `TurnState`
- `ModelOutputDelta` -> `ModelDelta`
- `FinalResponseProduced` -> `FinalResponse`
- `ToolCallProposed` -> `ToolCall` without `input_json`
- `ToolExecutionCompleted` -> `ToolResult { ok: true, summary: "completed" }`
- `ToolExecutionFailed` -> `ToolResult { ok: false, summary: message }`
- `PermissionDecisionRecorded` -> `Permission`
- `CheckpointRecorded` -> `Checkpoint`
- `RuntimeWarning` -> `Diagnostic { level: Warning, recoverable: true }`
- `RuntimeError` -> `Diagnostic { level: Error, recoverable }`
- all other payloads -> `Progress`

- [x] **Step 2: Export contracts**

Update `lib.rs`:

```rust
#![forbid(unsafe_code)]
//! UI bridge contracts for CLI, Desktop, and API clients.

pub mod error;
pub mod event;

pub use error::UiBridgeError;
pub use event::{normalize_runtime_event, UiDiagnosticLevel, UiEventEnvelope, UiEventPayload};
```

- [x] **Step 3: Verify GREEN**

Run:

```bash
cargo test -p ruse-ui-bridge --test ui_event_schema
```

Expected: all UI event schema tests pass.

### Task 4: Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`
- Modify: `docs/plans/M06-01-ui-event-schema.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Advance active focus**

Update active focus to `M06-02-ui-command-schema`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m06 ui event schema"
```

Expected: commit succeeds with source, tests, and documentation together.
