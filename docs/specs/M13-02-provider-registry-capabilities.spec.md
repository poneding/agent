# M13-02 Provider Registry Capabilities Spec

## Purpose

Add a provider registry and capability negotiation layer so runtime code can select a model provider without knowing implementation details.

## Product Context

The PRD requires provider-agnostic model integration. M13-02 turns provider config from static data into a runtime selection surface for fake, OpenAI-compatible, and future providers.

## Scope

This spec delivers:

- `ModelCapabilities`
- `ModelRequestRequirements`
- `RegisteredModelProvider`
- `ModelProviderRegistry`
- provider lookup, duplicate detection, and capability checks

Owned by later specs:

- retry and fallback decisions
- OpenAI-compatible HTTP adapter
- CLI provider selection

## Architecture

Registry contracts live in `ruse-model`. The registry stores provider metadata and boxed provider handles behind the existing `ModelProvider` trait.

## Public Contracts

```rust
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    pub max_context_tokens: Option<u32>,
}

pub struct ModelRequestRequirements {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    pub min_context_tokens: Option<u32>,
}
```

## Acceptance Criteria

- registry rejects duplicate provider ids
- registry returns provider-not-found errors for unknown ids
- capability negotiation accepts compatible providers
- capability negotiation explains missing capabilities with stable errors
