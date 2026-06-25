# M01-04 Fake Model Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide a scripted streaming model provider for deterministic runtime tests.

**Architecture:** `agent-model` owns provider-neutral request and stream contracts. The fake provider is the first provider implementation and returns scripted stream events without network access.

**Tech Stack:** Rust 2024 edition, `agent-types`, `async-trait`, `serde`, `thiserror`, `tokio` for async tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Model Dependencies and Modules

**Files:**
- Modify: `crates/agent-model/Cargo.toml`
- Modify: `crates/agent-model/src/lib.rs`

- [ ] **Step 1: Add dependencies**

Add:

```toml
[dependencies]
agent-types = { path = "../agent-types" }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
```

- [ ] **Step 2: Export modules**

`src/lib.rs` should export:

```rust
#![forbid(unsafe_code)]
//! Model provider contracts for agent.

pub mod fake;
pub mod provider;
pub mod request;
pub mod stream;

pub use fake::{FakeModelProvider, FakeModelScript};
pub use provider::{ModelError, ModelProvider};
pub use request::ModelRequest;
pub use stream::{ModelStreamEvent, ModelStreamResult};
```

### Task 2: Define Provider Contracts

**Files:**
- Create: `crates/agent-model/src/request.rs`
- Create: `crates/agent-model/src/stream.rs`
- Create: `crates/agent-model/src/provider.rs`

- [ ] **Step 1: Define model request**

```rust
use agent_types::{ConversationId, CorrelationId, TurnId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub provider: String,
    pub model: String,
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
    pub messages: Vec<String>,
}
```

- [ ] **Step 2: Define stream events**

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStreamEvent {
    Started { request_id: String },
    Delta { text: String },
    Completed { finish_reason: String, output_token_count: u32 },
}

pub type ModelStreamResult = Vec<ModelStreamEvent>;
```

- [ ] **Step 3: Define provider trait**

```rust
use async_trait::async_trait;
use thiserror::Error;

use crate::request::ModelRequest;
use crate::stream::ModelStreamResult;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("model script failed: {0}")]
    Script(String),
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn stream(&self, request: ModelRequest) -> Result<ModelStreamResult, ModelError>;
}
```

### Task 3: Implement Fake Provider

**Files:**
- Create: `crates/agent-model/src/fake.rs`
- Create: `crates/agent-model/tests/fake_model.rs`

- [ ] **Step 1: Implement scripted fake provider**

```rust
use async_trait::async_trait;

use crate::provider::{ModelError, ModelProvider};
use crate::request::ModelRequest;
use crate::stream::{ModelStreamEvent, ModelStreamResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeModelScript {
    pub request_id: String,
    pub deltas: Vec<String>,
    pub finish_reason: String,
}

impl FakeModelScript {
    #[must_use]
    pub fn text(text: &str) -> Self {
        Self {
            request_id: "fake-request-1".to_string(),
            deltas: vec![text.to_string()],
            finish_reason: "stop".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FakeModelProvider {
    script: FakeModelScript,
}

impl FakeModelProvider {
    #[must_use]
    pub const fn new(script: FakeModelScript) -> Self {
        Self { script }
    }
}

#[async_trait]
impl ModelProvider for FakeModelProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        let mut events = Vec::with_capacity(self.script.deltas.len() + 2);
        events.push(ModelStreamEvent::Started {
            request_id: self.script.request_id.clone(),
        });
        events.extend(
            self.script
                .deltas
                .iter()
                .cloned()
                .map(|text| ModelStreamEvent::Delta { text }),
        );
        events.push(ModelStreamEvent::Completed {
            finish_reason: self.script.finish_reason.clone(),
            output_token_count: self.script.deltas.iter().map(|d| d.len() as u32).sum(),
        });
        Ok(events)
    }
}
```

- [ ] **Step 2: Add fake model tests**

Tests must prove:

- deltas stream in scripted order
- a completed event is emitted
- an empty script still emits started and completed
- provider implements `Send + Sync`
- request metadata can be constructed from `agent-types` IDs

Run:

```bash
cargo test -p agent-model fake_model_streams_scripted_deltas
```

Expected: fake model tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Run standard checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands pass.

- [ ] **Step 2: Advance active focus**

Update active focus from `M01-04-fake-model` to `M01-05-turn-engine`.

- [ ] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m01-04 fake model"
```

Expected: commit succeeds.
