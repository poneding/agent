# Async Runtime

`agent` uses Tokio for asynchronous runtime paths. Async code must make cancellation, deadlines, streaming, and concurrency boundaries explicit.

## Runtime Rules

- Library crates expose async APIs where operations may wait on I/O, model streams, tool execution, storage, or scheduler leases.
- `agent-cli` and `agent-desktop` own process-level runtime startup.
- `agent-core` receives runtime handles, cancellation tokens, and deadlines from callers.
- Background tasks run under scheduler leases and quotas.

## Cancellation

Every long-running operation must accept cancellation.

Required cancellation points:

- model streaming
- tool execution
- shell process execution
- MCP calls
- plugin calls
- storage transactions that can wait on locks
- scheduler task loops
- context compaction

Cancellation produces a typed runtime event and a recoverable error when work can resume.

## Deadlines and Timeouts

Timeouts are part of policy, not hidden implementation constants.

Timeout scopes:

- turn timeout
- model request timeout
- tool call timeout
- shell command timeout
- plugin call timeout
- MCP call timeout
- background task lease timeout

Timeout errors include the scope, configured duration, elapsed duration, and recoverability.

## Structured Concurrency

Spawned work must have an owner.

Rules:

- Avoid detached tasks in library crates.
- Store join handles in runtime/task state when work outlives a function call.
- Propagate cancellation to child work.
- Join child work before marking parent work complete.
- Use bounded channels for streaming events and outputs.

## Streaming

Model output and tool output should stream through typed events.

Streaming rules:

- Deltas are ordered.
- Final events include finish reason and counts when available.
- UI clients subscribe to runtime events, not provider SDK streams.
- Storage can persist complete streams or compacted stream summaries according to the event policy.

## Concurrency Limits

Concurrency is limited by:

- workspace
- task
- model provider
- tool class
- plugin
- MCP server
- system resource class

Limits are enforced by scheduler and policy layers, not by UI code.
