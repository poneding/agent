# M13-04 OpenAI-Compatible Adapter Spec

## Purpose

Add a fixture-backed OpenAI-compatible provider adapter that maps local model requests to provider HTTP payloads and parses streaming fixture events into `ModelStreamEvent`.

## Product Context

This slice delivers the first real-provider adapter boundary required by the MVP without requiring live network calls in tests.

## Scope

This spec delivers:

- `OpenAiCompatibleProvider`
- request JSON builder
- fixture transport trait
- SSE stream parser for synthetic Responses API events
- provider error mapping

Owned by later specs:

- real secret provider implementation
- live network smoke tests
- provider-specific advanced tool schema mapping

## Architecture

The adapter lives under `ruse-model::openai_compatible`. Provider-specific JSON structs remain private. Tests use fixture transports and synthetic SSE chunks.

## Acceptance Criteria

- request builder emits expected `model`, `input`, and `stream` fields
- adapter requires a secret handle at construction
- synthetic SSE deltas map to `ModelStreamEvent::Delta`
- synthetic completed event maps to `ModelStreamEvent::Completed`
- provider error bodies are redacted in `ModelError`
