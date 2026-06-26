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

Milestone-level spec/plan pairs are required for every planned milestone. Slice-level spec/plan pairs are required when a milestone is actively decomposed for implementation.

## Planning Status

| Scope | Status |
|---|---|
| Architecture guidance | Complete for crate boundaries, event sourcing, async, errors, config, storage, security, plugins, extensibility, and testing |
| Templates | Complete for specs, plans, ADRs, and plugin manifests |
| ADRs | Initial decisions recorded for Rust workspace, event sourcing, SQLite-first storage, and Tokio |
| Milestone-level specs | Complete for M01-M14 |
| Milestone-level plans | Complete for M01-M14 |
| M01 slice specs | Complete for M01-01 through M01-06 |
| M01 slice plans | Complete for M01-01 through M01-06 |
| M01 implementation | Complete |
| M02 slice specs | Complete for M02-01 through M02-05 |
| M02 slice plans | Complete for M02-01 through M02-05 |
| M02 implementation | Complete |
| M03 slice specs | Complete for M03-01 through M03-05 |
| M03 slice plans | Complete for M03-01 through M03-05 |
| M03 implementation | Complete |
| M04 slice specs | Complete for M04-01 through M04-05 |
| M04 slice plans | Complete for M04-01 through M04-05 |
| M04 implementation | Complete |
| M05 slice specs | Complete for M05-01 through M05-05 |
| M05 slice plans | Complete for M05-01 through M05-05 |
| M05 implementation | Complete |
| M06 slice specs | Complete for M06-01 through M06-04 |
| M06 slice plans | Complete for M06-01 through M06-04 |
| M06 implementation | Complete |
| M07 slice specs | Complete for M07-01 through M07-04 |
| M07 slice plans | Complete for M07-01 through M07-04 |
| M07-01 implementation | Complete |
| M07-02 implementation | Complete |
| M07-03 implementation | Complete |
| M07-04 implementation | Complete |
| M07 implementation | Complete |
| M08 slice specs | Complete for M08-01 through M08-04 |
| M08 slice plans | Complete for M08-01 through M08-04 |
| M08-01 implementation | Complete |
| M08-02 implementation | Complete |
| M08-03 implementation | Complete |
| M08-04 implementation | Complete |
| M08 implementation | Complete |
| M09 slice specs | Complete for M09-01 through M09-04 |
| M09 slice plans | Complete for M09-01 through M09-04 |
| M09-01 implementation | Complete |
| M09-02 implementation | Complete |
| M09-03 implementation | Complete |
| M09-04 implementation | Complete |
| M09 implementation | Complete |
| M10 slice specs | Complete for M10-01 through M10-05 |
| M10 slice plans | Complete for M10-01 through M10-05 |
| M10-01 implementation | Complete |
| M10-02 implementation | Complete |
| M10-03 implementation | Complete |
| M10-04 implementation | Complete |
| M10-05 implementation | Complete |
| M10 implementation | Complete |
| M11 technical design | Complete |
| M11 slice specs | Complete for M11-01 through M11-05 |
| M11 slice plans | Complete for M11-01 through M11-05 |
| M11-01 implementation | Complete |
| M11-02 implementation | Complete |
| M11-03 implementation | Complete |
| M11-04 implementation | Complete |
| M11-05 implementation | Complete |
| M11 implementation | Complete |
| M12 technical design | Complete |
| M12 slice specs | Complete for M12-01 through M12-06 |
| M12 slice plans | Complete for M12-01 through M12-06 |
| M12-01 implementation | Complete |
| M12-02 implementation | Complete |
| M12-03 implementation | Complete |
| M12-04 implementation | Complete |
| M12-05 implementation | Complete |
| M12-06 implementation | Complete |
| M12 implementation | Complete |
| M13 technical design | Complete |
| M13 slice specs | Complete for M13-01 through M13-05 |
| M13 slice plans | Complete for M13-01 through M13-05 |
| M13-01 implementation | Complete |
| M13-02 implementation | Complete |
| M13-03 implementation | Complete |
| M13-04 implementation | Complete |
| M13-05 implementation | Complete |
| M13 implementation | Complete |
| M14 technical design | Complete |
| M14 slice specs | Complete for M14-01 through M14-05 |
| M14 slice plans | Complete for M14-01 through M14-05 |
| M14-01 implementation | Complete |
| M14-02 implementation | Complete |
| M14-03 implementation | Complete |
| M14-04 implementation | Complete |
| M14-05 implementation | In progress |
| M14 implementation | In progress |

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
| M13 Real Model Providers and Runtime Configuration | `docs/specs/M13-real-model-providers-config.spec.md` | `docs/plans/M13-real-model-providers-config.plan.md` | Add provider configuration, registry, invocation policy, OpenAI-compatible adapter contracts, and CLI model selection. |
| M14 Runtime Configuration and Secrets | `docs/specs/M14-runtime-configuration-secrets.spec.md` | `docs/plans/M14-runtime-configuration-secrets.plan.md` | Load durable runtime config, resolve secret handles safely, layer model settings, and expose CLI/Desktop-safe diagnostics. |

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

