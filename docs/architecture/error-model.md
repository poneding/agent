# Error Model

Errors in `agent` must be useful to users, recoverable by the runtime when possible, and stable enough for tests and replay.

## Error Layers

| Layer | Error Shape |
|---|---|
| Internal crate logic | crate-specific Rust error enums |
| Runtime events | serializable `AgentError` data |
| CLI/Desktop | user-facing diagnostics |
| Audit | stable code, actor, resource, decision, reason |

Internal errors can be rich. Durable event errors must be stable.

## Stable Error Data

Runtime-facing errors use:

```rust
pub struct AgentError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
```

Stable codes use namespaced strings:

```text
model.timeout
tool.denied
storage.event_conflict
policy.approval_required
sandbox.process_killed
replay.schema_mismatch
```

## Recoverability

| Kind | Runtime Behavior |
|---|---|
| Recoverable | persist error, keep task resumable, expose next action |
| Approval-required | persist waiting state, notify UI, resume after decision |
| Retryable | apply retry policy and record attempt |
| Terminal | persist failure, stop current turn/task |

## Diagnostic Rules

- Error messages are human-readable.
- Error codes are stable and testable.
- Sensitive values are redacted before events, logs, traces, and snapshots.
- Source errors can be chained internally, but durable events store stable summaries.
- Panics are bugs; runtime paths return errors.

## Testing Requirements

- Every public error conversion has a test.
- Recoverable errors produce resumable states.
- Terminal errors produce terminal states.
- Redaction tests cover secrets, environment variables, command arguments, and tool output snippets.
