# M16-02 Prompt Overlays and Diagnostics Spec

## Purpose

Implement deterministic prompt assembly rules for overlays, duplicate detection, required-layer enforcement, budget allocation, and prompt diagnostics.

## Product Context

After M16-01 defines profiles and metadata, the runtime needs to explain how built-in, project, user, skill, memory, team, and session layers are merged into one effective prompt. This slice makes prompt assembly auditable before prompt snapshots or UI summaries are added.

## Scope

This slice delivers:

- `PromptOverlay`
- `PromptOverlayAction`
- `PromptAssemblyRequest`
- `PromptAssemblyReport`
- `PromptDiagnostic`
- `PromptDiagnosticSeverity`
- pure `PromptRuntime` assembly in `ruse-context`
- required and optional layer enforcement
- budget integration and omission diagnostics

Owned by later slices:

- durable prompt snapshot event payloads
- CLI inspection commands
- UI bridge prompt summary payloads

## Architecture

`ruse-context` owns the complete pure assembly algorithm. `PromptRuntime` accepts a profile, candidate layers, overlays, and budget hints, then returns a model-facing `PromptBundle` plus a report containing diagnostics and UI-safe metadata.

Overlay rules must be deterministic and source-aware:

- built-in hidden required layers cannot be removed
- higher-priority overlays can append or replace named non-required layers
- duplicate ids at the same precedence produce diagnostics
- missing required layers fail before model invocation
- required layer budget overflow fails
- optional layer budget overflow omits the layer and records a diagnostic

## Public Contracts

```rust
pub enum PromptOverlayAction {
    Append,
    Replace { target_layer_id: PromptLayerId },
    Disable { target_layer_id: PromptLayerId },
}

pub struct PromptOverlay {
    pub id: String,
    pub priority: i32,
    pub action: PromptOverlayAction,
    pub layer: Option<PromptLayer>,
}

pub struct PromptAssemblyRequest {
    pub profile: PromptProfile,
    pub layers: Vec<PromptLayer>,
    pub overlays: Vec<PromptOverlay>,
}

pub struct PromptAssemblyReport {
    pub profile_id: PromptProfileId,
    pub profile_version: u32,
    pub included_layer_ids: Vec<PromptLayerId>,
    pub omitted_layer_ids: Vec<PromptLayerId>,
    pub estimated_tokens: u32,
    pub diagnostics: Vec<PromptDiagnostic>,
}

pub struct PromptDiagnostic {
    pub severity: PromptDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub layer_id: Option<PromptLayerId>,
}
```

## State And Failure Handling

- Missing required layers return `ContextError::InvalidLayer`.
- Duplicate required layer ids return `ContextError::InvalidLayer`.
- Duplicate optional layer ids record diagnostics and keep the highest-priority deterministic winner.
- Invalid overlays return `ContextError::InvalidLayer`.
- Required layer overflow returns `ContextError::BudgetExceeded`.
- Optional layer overflow records an omission diagnostic.

## Security And Privacy

Diagnostic messages must mention ids, source labels, and policy outcomes only. They must not echo hidden layer content or secret-like layer content. Prompt files, skill prompts, memory records, and overlays remain untrusted input.

## Testing Strategy

- required layers are enforced before bundle creation
- overlays append layers deterministically
- overlays replace optional named layers deterministically
- overlays cannot remove hidden required layers
- duplicate optional layer ids produce diagnostics
- optional budget overflow records omission diagnostics
- diagnostic serialization excludes prompt content

## Acceptance Criteria

- `ruse-context` can assemble an effective prompt from a profile, layers, and overlays.
- Assembly reports explain included, omitted, and conflicting layers.
- Prompt diagnostics are deterministic and safe for CLI/Desktop display.
- Later snapshot and UI slices can consume report metadata without re-running assembly.

## Completion Signal

M16-02 is complete when focused `ruse-context` overlay and diagnostics tests pass, workspace gates pass, and active implementation advances to M16-03.
