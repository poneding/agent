# M17-02 OpenAI Reqwest Transport Spec

## Purpose

Add a real OpenAI-compatible HTTP transport backed by `reqwest` while preserving fixture-only tests and secret-safe diagnostics.

## Product Context

M13 introduced OpenAI-compatible request and SSE fixtures. M17-01 makes providers truly stream. This slice adds the production transport implementation that can POST to `/responses`, resolve secrets at request time, and parse streamed SSE bytes.

## Scope

This slice delivers:

- `ReqwestOpenAiCompatibleTransport`
- request URL, header, timeout, and body construction
- authorization header injection from `SecretResolver`
- SSE byte/line parsing into provider-local stream events
- HTTP status classification and redacted provider errors
- deterministic tests using fixture or local in-process HTTP streams

Out of scope:

- CLI provider registration
- retry/fallback orchestration
- live external network tests

## Architecture

`ruse-model::openai_compatible` owns both fixture and reqwest transports. The provider receives an `Arc<dyn OpenAiCompatibleTransport>` so tests continue to use fixtures. The reqwest transport owns HTTP client construction and secret resolution; provider code maps transport output to `ModelStreamEvent`.

Recommended choices:

- `reqwest` with `rustls-tls`, `json`, and `stream`
- manual SSE parsing over response bytes to avoid a provider-specific eventsource dependency
- `SecretResolver` injected into the transport, not stored in request debug output

## State And Failure Handling

- Missing secrets fail before a network request.
- 401/403/400 invalid request errors are terminal.
- 408/429/5xx and transport timeouts are retryable.
- Response bodies are size-limited and redacted before entering errors.
- Malformed SSE emits an `InvalidStreamEvent` error.

## Security And Privacy

Raw secret values never appear in `Debug`, runtime events, audit logs, or error messages. URLs are redacted if credentials or query secrets are present. Raw SSE frames are not persisted by default.

## Testing Strategy

- reqwest request builder includes the expected URL and JSON body
- authorization header is present in the HTTP request but not in debug output
- fixture/local HTTP stream maps deltas and completion incrementally
- error body redaction covers `sk-`, `api_key`, `token`, and `password`
- tests do not call external services

## Acceptance Criteria

- A real reqwest transport can stream OpenAI-compatible responses.
- Secret injection is late and redacted.
- All tests are deterministic and offline.

## Completion Signal

M17-02 is complete when reqwest transport tests pass and workspace checks are green without credentials.