## M03 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M03-01 CLI TTY Shell | `docs/specs/M03-01-cli-tty-shell.spec.md` | `docs/plans/M03-01-cli-tty-shell.plan.md` | CLI mode parsing, slash commands, and deterministic batch output. |
| M03-02 File Edit Tool | `docs/specs/M03-02-file-edit-tool.spec.md` | `docs/plans/M03-02-file-edit-tool.plan.md` | Diff-oriented workspace-confined file edit preview and apply tool. |
| M03-03 Shell Tool | `docs/specs/M03-03-shell-tool.spec.md` | `docs/plans/M03-03-shell-tool.plan.md` | Controlled command execution with cwd confinement, timeout, and captured output. |
| M03-04 Git Tools | `docs/specs/M03-04-git-tools.spec.md` | `docs/plans/M03-04-git-tools.plan.md` | Read-only Git status and diff inspection tools. |
| M03-05 Local Coding E2E | `docs/specs/M03-05-local-coding-e2e.spec.md` | `docs/plans/M03-05-local-coding-e2e.plan.md` | Deterministic local coding workflow with edit, shell, Git, policy, audit, and summary. |

## M04 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M04-01 Checkpoint Store | `docs/specs/M04-01-checkpoint-store.spec.md` | `docs/plans/M04-01-checkpoint-store.plan.md` | Durable checkpoint records and in-memory/SQLite checkpoint stores. |
| M04-02 Idempotency | `docs/specs/M04-02-idempotency.spec.md` | `docs/plans/M04-02-idempotency.plan.md` | Idempotency keys, completed tool-effect records, and replay-safe execution wrapper. |
| M04-03 Recovery Flow | `docs/specs/M04-03-recovery-flow.spec.md` | `docs/plans/M04-03-recovery-flow.plan.md` | Recovery plans that resume from checkpoint plus event-log tail. |
| M04-04 Trace Commands | `docs/specs/M04-04-trace-commands.spec.md` | `docs/plans/M04-04-trace-commands.plan.md` | Trace query contracts and deterministic CLI trace summaries. |
| M04-05 Replay Compatibility | `docs/specs/M04-05-replay-compatibility.spec.md` | `docs/plans/M04-05-replay-compatibility.plan.md` | Versioned replay fixtures and compatibility diagnostics. |

