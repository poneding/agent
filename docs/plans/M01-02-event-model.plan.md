# M01-02 Event Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define typed IDs, event envelope, event payloads, state enums, and JSON serialization rules in `ruse-types`.

**Architecture:** `ruse-types` owns all durable event contracts and has no dependency on runtime orchestration, storage engines, model SDKs, CLI rendering, or Tokio. The event model is split into focused modules: IDs and sequences, schema version, timestamp, actor, state, error, and event payloads.

**Tech Stack:** Rust 2024 edition, `serde`, `serde_json`, `uuid`, `time`, `thiserror`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Dependencies and Module Exports

**Files:**
- Modify: `crates/ruse-types/Cargo.toml`
- Modify: `crates/ruse-types/src/lib.rs`

- [ ] **Step 1: Add event-model dependencies**

Modify `crates/ruse-types/Cargo.toml`:

```toml
[package]
name = "ruse-types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
time = { version = "0.3", features = ["formatting", "parsing", "serde"] }
uuid = { version = "1", features = ["serde", "v4"] }

[dev-dependencies]
serde_json = "1"

[lints]
workspace = true
```

- [ ] **Step 2: Export event model modules**

Replace `crates/ruse-types/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Shared durable contracts for agent.

pub mod actor;
pub mod error;
pub mod event;
pub mod ids;
pub mod schema;
pub mod state;
pub mod timestamp;

pub use actor::{ActorKind, ActorRef};
pub use error::{AgentError, AgentTypesError, RuntimeError, RuntimeWarning};
pub use event::*;
pub use ids::*;
pub use schema::SchemaVersion;
pub use state::{TaskState, TurnState};
pub use timestamp::Timestamp;

/// Returns this crate's stable package name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "ruse-types"
}
```

- [ ] **Step 3: Run tests and verify missing modules**

Run:

```bash
cargo test -p ruse-types
```

Expected: compilation fails because the exported modules do not exist yet.

### Task 2: Implement Typed IDs and Event Sequence

**Files:**
- Create: `crates/ruse-types/src/ids.rs`

- [ ] **Step 1: Create ID and sequence types**

Create `crates/ruse-types/src/ids.rs`:

```rust
use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AgentTypesError;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }
    };
}

uuid_id!(WorkspaceId);
uuid_id!(SessionId);
uuid_id!(ConversationId);
uuid_id!(TurnId);
uuid_id!(TaskId);
uuid_id!(AgentId);
uuid_id!(TeamId);
uuid_id!(EventId);
uuid_id!(CorrelationId);
uuid_id!(CheckpointId);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSeq(NonZeroU64);

impl EventSeq {
    pub fn new(value: u64) -> Result<Self, AgentTypesError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(AgentTypesError::InvalidEventSequence(value))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for EventSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_through_string() {
        let original = WorkspaceId::from_uuid(Uuid::nil());
        let parsed: WorkspaceId = original.to_string().parse().expect("valid uuid");
        assert_eq!(parsed, original);
    }

    #[test]
    fn id_serializes_as_uuid_string() {
        let id = EventId::from_uuid(Uuid::nil());
        let json = serde_json::to_string(&id).expect("serialize id");
        assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
    }

    #[test]
    fn event_sequence_rejects_zero() {
        let error = EventSeq::new(0).expect_err("zero sequence is invalid");
        assert_eq!(error.to_string(), "event sequence must be non-zero: 0");
    }

    #[test]
    fn event_sequence_accepts_positive_values() {
        let seq = EventSeq::new(7).expect("positive sequence");
        assert_eq!(seq.get(), 7);
    }
}
```

- [ ] **Step 2: Run focused tests**

Run:

```bash
cargo test -p ruse-types id_round_trips_through_string event_sequence_rejects_zero
```

Expected: tests compile after `AgentTypesError` exists in the next task; until then compilation fails because `crate::error` is missing.

### Task 3: Implement Shared Error, Schema Version, and Timestamp

**Files:**
- Create: `crates/ruse-types/src/error.rs`
- Create: `crates/ruse-types/src/schema.rs`
- Create: `crates/ruse-types/src/timestamp.rs`

- [ ] **Step 1: Create error types**

