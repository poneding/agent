# M16-01 Prompt Profile Contracts Spec

## Purpose

Add the versioned prompt profile and prompt layer metadata contracts that later M16 slices use for overlays, snapshots, CLI inspection, and UI summaries.

## Product Context

`agent-context` already has basic prompt layers and bundles. M16-01 upgrades those contracts so a turn can describe not just prompt text, but where each layer came from, how visible it is, how sensitive it is, which profile required it, and which version affected the model request.

## Scope

This slice delivers:

- `PromptProfileId` and `PromptLayerId`
- `PromptProfile`
- `PromptBudget`
- `PromptLayerSource`
- `PromptLayerVisibility`
- `PromptLayerSensitivity`
- `PromptLayerPolicy`
- version metadata on prompt profiles and layers
- validation helpers for profiles and layers
- stable serde shape for all new contracts

Owned by later slices:

- overlay application and conflict diagnostics
- prompt snapshot hashing and persistence
- CLI prompt inspection commands
- UI prompt summary contracts

## Architecture

All contracts live in `agent-context::prompt`. `agent-context` remains pure and UI-agnostic. It may reuse existing `PromptLayerKind`, `PromptLayer`, `PromptMessage`, `PromptBundle`, and `ContextError`, but it must not depend on `agent-core`, storage, CLI, Desktop, or model provider crates.

The implementation should preserve existing prompt assembly behavior while extending `PromptLayer` with source, visibility, sensitivity, version, and policy metadata.

## Public Contracts

```rust
pub struct PromptProfileId(String);
pub struct PromptLayerId(String);

pub struct PromptBudget {
    pub max_tokens: u32,
    pub reserved_response_tokens: u32,
}

pub struct PromptProfile {
    pub id: PromptProfileId,
    pub version: u32,
    pub display_name: String,
    pub required_layers: Vec<PromptLayerId>,
    pub default_budget: PromptBudget,
}

pub enum PromptLayerSource {
    BuiltIn,
    RuntimeConfig { source: String },
    ProjectFile { path: String, content_hash: String },
    UserPreference { key: String },
    Skill { skill_id: String, source: String },
    Tool { tool_name: String },
    Memory { memory_id: String },
    TeamRole { role_id: String },
    SessionOverlay { command_id: String },
}

pub enum PromptLayerVisibility {
    Hidden,
    SummaryOnly,
    Full,
}

pub enum PromptLayerSensitivity {
    Public,
    Workspace,
    UserPrivate,
    SecretLike,
}

pub enum PromptLayerPolicy {
    Required,
    Optional,
}
```

The exact constructor names can follow local Rust style, but they must validate empty ids, empty labels, zero profile versions, and invalid budgets.

## State And Failure Handling

- Empty profile ids and layer ids return `ContextError::InvalidLayer`.
- Empty display names return `ContextError::InvalidLayer`.
- Profile version `0` returns `ContextError::InvalidLayer`.
- Required layer ids must be unique within a profile.
- `PromptBudget` rejects `reserved_response_tokens >= max_tokens`.
- Secret-like layers may be represented, but later UI slices must never render their content.

## Security And Privacy

Contracts preserve sensitivity metadata from the beginning. `PromptLayerVisibility::Hidden` and `PromptLayerSensitivity::SecretLike` are explicit data, not naming conventions. Serialization must not redact by itself; redaction belongs to diagnostic and UI summary layers.

## Testing Strategy

- profile constructor accepts valid ids, versions, required layers, and budget
- profile constructor rejects empty profile id
- profile constructor rejects duplicate required layers
- layer constructors reject empty ids and empty content
- `PromptLayerSource`, visibility, sensitivity, and policy serialize with snake-case enum names
- prompt bundle assembly keeps deterministic ordering after metadata fields are added

## Acceptance Criteria

- `agent-context` exposes versioned prompt profile contracts.
- Existing prompt bundle assembly tests remain green after metadata additions.
- New contracts have stable serde tests.
- Later M16 slices can reference prompt source, visibility, sensitivity, policy, and version without adding new foundational types.

## Completion Signal

M16-01 is complete when `cargo test -p agent-context --test prompt_profiles` and `cargo test -p agent-context --test prompt_layers` pass, workspace gates pass, and M16 planning advances to M16-02.
