# agent Milestones

## Planning Principle

Build the runtime spine first. The first working product should prove that a model-driven turn can be streamed, persisted, permission-checked, tool-executed, audited, and replayed from one shared Rust core. CLI and Desktop then become clients of the same runtime truth.

## Document Layout

Milestone coordination lives here. Detailed specs and implementation plans live in sibling directories under `docs/`.

```text
docs/
  architecture/
  milestones/
    README.md
  specs/
    M01-runtime-spine.spec.md
    M01-01-workspace-scaffold.spec.md
  plans/
    M01-runtime-spine.plan.md
    M01-01-workspace-scaffold.plan.md
  adr/
  templates/
```

Use milestone-prefixed filenames for milestone-level planning, such as `M01-runtime-spine.spec.md` and `M01-runtime-spine.plan.md`. Use slice-prefixed filenames for implementation slices inside a milestone, such as `M01-01-workspace-scaffold.spec.md` and `M01-01-workspace-scaffold.plan.md`.

Milestone-level spec/plan pairs are required for every planned milestone. Slice-level spec/plan pairs are required when a milestone is actively decomposed for implementation; M01 is decomposed now because it establishes the runtime spine.

## Planning Status

| Scope | Status |
|---|---|
| Architecture guidance | Complete for crate boundaries, event sourcing, async, errors, config, storage, security, plugins, extensibility, and testing |
| Templates | Complete for specs, plans, ADRs, and plugin manifests |
| ADRs | Initial decisions recorded for Rust workspace, event sourcing, SQLite-first storage, and Tokio |
| Milestone-level specs | Complete for M01-M12 |
| Milestone-level plans | Complete for M01-M12 |
| M01 slice specs | Complete for M01-01 through M01-06 |
| M01 slice plans | Complete for M01-01 through M01-06 |
| M01 implementation | Complete |
| M02 slice specs | Complete for M02-01 through M02-05 |
| M02 slice plans | Development-ready for M02-01 through M02-05 |

## Milestone Spec and Plan Index

| Milestone | Spec | Plan | Goal |
|---|---|---|---|
| M01 Runtime Spine MVP | `docs/specs/M01-runtime-spine.spec.md` | `docs/plans/M01-runtime-spine.plan.md` | Create the smallest event-sourced Rust runtime that can execute and replay a deterministic agent turn. |
| M02 Tool, Policy, and Audit Kernel | `docs/specs/M02-tool-policy-audit.spec.md` | `docs/plans/M02-tool-policy-audit.plan.md` | Add controlled tool execution through one registry, one permission path, and one audit trail. |
| M03 Local Coding Loop | `docs/specs/M03-local-coding-loop.spec.md` | `docs/plans/M03-local-coding-loop.plan.md` | Make the runtime useful in a local repository through an interactive CLI coding loop. |
| M04 Checkpoint, Recovery, and Trace Replay | `docs/specs/M04-checkpoint-recovery-trace.spec.md` | `docs/plans/M04-checkpoint-recovery-trace.plan.md` | Make interrupted turns durable, inspectable, and replayable without duplicating completed side effects. |
| M05 Context and Memory MVP | `docs/specs/M05-context-memory.spec.md` | `docs/plans/M05-context-memory.plan.md` | Add bounded context assembly and scoped memory while preserving reproducible runtime behavior. |
| M06 JSON Mode and UI Bridge | `docs/specs/M06-json-ui-bridge.spec.md` | `docs/plans/M06-json-ui-bridge.plan.md` | Expose the shared runtime to CLI JSON mode and future Desktop clients through stable command and event schemas. |
| M07 MCP and Skills | `docs/specs/M07-mcp-skills.spec.md` | `docs/plans/M07-mcp-skills.plan.md` | Add standard external capability discovery through MCP servers and skill roots. |
| M08 Background Tasks and Scheduler | `docs/specs/M08-scheduler-background-tasks.spec.md` | `docs/plans/M08-scheduler-background-tasks.plan.md` | Support durable long-running and recurring work with leases, retries, quotas, and notifications. |
| M09 Worktree Isolation and Subagents | `docs/specs/M09-worktree-subagents.spec.md` | `docs/plans/M09-worktree-subagents.plan.md` | Enable safe parallel work through isolated worktrees, conflict detection, reduced-capability subagents, and a team blackboard. |
| M10 Desktop Preview | `docs/specs/M10-desktop-preview.spec.md` | `docs/plans/M10-desktop-preview.plan.md` | Provide a visual Desktop client over the shared runtime bridge. |
| M11 Plugin Runtime | `docs/specs/M11-plugin-runtime.spec.md` | `docs/plans/M11-plugin-runtime.plan.md` | Open the runtime to controlled third-party extension through manifests, process isolation, declared capabilities, tools, hooks, and a WASM prototype. |
| M12 Production Hardening | `docs/specs/M12-production-hardening.spec.md` | `docs/plans/M12-production-hardening.plan.md` | Make the runtime reliable for daily use and external integration with migrations, telemetry, audit export, memory portability, platform reports, and benchmarks. |

