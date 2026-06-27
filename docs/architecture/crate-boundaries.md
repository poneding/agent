# Crate Boundaries

`agent` is a Rust workspace composed of small crates with explicit ownership. Crates should communicate through typed contracts, not through shared global state or ad hoc strings.

## Dependency Direction

Lower-level crates must not depend on higher-level orchestration or UI crates.

```text
agent-types
  <- agent-model
  <- agent-storage
  <- agent-tool
  <- agent-policy
  <- agent-context
  <- agent-memory
  <- agent-scheduler
  <- agent-core
       <- agent-ui-bridge
       <- agent-cli
       <- agent-tui
       <- agent-desktop
```

Plugin, MCP, sandbox, and telemetry crates integrate through typed interfaces and runtime events.

## Crate Ownership

| Crate | Owns | Does Not Own |
|---|---|---|
| `agent-types` | IDs, event envelopes, shared state enums, serializable public data | Runtime orchestration, I/O, model SDKs |
| `agent-core` | Turn lifecycle, task orchestration, event emission, cancellation wiring | Provider-specific SDKs, UI rendering, direct DB SQL |
| `agent-model` | Model provider traits, requests, streams, fake providers | Tool execution, storage persistence |
| `agent-storage` | Event store, checkpoint store, artifact store, migrations | Business decisions, UI formatting |
| `agent-tool` | Tool trait, registry, tool schemas, tool result contract | Permission policy decisions |
| `agent-policy` | Capability model, decisions, leases, policy evaluation | Process execution |
| `agent-sandbox` | Process/filesystem/network isolation primitives | Agent state machines |
| `agent-context` | Prompt layers, budget planning, compaction contracts | Long-term memory storage |
| `agent-memory` | Memory records, retrieval, retention, import/export | Prompt assembly policy |
| `agent-scheduler` | Durable task queue, leases, retries, quotas | Turn internals |
| `agent-mcp` | MCP client/server/gateway protocol mapping | Local policy bypass |
| `agent-plugin` | Plugin manifest, loading, compatibility, isolation | Core side effects without policy |
| `agent-ui-bridge` | UI command/event protocol | CLI/TUI/Desktop presentation decisions |
| `agent-cli` | binary entrypoint, mode dispatch, batch, JSON, and plain REPL commands | Runtime business logic, full-screen TUI rendering |
| `agent-tui` | full-screen terminal REPL, Ratatui rendering, terminal input and panel state | Runtime business logic, model/tool execution |
| `agent-desktop` | Desktop shell and views | Runtime business logic |

## Public API Rules

- Public data crossing crate boundaries must be typed.
- Durable or wire-facing public data must derive or implement `Serialize` and `Deserialize`.
- Domain IDs use newtypes, not raw `String`.
- Avoid exposing provider-specific request/response types outside `agent-model`.
- Avoid exposing database row structs outside `agent-storage`.
- Avoid exposing UI widgets or CLI renderer types outside UI crates.

## Module Shape

Prefer small modules named by responsibility:

```text
src/
  lib.rs
  ids.rs
  event.rs
  state.rs
  error.rs
```

`lib.rs` should re-export stable public contracts and keep implementation wiring light.

## Dependency Review Checklist

- Does this dependency point from higher-level behavior to lower-level contracts?
- Can this crate be unit-tested without a live model, network, shell, or desktop?
- Does this change introduce a cycle or force a lower-level crate to know about UI/runtime policy?
- Is a new crate justified by ownership, testability, or extension boundaries?
