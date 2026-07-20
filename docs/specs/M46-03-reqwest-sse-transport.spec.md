# M46-03 Reqwest SSE Transport Spec

## Purpose

Teach the live reqwest HTTP MCP transport to classify `text/event-stream` responses and return bounded SSE response mode data.

## Product Context

After M46-01 and M46-02, the live transport can safely preserve JSON behavior while accepting SSE response bodies from MCP servers.

## Scope

This spec delivers:

- content-type classification for JSON versus SSE
- bounded reading of SSE response bodies
- local HTTP fixture tests for `text/event-stream`
- safe diagnostics for unsupported content types and body read failures

Owned by other specs:

- parser contracts in M46-01
- response-mode mapping in M46-02
- client dispatch in M46-04

## Architecture

`McpClientHttpReqwestTransport` returns the typed response-mode contract while preserving the transport trait boundary.

## Public Contracts

The transport trait may evolve from returning `McpClientHttpResponse` to returning `McpClientHttpResponseMode`.

## State and Failure Handling

- 2xx status is required before content parsing.
- Missing or JSON content type defaults to JSON for M45 compatibility.
- `text/event-stream` reads a bounded body in M46; true incremental streaming is later work.
- Unsupported content types fail safely without exposing response body.

## Security and Privacy

- Response bodies are untrusted and never echoed in diagnostics.
- Header values are not surfaced.

## Testing Strategy

- local HTTP fixture for `text/event-stream`
- JSON compatibility test
- unsupported content type safe error test
- oversized body test

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_transport`
- focused clippy for `ruse-mcp`

## Completion Signal

M46-03 is complete when reqwest can return JSON and SSE response modes with bounded, redacted behavior.