## M01 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M01-01 Workspace Scaffold | `docs/specs/M01-01-workspace-scaffold.spec.md` | `docs/plans/M01-01-workspace-scaffold.plan.md` | Cargo workspace, runtime-spine crate skeletons, and verification baseline. |
| M01-02 Event Model | `docs/specs/M01-02-event-model.spec.md` | `docs/plans/M01-02-event-model.plan.md` | Typed IDs, event envelope, event payloads, state enums, and serialization rules. |
| M01-03 Storage and Replay | `docs/specs/M01-03-storage-replay.spec.md` | `docs/plans/M01-03-storage-replay.plan.md` | In-memory event store, SQLite event store, and deterministic replay API. |
| M01-04 Fake Model Provider | `docs/specs/M01-04-fake-model.spec.md` | `docs/plans/M01-04-fake-model.plan.md` | Scripted streaming provider for repeatable runtime tests. |
| M01-05 Turn Engine | `docs/specs/M01-05-turn-engine.spec.md` | `docs/plans/M01-05-turn-engine.plan.md` | Minimal turn state machine that emits events and reaches a final response. |
| M01-06 Spine Integration | `docs/specs/M01-06-spine-integration.spec.md` | `docs/plans/M01-06-spine-integration.plan.md` | End-to-end fake turn test with persisted events and replayed final state. |

## M02 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M02-01 Tool Registry | `docs/specs/M02-01-tool-registry.spec.md` | `docs/plans/M02-01-tool-registry.plan.md` | Tool contracts, registry lookup, and duplicate-safe registration. |
| M02-02 Policy Engine | `docs/specs/M02-02-policy-engine.spec.md` | `docs/plans/M02-02-policy-engine.plan.md` | Permission requests, allow/deny/ask decisions, and leases. |
| M02-03 Read-Only Built-ins | `docs/specs/M02-03-read-only-builtins.spec.md` | `docs/plans/M02-03-read-only-builtins.plan.md` | Sandboxed read-only file and search tools. |
| M02-04 Audit Trail | `docs/specs/M02-04-audit-trail.spec.md` | `docs/plans/M02-04-audit-trail.plan.md` | Tool and permission audit payloads in runtime events. |
| M02-05 Tool Loop Integration | `docs/specs/M02-05-tool-loop-integration.spec.md` | `docs/plans/M02-05-tool-loop-integration.plan.md` | Fake model tool call through validation, policy, execution, audit, and observation. |

## Later Milestone Slice Candidates

These slices guide future decomposition. Their milestone-level specs and plans already exist; slice-level specs/plans should be created when the milestone becomes active.

### M03 Local Coding Loop

| Slice | Outcome |
|---|---|
| M03-01 CLI TTY Shell | Interactive CLI with streaming output and slash command foundation. |
| M03-02 File Edit Tool | Diff-oriented file write/edit flow with approval points. |
| M03-03 Shell Tool | Command execution with cwd, timeout, environment allowlist, and captured output. |
| M03-04 Git Tools | Status and diff inspection tools. |
| M03-05 Local Coding E2E | Agent inspects a repo, edits a file, runs an approved command, and reports results. |

### M04 Checkpoint, Recovery, and Trace Replay

| Slice | Outcome |
|---|---|
| M04-01 Checkpoint Store | Durable snapshots of turn/task state. |
| M04-02 Idempotency | Idempotency keys for side-effecting tool calls. |
| M04-03 Recovery Flow | Resume interrupted turns from checkpoint plus event log. |
| M04-04 Trace Commands | CLI commands for inspecting events, tool calls, policy decisions, and final state. |
| M04-05 Replay Compatibility | Replay tests and schema compatibility checks. |