Create `crates/ruse-types/src/error.rs`:

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AgentTypesError {
    #[error("event sequence must be non-zero: {0}")]
    InvalidEventSequence(u64),
    #[error("schema version must be positive: {0}")]
    InvalidSchemaVersion(u16),
    #[error("timestamp must be UTC")]
    NonUtcTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl AgentError {
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
```

- [ ] **Step 2: Create schema version**

Create `crates/ruse-types/src/schema.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::error::AgentTypesError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const CURRENT_VALUE: u16 = 1;

    #[must_use]
    pub const fn current() -> Self {
        Self(Self::CURRENT_VALUE)
    }

    pub fn new(value: u16) -> Result<Self, AgentTypesError> {
        if value == 0 {
            return Err(AgentTypesError::InvalidSchemaVersion(value));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_schema_version_is_one() {
        assert_eq!(SchemaVersion::current().get(), 1);
    }

    #[test]
    fn schema_version_rejects_zero() {
        let error = SchemaVersion::new(0).expect_err("zero schema version");
        assert_eq!(error.to_string(), "schema version must be positive: 0");
    }
}
```

- [ ] **Step 3: Create timestamp wrapper**

Create `crates/ruse-types/src/timestamp.rs`:

```rust
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

impl Timestamp {
    #[must_use]
    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    #[must_use]
    pub const fn from_utc(value: OffsetDateTime) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_offset_date_time(self) -> OffsetDateTime {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn timestamp_serializes_as_rfc3339() {
        let timestamp = Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC));
        let json = serde_json::to_string(&timestamp).expect("serialize timestamp");
        assert_eq!(json, "\"2026-06-25T00:00:00Z\"");
    }
}
```

- [ ] **Step 4: Run focused tests**

Run:

```bash
cargo test -p ruse-types schema_version_rejects_zero timestamp_serializes_as_rfc3339 event_sequence_rejects_zero
```

Expected: tests pass after the modules compile.

### Task 4: Implement Actor and State Contracts

**Files:**
- Create: `crates/ruse-types/src/actor.rs`
- Create: `crates/ruse-types/src/state.rs`

- [ ] **Step 1: Create actor model**

Create `crates/ruse-types/src/actor.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Agent,
    System,
    Tool,
    Scheduler,
    Plugin,
    McpServer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: String,
    pub display_name: Option<String>,
}

impl ActorRef {
    #[must_use]
    pub fn system() -> Self {
        Self {
            kind: ActorKind::System,
            id: "system".to_string(),
            display_name: Some("system".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_kind_uses_snake_case() {
        let json = serde_json::to_string(&ActorKind::McpServer).expect("serialize actor kind");
        assert_eq!(json, "\"mcp_server\"");
    }
}
```

- [ ] **Step 2: Create turn and task states**

Create `crates/ruse-types/src/state.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnState {
    Accepted,
    Prepared,
    ContextBuilt,
    ModelStreaming,
    ToolProposed,
    PermissionEvaluating,
    ToolExecuting,
    Observed,
    Finalizing,
    Persisted,
    Done,
    WaitingApproval,
    WaitingIo,
    Cancelled,
    Failed,
    Recovered,
}

impl TurnState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled | Self::Failed)
    }

    #[must_use]
    pub const fn is_resumable(self) -> bool {
        matches!(self, Self::WaitingApproval | Self::WaitingIo | Self::Recovered)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Created,
    Queued,
    Running,
    WaitingApproval,
    Sleeping,
    Paused,
    Succeeded,
    Failed,
    Cancelled,
    Expired,
}

impl TaskState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled | Self::Expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_state_uses_snake_case() {
        let json = serde_json::to_string(&TurnState::ModelStreaming).expect("serialize state");
        assert_eq!(json, "\"model_streaming\"");
    }

    #[test]
    fn terminal_states_are_marked() {
        assert!(TurnState::Done.is_terminal());
        assert!(TaskState::Succeeded.is_terminal());
        assert!(!TaskState::Running.is_terminal());
    }
}
```

- [ ] **Step 3: Run focused tests**

Run:

```bash
cargo test -p ruse-types actor_kind_uses_snake_case turn_state_uses_snake_case terminal_states_are_marked
```

Expected: tests pass.

### Task 5: Implement Runtime Event Payloads and Envelope

**Files:**
- Create: `crates/ruse-types/src/event.rs`

- [ ] **Step 1: Create runtime event contracts**

Create `crates/ruse-types/src/event.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::actor::ActorRef;
use crate::error::AgentError;
use crate::ids::{
    AgentId, CheckpointId, ConversationId, CorrelationId, EventId, EventSeq, TaskId, TeamId,
    TurnId, WorkspaceId,
};
use crate::schema::SchemaVersion;
use crate::state::{TaskState, TurnState};
use crate::timestamp::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTrustLevel {
    Trusted,
    Untrusted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Cli,
    Desktop,
    Api,
    Test,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFinishReason {
    Stop,
    Length,
    Cancelled,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub schema_version: SchemaVersion,
    pub id: EventId,
    pub seq: Option<EventSeq>,
    pub occurred_at: Timestamp,
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub task_id: Option<TaskId>,
    pub agent_id: Option<AgentId>,
    pub team_id: Option<TeamId>,
    pub actor: ActorRef,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub payload: RuntimeEventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum RuntimeEventPayload {
    WorkspaceOpened(WorkspaceOpened),
    SessionStarted(SessionStarted),
    ConversationStarted(ConversationStarted),
    UserInputReceived(UserInputReceived),
    TurnStarted(TurnStarted),
    TurnStateChanged(TurnStateChanged),
    ContextAssembled(ContextAssembled),
    ModelStreamStarted(ModelStreamStarted),
    ModelOutputDelta(ModelOutputDelta),
    ModelStreamCompleted(ModelStreamCompleted),
    FinalResponseProduced(FinalResponseProduced),
    TurnCompleted(TurnCompleted),
    TurnFailed(TurnFailed),
    TaskStateChanged(TaskStateChanged),
    CheckpointRecorded(CheckpointRecorded),
    ReplayStarted(ReplayStarted),
    ReplayCompleted(ReplayCompleted),
    RuntimeWarning(crate::error::RuntimeWarning),
    RuntimeError(crate::error::RuntimeError),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpened {
    pub root_path: String,
    pub trust_level: WorkspaceTrustLevel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionStarted {
    pub client_kind: ClientKind,
    pub client_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationStarted {
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserInputReceived {
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnStarted {
    pub input_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnStateChanged {
    pub from: TurnState,
    pub to: TurnState,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextAssembled {
    pub message_count: u32,
    pub estimated_tokens: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelStreamStarted {
    pub provider: String,
    pub model: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelOutputDelta {
    pub delta: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelStreamCompleted {
    pub finish_reason: ModelFinishReason,
    pub output_token_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinalResponseProduced {
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnCompleted {
    pub final_state: TurnState,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TurnFailed {
    pub state: TurnState,
    pub error: AgentError,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskStateChanged {
    pub task_id: TaskId,
    pub from: TaskState,
    pub to: TaskState,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRecorded {
    pub checkpoint_id: CheckpointId,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayStarted {
    pub source_conversation_id: ConversationId,
    pub source_turn_id: TurnId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayCompleted {
    pub source_conversation_id: ConversationId,
    pub source_turn_id: TurnId,
    pub event_count: u64,
}
```

- [ ] **Step 2: Run payload serialization smoke test**

Run:

```bash
cargo test -p ruse-types
```

Expected: crate tests pass before integration snapshots are added.

### Task 6: Add JSON Snapshot and Round-Trip Tests

**Files:**
- Create: `crates/ruse-types/tests/event_json.rs`

- [ ] **Step 1: Add integration tests for event JSON**

Create `crates/ruse-types/tests/event_json.rs`:

```rust
use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, FinalResponseProduced, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, TurnId, UserInputReceived, WorkspaceId,
};
use time::macros::datetime;
use uuid::Uuid;

fn fixed_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::from_uuid(Uuid::from_u128(1)),
        seq: None,
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(2)),
        conversation_id: ConversationId::from_uuid(Uuid::from_u128(3)),
        turn_id: Some(TurnId::from_uuid(Uuid::from_u128(4))),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::from_uuid(Uuid::from_u128(5)),
        causation_id: None,
        payload,
    }
}

#[test]
fn user_input_event_round_trips_through_json() {
    let event = fixed_event(RuntimeEventPayload::UserInputReceived(UserInputReceived {
        message: "inspect the workspace".to_string(),
    }));

    let json = serde_json::to_string_pretty(&event).expect("serialize event");
    let decoded: RuntimeEvent = serde_json::from_str(&json).expect("deserialize event");

    assert_eq!(decoded, event);
    assert!(json.contains("\"type\": \"user_input_received\""));
}

#[test]
fn final_response_event_has_stable_shape() {
    let event = fixed_event(RuntimeEventPayload::FinalResponseProduced(
        FinalResponseProduced {
            message: "done".to_string(),
        },
    ));

    let value = serde_json::to_value(event).expect("event json value");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["payload"]["type"], "final_response_produced");
    assert_eq!(value["payload"]["data"]["message"], "done");
    assert_eq!(value["occurred_at"], "2026-06-25T00:00:00Z");
}
```

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test -p ruse-types --test event_json
```

Expected: event JSON tests pass.

### Task 7: Run Full Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Run standard verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands pass.

- [ ] **Step 2: Confirm M01-02 acceptance criteria**

Confirm:

```text
ruse-types exposes event model modules through lib.rs.
cargo test -p ruse-types passes.
cargo test --workspace passes.
Representative JSON round-trip and shape tests exist.
No runtime crate outside ruse-types defines duplicate ID, event, actor, or state types.
```

### Task 8: Update Progress and Commit

**Files:**
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Advance active focus**

Update `docs/milestones/README.md` from:

```markdown
Current implementation focus: `M01-02-event-model`.
```

to:

```markdown
Current implementation focus: `M01-03-storage-replay`.
```

- [ ] **Step 2: Commit implementation and docs**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m01-02 event model"
```

Expected: commit succeeds with implementation, tests, and documentation together.