## M05 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M05-01 Prompt Layers | `docs/specs/M05-01-prompt-layers.spec.md` | `docs/plans/M05-01-prompt-layers.plan.md` | Deterministic prompt layer assembly in `agent-context`. |
| M05-02 Budget Planner | `docs/specs/M05-02-budget-planner.spec.md` | `docs/plans/M05-02-budget-planner.plan.md` | Token budget allocation, required overflow errors, and optional omission reporting. |
| M05-03 Pinning and Compact | `docs/specs/M05-03-pinning-compact.spec.md` | `docs/plans/M05-03-pinning-compact.plan.md` | Pinned constraints plus deterministic rolling summary compaction. |
| M05-04 Conversation Memory | `docs/specs/M05-04-conversation-memory.spec.md` | `docs/plans/M05-04-conversation-memory.plan.md` | Conversation-scoped memory records with add, retrieve, and delete behavior. |
| M05-05 Project Memory | `docs/specs/M05-05-project-memory.spec.md` | `docs/plans/M05-05-project-memory.plan.md` | Project-scoped memory with audit entries and context integration coverage. |

## M06 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M06-01 UI Event Schema | `docs/specs/M06-01-ui-event-schema.spec.md` | `docs/plans/M06-01-ui-event-schema.plan.md` | UI bridge crate, UI-safe event envelopes, and runtime event normalization. |
| M06-02 UI Command Schema | `docs/specs/M06-02-ui-command-schema.spec.md` | `docs/plans/M06-02-ui-command-schema.plan.md` | UI command envelopes, client metadata, command variants, and validation. |
| M06-03 CLI JSON Mode | `docs/specs/M06-03-cli-json-mode.spec.md` | `docs/plans/M06-03-cli-json-mode.plan.md` | `--json` CLI output mode with newline-delimited UI bridge frames. |
| M06-04 Bridge Snapshots | `docs/specs/M06-04-bridge-snapshots.spec.md` | `docs/plans/M06-04-bridge-snapshots.plan.md` | Stable bridge snapshots and M06 completion verification. |

## M07 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M07-01 MCP Client | `docs/specs/M07-01-mcp-client.spec.md` | `docs/plans/M07-01-mcp-client.plan.md` | Server configuration, tool discovery, and health checks. |
| M07-02 MCP Policy Mapping | `docs/specs/M07-02-mcp-policy-mapping.spec.md` | `docs/plans/M07-02-mcp-policy-mapping.plan.md` | MCP tools mapped into local capability checks. |
| M07-03 Skill Discovery | `docs/specs/M07-03-skill-discovery.spec.md` | `docs/plans/M07-03-skill-discovery.plan.md` | Skill roots, metadata, source tracking, and prompt contribution. |
| M07-04 MCP/Skill Traceability | `docs/specs/M07-04-mcp-skill-traceability.spec.md` | `docs/plans/M07-04-mcp-skill-traceability.plan.md` | Runtime traces expose source and permission metadata. |

## M08 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M08-01 Task State Machine | `docs/specs/M08-01-task-state-machine.spec.md` | `docs/plans/M08-01-task-state-machine.plan.md` | Durable task-run state machine and legal transitions. |
| M08-02 Queue and Leases | `docs/specs/M08-02-queue-leases.spec.md` | `docs/plans/M08-02-queue-leases.plan.md` | In-memory queue, leases, cancellation, pause/resume, and retry decisions. |
| M08-03 Quotas | `docs/specs/M08-03-quotas.spec.md` | `docs/plans/M08-03-quotas.plan.md` | Workspace and system concurrency quota evaluation. |
| M08-04 Notifications | `docs/specs/M08-04-notifications.spec.md` | `docs/plans/M08-04-notifications.plan.md` | Task notifications, runtime events, and UI bridge normalization. |

## M09 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M09-01 Worktree Manager | `docs/specs/M09-01-worktree-manager.spec.md` | `docs/plans/M09-01-worktree-manager.plan.md` | Worktree lease lifecycle contracts and in-memory manager. |
| M09-02 Conflict Detection | `docs/specs/M09-02-conflict-detection.spec.md` | `docs/plans/M09-02-conflict-detection.plan.md` | Deterministic conflict reports for lead/subagent changed-file sets. |
| M09-03 Subagent Handoff | `docs/specs/M09-03-subagent-handoff.spec.md` | `docs/plans/M09-03-subagent-handoff.plan.md` | Subagent profiles, handoff envelopes, and capability reduction. |
| M09-04 Team Blackboard | `docs/specs/M09-04-team-blackboard.spec.md` | `docs/plans/M09-04-team-blackboard.plan.md` | Shared findings, review output, artifacts, and trace links. |

