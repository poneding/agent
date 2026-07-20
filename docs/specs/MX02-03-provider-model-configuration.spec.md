# MX02-03 Provider and Model Configuration Spec

## Purpose

Make Desktop provider/model configuration a first-class product surface with multiple providers, per-provider model profiles, capability diagnostics, safe secret references, defaults, per-thread overrides, and connection test diagnostics.

## Product Context

M57 introduced multi-provider runtime support and capability diagnostics. M60 exposed a basic settings view. MX02-03 upgrades Desktop configuration so model choice is visible in both settings and the composer without leaking secrets into React/Tauri snapshots.

## Scope

This spec delivers:

- `ProviderProfile` and `ModelProfile` shared contracts
- provider kinds for OpenAI-compatible, Anthropic, Gemini, local/Ollama-style, and fake/test
- settings UI with provider list, model table/cards, add/edit provider dialog, base URL field, secret ref/API key status, default model selector, per-thread model override, and connection test diagnostics
- composer model selector integration
- model capability hints for tools, streaming, vision, context window, output limit, and cost
- capability mismatch diagnostics before a turn fails where possible

Owned by other specs:

- actual provider transport behavior remains in provider/runtime specs
- installer/keychain onboarding remains in M65
- live provider smoke beyond deterministic diagnostics belongs to future provider hardening unless required by this slice

## Architecture

`ruse-model` owns provider/model config validation and secret reference handling. `ruse-ui-bridge` exposes UI-safe snapshots. `ruse-desktop` maps provider/model profiles into Desktop settings view models. Tauri commands load/validate snapshots without raw secret values. React renders settings and composer selectors.

## File Layout

```text
crates/
  ruse-model/src/config.rs
  ruse-desktop/src/settings.rs
  ruse-ui-bridge/src/frame.rs
apps/desktop/src/
  components/SettingsView.tsx
  components/Composer.tsx
  state/settings.ts
  types.ts
  __tests__/trace-settings-ui.test.tsx
apps/desktop/src-tauri/src/settings_commands.rs
```

## Public Contracts

```rust
pub struct ProviderProfile {
    pub id: String,
    pub kind: ProviderKind,
    pub display_name: String,
    pub endpoint: Option<String>,
    pub secret_ref: Option<String>,
    pub enabled: bool,
    pub default_model_id: Option<String>,
    pub models: Vec<ModelProfile>,
    pub diagnostics: Vec<ProviderDiagnostic>,
}

pub struct ModelProfile {
    pub id: String,
    pub display_name: String,
    pub enabled: bool,
    pub capabilities: ModelCapabilities,
    pub context_window: Option<u32>,
    pub output_limit: Option<u32>,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub cost_hint: Option<String>,
}
```

## State and Failure Handling

- Invalid endpoint, missing secret ref, disabled provider, missing model, and capability mismatch produce typed diagnostics.
- Connection tests return bounded redacted diagnostics and never persist raw secrets.
- Per-thread model override falls back to workspace default with visible diagnostics if unavailable.
- React snapshots and tests contain secret refs/status only, never secret values.

## Security and Privacy

- API keys and provider tokens are never serialized to Tauri, React, logs, snapshots, or tests.
- Secret refs may be displayed as safe labels or status indicators only.
- Endpoint diagnostics redact credentials and unsafe URL material.

## Testing Strategy

- `ruse-model` tests for provider/profile validation and secret reference handling
- `ruse-desktop` settings tests for profile view models and diagnostics
- `ruse-ui-bridge` config snapshot tests for stable JSON and redaction
- Tauri settings tests for validation and redaction
- Vitest `trace-settings-ui` for provider/model UI, dialog states, diagnostics, and composer hints

## Acceptance Criteria

- Desktop can show multiple providers and multiple models per provider.
- Settings can add/edit provider metadata without exposing secret values.
- Composer can select provider/model and show capability hints.
- Capability mismatch diagnostics are visible before turn submission where possible.

## Completion Signal

MX02-03 is complete when the focused Rust, Tauri, Vitest, and build checks in the plan pass.
