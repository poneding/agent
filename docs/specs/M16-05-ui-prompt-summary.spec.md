# M16-05 UI Prompt Summary Spec

## Purpose

Expose prompt profile, layer, diagnostic, and snapshot metadata through UI bridge contracts shared by future TUI and Desktop clients.

## Product Context

M16-04 gives CLI users prompt inspection. M16-05 formalizes the UI-safe summary contract so terminal TUI and Desktop can render the same prompt composition without learning `ruse-context` internals or receiving hidden prompt content.

## Scope

This slice delivers:

- `UiPromptSummary`
- `UiPromptLayerSummary`
- `UiPromptDiagnosticSummary`
- UI event or JSON frame support for prompt summaries
- runtime event normalization for prompt snapshot metadata
- bridge snapshot tests for redaction and stable JSON shape

Owned by later milestones:

- visual TUI prompt panel
- Desktop prompt inspector
- prompt editing and profile management UI

## Architecture

`ruse-ui-bridge` owns UI-safe view models and runtime event normalization. It may depend on `ruse-types` payloads and mirror `ruse-context` summary concepts as strings, but it must not expose raw hidden prompt content. CLI can reuse the bridge payload after this slice, but the bridge remains independent from CLI rendering.

## Public Contracts

```rust
pub struct UiPromptSummary {
    pub profile_id: String,
    pub profile_version: u32,
    pub display_name: String,
    pub bundle_hash: String,
    pub estimated_tokens: u32,
    pub message_count: u32,
    pub layers: Vec<UiPromptLayerSummary>,
    pub diagnostics: Vec<UiPromptDiagnosticSummary>,
}

pub struct UiPromptLayerSummary {
    pub layer_id: String,
    pub kind: String,
    pub source_label: String,
    pub priority: i32,
    pub token_estimate: u32,
    pub visibility: String,
    pub sensitivity: String,
    pub content_preview: Option<String>,
}
```

Hidden and secret-like layers must set `content_preview` to `None`.

## State And Failure Handling

- Missing optional prompt summary fields normalize to diagnostics rather than panics.
- Runtime events without prompt metadata continue to normalize as before.
- Unknown prompt diagnostic severities normalize to bridge diagnostics only after explicit mapping is added.

## Security And Privacy

Bridge payloads are UI-facing. They must not contain hidden instructions, secret-like content, raw memory content marked private, raw environment values, or raw provider secrets. Source ids, hashes, counts, and redacted labels are allowed.

## Testing Strategy

- UI prompt summary serializes to stable snake-case JSON
- hidden layer summary omits content preview
- secret-like layer summary omits content preview
- runtime `PromptSnapshotRecorded` normalizes to a UI prompt summary or prompt progress event
- JSON frame encoding remains single-line
- bridge snapshots exclude raw prompt content

## Acceptance Criteria

- TUI and Desktop can consume one shared prompt summary contract.
- CLI JSON mode can reuse the same bridge payload.
- UI prompt summaries preserve auditability without leaking hidden or sensitive content.

## Completion Signal

M16-05 is complete when focused UI bridge tests pass, workspace gates pass, M16 is marked complete, and active implementation advances to M17.
