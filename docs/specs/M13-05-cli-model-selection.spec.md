# M13-05 CLI Model Selection Spec

## Purpose

Wire model provider and model selection into CLI-facing runtime settings without exposing provider transport details.

## Product Context

CLI users need to choose a provider/model for a run. Desktop will later consume the same shared settings through the UI bridge.

## Scope

This spec delivers:

- CLI `--provider` and `--model` parsing
- typed model selection settings
- validation against provider config contracts
- JSON mode output that reports selected provider/model without secrets

Owned by later specs:

- interactive provider management UI
- durable config file loading
- secret-provider prompts

## Architecture

CLI parsing lives in `ruse-cli`. Shared model selection data uses `ruse-model` contracts. CLI output remains UI-safe and does not include raw config secrets.

## Acceptance Criteria

- CLI accepts provider/model flags in batch mode
- invalid empty provider/model values fail with clear diagnostics
- JSON output includes provider id and model id
- output never includes secret reference names unless explicitly needed for diagnostics