## M10 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M10-01 Desktop Shell | `docs/specs/M10-01-desktop-shell.spec.md` | `docs/plans/M10-01-desktop-shell.plan.md` | Desktop shell session, route, and lifecycle contracts. |
| M10-02 Conversation and Tasks | `docs/specs/M10-02-conversation-tasks.spec.md` | `docs/plans/M10-02-conversation-tasks.plan.md` | Conversation list, task list, and workspace snapshot view models. |
| M10-03 Permissions and Diffs | `docs/specs/M10-03-permissions-diffs.spec.md` | `docs/plans/M10-03-permissions-diffs.plan.md` | Permission dialog and diff review contracts. |
| M10-04 Trace Timeline | `docs/specs/M10-04-trace-timeline.spec.md` | `docs/plans/M10-04-trace-timeline.plan.md` | Ordered, UI-safe runtime trace timeline. |
| M10-05 Settings | `docs/specs/M10-05-settings.spec.md` | `docs/plans/M10-05-settings.plan.md` | Model, policy, memory, MCP, and UI settings snapshot. |

## M11 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M11-00 Plugin Runtime Technical Design | `docs/specs/M11-00-plugin-runtime-technical-design.spec.md` | `docs/plans/M11-00-plugin-runtime-technical-design.plan.md` | Architecture baseline for manifest, compatibility, isolation, capabilities, tools, hooks, WASM, security, and tests. |
| M11-01 Plugin Manifest | `docs/specs/M11-01-plugin-manifest.spec.md` | `docs/plans/M11-01-plugin-manifest.plan.md` | `agent-plugin` crate, manifest schema, validation, and compatibility checks. |
| M11-02 Process Plugin Host | `docs/specs/M11-02-process-plugin-host.spec.md` | `docs/plans/M11-02-process-plugin-host.plan.md` | Process plugin protocol, fixture execution, host errors, timeout, and output limits. |
| M11-03 Plugin Capabilities | `docs/specs/M11-03-plugin-capabilities.spec.md` | `docs/plans/M11-03-plugin-capabilities.plan.md` | Capability declaration validation and policy request mapping. |
| M11-04 Plugin Tools and Hooks | `docs/specs/M11-04-plugin-tools-hooks.spec.md` | `docs/plans/M11-04-plugin-tools-hooks.plan.md` | Plugin tool registration through `ToolRegistry`, shared `ToolLoop` policy path, and hook contracts. |
| M11-05 WASM Prototype | `docs/specs/M11-05-wasm-prototype.spec.md` | `docs/plans/M11-05-wasm-prototype.plan.md` | Constrained WASM module validation and non-privileged host prototype. |

## M12 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M12-00 Production Hardening Technical Design | `docs/specs/M12-00-production-hardening-technical-design.spec.md` | `docs/plans/M12-00-production-hardening-technical-design.plan.md` | M12 crate ownership, contract, privacy, compatibility, and verification baseline. |
| M12-01 Schema Migration | `docs/specs/M12-01-schema-migration.spec.md` | `docs/plans/M12-01-schema-migration.plan.md` | Event, checkpoint, memory, config, and plugin schema migration contracts. |
| M12-02 Telemetry | `docs/specs/M12-02-telemetry.spec.md` | `docs/plans/M12-02-telemetry.plan.md` | Provider-neutral spans, metrics, trace context, redaction, and in-memory sink. |
| M12-03 Audit Export | `docs/specs/M12-03-audit-export.spec.md` | `docs/plans/M12-03-audit-export.plan.md` | Exportable audit records and trace bundles. |
| M12-04 Memory Portability | `docs/specs/M12-04-memory-portability.spec.md` | `docs/plans/M12-04-memory-portability.plan.md` | Memory import/export archives and retention controls. |
| M12-05 Platform Capability Report | `docs/specs/M12-05-platform-capability-report.spec.md` | `docs/plans/M12-05-platform-capability-report.plan.md` | Cross-platform sandbox capability reporting. |
| M12-06 Benchmarks | `docs/specs/M12-06-benchmarks.spec.md` | `docs/plans/M12-06-benchmarks.plan.md` | Deterministic runtime benchmark smoke scenarios. |

