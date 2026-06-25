# M06-02 UI Command Schema Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI command contracts and validation to `agent-ui-bridge`.

**Architecture:** Command schemas stay in `agent-ui-bridge` and depend only on `agent-types` shared IDs/enums. Commands declare intent; they do not execute runtime work or bypass policy.

**Tech Stack:** Rust 2024 edition, `agent-ui-bridge`, `agent-types`, serde, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write UI Command RED Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/ui_command_schema.rs`

- [x] **Step 1: Add failing command tests**

Create tests named:

- `ui_command_schema_deserializes_start_turn`
- `ui_command_schema_serializes_permission_response`
- `ui_command_schema_rejects_unknown_command_type`
- `ui_command_schema_validates_empty_prompt`
- `ui_command_schema_validates_empty_permission_request`

The tests should import `UiCommandEnvelope`, `UiCommand`, `UiClientInfo`, `UiPermissionDecision`, and `validate_ui_command`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-ui-bridge ui_command_schema
```

Expected: fails because command contracts do not exist.

### Task 2: Implement Command Contracts

**Files:**
- Create: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`

- [x] **Step 1: Add command module**

Define:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiCommandEnvelope {
    pub schema_version: SchemaVersion,
    pub command_id: CorrelationId,
    pub client: UiClientInfo,
    pub command: UiCommand,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiClientInfo {
    pub kind: ClientKind,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiCommand {
    StartTurn {
        workspace_id: WorkspaceId,
        conversation_id: ConversationId,
        prompt: String,
    },
    CancelTurn {
        conversation_id: ConversationId,
        turn_id: TurnId,
        reason: String,
    },
    PermissionResponse {
        request_id: String,
        decision: UiPermissionDecision,
        reason: String,
    },
    TraceRequest {
        conversation_id: ConversationId,
        include_tools: bool,
        include_policy: bool,
        include_checkpoints: bool,
        include_final: bool,
    },
    Ping,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPermissionDecision {
    Allow,
    Deny,
}
```

- [x] **Step 2: Add validation**

Add:

```rust
pub fn validate_ui_command(envelope: &UiCommandEnvelope) -> Result<(), UiBridgeError>;
```

Validation rules:

- `client.version` must not be empty.
- `StartTurn.prompt` must not be empty after trimming.
- `CancelTurn.reason` must not be empty after trimming.
- `PermissionResponse.request_id` and `reason` must not be empty after trimming.

- [x] **Step 3: Export command contracts**

Update `lib.rs` to export:

```rust
pub mod command;

pub use command::{
    validate_ui_command, UiClientInfo, UiCommand, UiCommandEnvelope, UiPermissionDecision,
};
```

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p agent-ui-bridge --test ui_command_schema
```

Expected: all UI command schema tests pass.

### Task 3: Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`
- Modify: `docs/plans/M06-02-ui-command-schema.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Advance active focus**

Update active focus to `M06-03-cli-json-mode`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m06 ui command schema"
```

Expected: commit succeeds with source, tests, and documentation together.