### M05 Context and Memory MVP

| Slice | Outcome |
|---|---|
| M05-01 Prompt Layers | Deterministic prompt layer assembly. |
| M05-02 Budget Planner | Token budget allocation and priority ordering. |
| M05-03 Pinning and Compact | Pinned constraints plus rolling summary compaction. |
| M05-04 Conversation Memory | Conversation-scoped memory records. |
| M05-05 Project Memory | Project-scoped memory with audit and deletion. |

### M06 JSON Mode and UI Bridge

| Slice | Outcome |
|---|---|
| M06-01 UI Event Schema | Stable event schema for UI clients. |
| M06-02 UI Command Schema | Stable command schema for UI clients. |
| M06-03 CLI JSON Mode | JSON event stream mode over runtime events. |
| M06-04 Bridge Snapshots | Snapshot tests for UI-facing runtime events. |

### M07 MCP and Skills

| Slice | Outcome |
|---|---|
| M07-01 MCP Client | Server configuration, tool discovery, and health checks. |
| M07-02 MCP Policy Mapping | MCP tools mapped into local capability checks. |
| M07-03 Skill Discovery | Skill roots, metadata, source tracking, and prompt contribution. |
| M07-04 MCP/Skill Traceability | Runtime traces expose source and permission metadata. |

### M08 Background Tasks and Scheduler

| Slice | Outcome |
|---|---|
| M08-01 Task State Machine | Durable task lifecycle and transitions. |
| M08-02 Queue and Leases | Local durable queue, leases, cancellation, pause/resume, and retries. |
| M08-03 Quotas | Time, concurrency, model usage, and tool class limits. |
| M08-04 Notifications | Waiting-approval and completion notification events. |

### M09 Worktree Isolation and Subagents

| Slice | Outcome |
|---|---|
| M09-01 Worktree Manager | Git worktree creation, cleanup, and status tracking. |
| M09-02 Conflict Detection | File locks, changed-file tracking, and conflict reports. |
| M09-03 Subagent Handoff | Role definition, handoff envelope, and capability reduction. |
| M09-04 Team Blackboard | Shared findings, review output, artifacts, and trace links. |

### M10 Desktop Preview

| Slice | Outcome |
|---|---|
| M10-01 Desktop Shell | Desktop app connected through the UI bridge. |
| M10-02 Conversation and Tasks | Conversation view and task list. |
| M10-03 Permissions and Diffs | Permission dialog and diff review. |
| M10-04 Trace Timeline | Visual model/tool/policy trace timeline. |
| M10-05 Settings | Model, policy, MCP, and memory settings surface. |

### M11 Plugin Runtime

| Slice | Outcome |
|---|---|
| M11-01 Plugin Manifest | Manifest schema, validation, and compatibility checks. |
| M11-02 Process Plugin Host | Process-based plugin execution. |
| M11-03 Plugin Capabilities | Capability and permission declaration validation. |
| M11-04 Plugin Tools and Hooks | Plugin-provided tools and lifecycle hooks. |
| M11-05 WASM Prototype | Constrained WASM plugin host prototype. |

### M12 Production Hardening

| Slice | Outcome |
|---|---|
| M12-01 Schema Migration | Event, checkpoint, memory, config, and plugin schema migrations. |
| M12-02 Telemetry | OpenTelemetry spans, metrics, and structured logs. |
| M12-03 Audit Export | Exportable audit records and trace bundles. |
| M12-04 Memory Portability | Memory import/export and retention controls. |
| M12-05 Platform Capability Report | Cross-platform sandbox capability reporting. |
| M12-06 Benchmarks | Turn latency, event throughput, tool dispatch, and compaction benchmarks. |

## Active Work

Current implementation focus: `M02-03-read-only-builtins`.

Current planning focus: all milestone-level specs and plans are created; the next planning document should be a slice-level doc only when implementation reaches a milestone that needs further decomposition.

Primary execution documents for the current implementation focus:

- `docs/specs/M02-03-read-only-builtins.spec.md`
- `docs/plans/M02-03-read-only-builtins.plan.md`