## M13 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M13-00 Real Model Providers and Runtime Configuration Technical Design | `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md` | `docs/plans/M13-00-real-model-providers-config-technical-design.plan.md` | Provider config, secret refs, registry, invocation policy, adapter, CLI/Desktop, privacy, and test boundaries. |
| M13-01 Provider Config | `docs/specs/M13-01-provider-config.spec.md` | `docs/plans/M13-01-provider-config.plan.md` | Provider config, endpoint config, secret refs, catalog validation, and stable serde contracts. |
| M13-02 Provider Registry Capabilities | `docs/specs/M13-02-provider-registry-capabilities.spec.md` | `docs/plans/M13-02-provider-registry-capabilities.plan.md` | Registry lookup, duplicate safety, and capability negotiation. |
| M13-03 Invocation Policy | `docs/specs/M13-03-invocation-policy.spec.md` | `docs/plans/M13-03-invocation-policy.plan.md` | Timeout, retry, and fallback decision contracts. |
| M13-04 OpenAI-Compatible Adapter | `docs/specs/M13-04-openai-compatible-adapter.spec.md` | `docs/plans/M13-04-openai-compatible-adapter.plan.md` | Fixture-backed request mapping and streaming parser for OpenAI-compatible Responses API shape. |
| M13-05 CLI Model Selection | `docs/specs/M13-05-cli-model-selection.spec.md` | `docs/plans/M13-05-cli-model-selection.plan.md` | CLI provider/model flags and UI-safe selection output. |

## M14 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M14-00 Runtime Configuration and Secrets Technical Design | `docs/specs/M14-00-runtime-configuration-secrets-technical-design.spec.md` | `docs/plans/M14-00-runtime-configuration-secrets-technical-design.plan.md` | Architecture baseline for secret resolvers, config schema, layering, CLI diagnostics, UI snapshots, privacy, and tests. |
| M14-01 Secret Resolver | `docs/specs/M14-01-secret-resolver.spec.md` | `docs/plans/M14-01-secret-resolver.plan.md` | Redacted secret value and provider-neutral resolver contracts in `agent-model`. |
| M14-02 Config File Loader | `docs/specs/M14-02-config-file-loader.spec.md` | `docs/plans/M14-02-config-file-loader.plan.md` | Schema version 1 TOML config loading into model catalog and invocation policy contracts. |
| M14-03 Config Layering | `docs/specs/M14-03-config-layering.spec.md` | `docs/plans/M14-03-config-layering.plan.md` | Deterministic config source precedence and explicit provider/model overrides. |
| M14-04 CLI Config Diagnostics | `docs/specs/M14-04-cli-config-diagnostics.spec.md` | `docs/plans/M14-04-cli-config-diagnostics.plan.md` | CLI config-check flags and UI-safe JSON diagnostics. |
| M14-05 UI Config Snapshot | `docs/specs/M14-05-ui-config-snapshot.spec.md` | `docs/plans/M14-05-ui-config-snapshot.plan.md` | Shared UI/Desktop config snapshot contracts without secret values. |

## Active Work

Current implementation focus: `M14-05-ui-config-snapshot`.

Primary planning documents for the current focus:

- `docs/specs/M14-05-ui-config-snapshot.spec.md`
- `docs/plans/M14-05-ui-config-snapshot.plan.md`
