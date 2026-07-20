# M14-05 UI Config Snapshot Spec

## Purpose

Add UI bridge and Desktop-facing config snapshot contracts so visual clients can display effective runtime settings and diagnostics without reading config files or resolving secrets.

## Product Context

Desktop needs the same runtime truth as CLI. Before a native settings screen exists, it needs typed snapshots for model providers, selected model, config sources, and diagnostics.

## Scope

This spec delivers:

- `UiConfigSnapshot`
- `UiModelProviderSummary`
- `UiConfigDiagnostic`
- `UiJsonFrame::ConfigSnapshot`
- Desktop config snapshot view model

Owned by later specs:

- native Desktop settings editor
- plugin/MCP/tool settings snapshots
- live config mutation over UI commands

## Architecture

`ruse-ui-bridge` owns wire-safe snapshots. `ruse-desktop` can wrap them in presentation models. `ruse-core` creates the effective config; UI crates do not load config files.

## Public Contracts

```rust
pub struct UiConfigSnapshot {
    pub selected_provider: String,
    pub selected_model: String,
    pub providers: Vec<UiModelProviderSummary>,
    pub diagnostics: Vec<UiConfigDiagnostic>,
}
```

## State And Failure Handling

- provider summaries are sorted by provider id
- disabled providers are shown as disabled, not silently removed
- diagnostics preserve severity and field paths
- snapshot conversion fails if selected provider/model are empty

## Security And Privacy

- snapshots contain no secret values
- snapshots do not expose raw YAML/TOML or direct `apiKey` values
- tests include secret-like strings and assert absence after serialization

## Testing Strategy

- UI snapshot serializes stable shape
- disabled providers remain visible
- diagnostics convert from core config diagnostics
- Desktop model rejects empty selected provider/model
- JSON frame encodes a config snapshot as one line

## Acceptance Criteria

- Desktop can display model config status through shared UI bridge contracts.
- CLI JSON and Desktop consume the same snapshot type.
- M14 status can be marked complete.

## Completion Signal

M14-05 is complete when focused UI bridge/Desktop tests and workspace gates pass, M14 status is marked complete, and the next milestone focus is recorded.
