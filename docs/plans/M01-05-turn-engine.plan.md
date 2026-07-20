# M01-05 Turn Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a minimal turn state machine that consumes user input, streams fake model output, emits runtime events, and reaches a final response.

**Architecture:** `ruse-core` owns turn orchestration. It accepts typed input, builds minimal context, calls a `ModelProvider`, emits `RuntimeEvent` values, and returns a `TurnOutcome`. It does not write to storage in this slice; persistence is covered by M01-06 integration.

**Tech Stack:** Rust 2024 edition, `ruse-types`, `ruse-model`, `thiserror`, `tokio` for async tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Core Dependencies and Modules

**Files:**
- Modify: `crates/ruse-core/Cargo.toml`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add dependencies**

Add:

```toml
[dependencies]
ruse-model = { path = "../ruse-model" }
ruse-storage = { path = "../ruse-storage" }
ruse-types = { path = "../ruse-types" }
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
```

- [x] **Step 2: Export modules**

`src/lib.rs` should export:

```rust
#![forbid(unsafe_code)]
//! Core runtime orchestration for agent.

pub mod context;
pub mod runtime;
pub mod turn;

pub use context::ContextAssembler;
pub use runtime::RuntimeIds;
pub use turn::{TurnEngine, TurnError, TurnInput, TurnOutcome};
```

### Task 2: Define Runtime IDs and Context Assembly

**Files:**
- Create: `crates/ruse-core/src/runtime.rs`
- Create: `crates/ruse-core/src/context.rs`

- [x] **Step 1: Define runtime identity bundle**

```rust
use ruse_types::{ConversationId, CorrelationId, TurnId, WorkspaceId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RuntimeIds {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
}
```

- [x] **Step 2: Define minimal context assembler**

```rust
#[derive(Clone, Debug, Default)]
pub struct ContextAssembler;

impl ContextAssembler {
    #[must_use]
    pub fn assemble(&self, user_input: &str) -> Vec<String> {
        vec![user_input.to_string()]
    }
}
```

### Task 3: Implement Turn Engine

**Files:**
- Create: `crates/ruse-core/src/turn.rs`
- Create: `crates/ruse-core/tests/turn_engine.rs`

- [x] **Step 1: Define turn input and outcome**

```rust
use ruse_model::{ModelProvider, ModelRequest, ModelStreamEvent};
use ruse_types::{
    ActorRef, FinalResponseProduced, ModelOutputDelta, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, Timestamp, TurnCompleted, TurnStarted, TurnState, TurnStateChanged,
    UserInputReceived,
};
use thiserror::Error;

use crate::context::ContextAssembler;
use crate::runtime::RuntimeIds;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    pub provider: String,
    pub model: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutcome {
    pub final_message: String,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Error)]
pub enum TurnError {
    #[error("model stream failed: {message}")]
    Model {
        message: String,
        events: Vec<RuntimeEvent>,
    },
}
```

- [x] **Step 2: Implement `TurnEngine::run`**

`TurnEngine::run` should:

- emit `UserInputReceived`
- emit `TurnStarted`
- emit state changes from `Accepted` to `ContextBuilt` to `ModelStreaming`
- call `ModelProvider::stream`
- aggregate `ModelStreamEvent::Delta`
- emit `ModelOutputDelta` per delta
- emit `FinalResponseProduced`
- emit `TurnCompleted` with `Done`
- return `TurnError::Model` with failure events when the provider fails

- [x] **Step 3: Add turn engine tests**

Tests must prove:

- a turn starts from accepted input
- state changes are emitted in order
- model deltas aggregate into final response
- model error produces a `TurnError::Model`
- completed turn reaches `TurnState::Done`

Run:

```bash
cargo test -p ruse-core turn_engine_aggregates_fake_model_output
```

Expected: turn engine tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run standard checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands pass.

- [x] **Step 2: Advance active focus**

Update active focus from `M01-05-turn-engine` to `M01-06-spine-integration`.

- [x] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m01-05 turn engine"
```

Expected: commit succeeds.
