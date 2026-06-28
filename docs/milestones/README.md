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
| Milestone-level specs | Complete for M01-M25 |
| Milestone-level plans | Complete for M01-M25 |
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
| M14-05 implementation | Complete |
| M14 implementation | Complete |
| M15 technical design | Complete |
| M15 slice specs | Complete for M15-01 through M15-05 |
| M15 slice plans | Complete for M15-01 through M15-05 |
| M15-01 implementation | Complete |
| M15-02 implementation | Complete |
| M15-03 implementation | Complete |
| M15-04 implementation | Complete |
| M15-05 implementation | Complete |
| M15 implementation | Complete |
| M16-M25 technical design | Complete for M16-00 through M25-00 |
| M16 slice specs | Complete for M16-01 through M16-05 |
| M16 slice plans | Complete for M16-01 through M16-05 |
| M16-01 implementation | Complete |
| M16-02 implementation | Complete |
| M16-03 implementation | Complete |
| M16-04 implementation | Complete |
| M16-05 implementation | Complete |
| M16 implementation | Complete |
| M17 implementation slice specs | Complete for M17-01 through M17-05 |
| M17 implementation slice plans | Complete for M17-01 through M17-05 |
| M17-01 implementation | Complete |
| M17-02 implementation | Complete |
| M17-03 implementation | Complete |
| M17-04 implementation | Complete |
| M17-05 implementation | Complete |
| M17 implementation | Complete |
| M18 slice specs | Complete for M18-01 through M18-05 |
| M18 slice plans | Complete for M18-01 through M18-05 |
| M18-01 implementation | Complete |
| M18-02 implementation | Complete |
| M18-03 implementation | Complete |
| M18-04 implementation | Complete |
| M18-05 implementation | Complete |
| M18 implementation | Complete |
| M19 slice specs | Complete for M19-01 through M19-05 |
| M19 slice plans | Complete for M19-01 through M19-05 |
| M19-01 implementation | Complete |
| M19-02 implementation | Complete |
| M19-03 implementation | Complete |
| M19-04 implementation | Complete |
| M19-05 implementation | Complete |
| M19 implementation | Complete |
| M20 slice specs | Complete for M20-01 through M20-07 |
| M20 slice plans | Complete for M20-01 through M20-07 |
| M20-01 implementation | Complete |
| M20-02 implementation | Complete |
| M20-03 implementation | Complete |
| M20-04 implementation | Complete |
| M20-05 implementation | Complete |
| M20-06 implementation | Complete |
| M20-07 implementation | Complete |
| M20 implementation | Complete |
| M21 slice specs | Complete for M21-01 through M21-05 |
| M21 slice plans | Complete for M21-01 through M21-05 |
| M21-01 implementation | Complete |
| M21-02 implementation | Complete |
| M21-03 implementation | Complete |
| M21-04 implementation | Complete |
| M21-05 implementation | Complete |
| M21 implementation | Complete |
| M22 slice specs | Complete for M22-01 through M22-05 |
| M22 slice plans | Complete for M22-01 through M22-05 |
| M22-01 implementation | Complete |
| M22-02 implementation | Complete |
| M22-03 implementation | Complete |
| M22-04 implementation | Complete |
| M22-05 implementation | Complete |
| M22 implementation | Complete |
| M23 slice specs | Complete for M23-01 through M23-05 |
| M23 slice plans | Complete for M23-01 through M23-05 |
| M23-01 implementation | Complete |
| M23-02 implementation | Complete |
| M23-03 implementation | Complete |
| M23-04 implementation | Complete |
| M23-05 implementation | Complete |
| M23 implementation | Complete |
| M24 implementation slice specs | Complete for M24-01 through M24-05 |
| M24 implementation slice plans | Complete for M24-01 through M24-05 |
| M24-01 implementation | Complete |
| M24-02 implementation | Complete |
| M24-03 implementation | Complete |
| M24-04 implementation | Complete |
| M24-05 implementation | Complete |
| M24 implementation | Complete |
| M25 implementation slice specs | Complete for M25-01 through M25-05 |
| M25 implementation slice plans | Complete for M25-01 through M25-05 |
| M25-01 implementation | Complete |
| M25-02 implementation | Complete |
| M25-03 implementation | Complete |
| M25-04 implementation | Complete |
| M25-05 implementation | Complete |
| M25 implementation | Complete |
| M26 technical design | Complete |
| M26 slice specs | Complete for M26-01 through M26-05 |
| M26 slice plans | Complete for M26-01 through M26-05 |
| M26-01 implementation | Complete |
| M26-02 implementation | Complete |
| M26-03 implementation | Complete |
| M26-04 implementation | Complete |
| M26-05 implementation | Complete |
| M26 implementation | Complete |

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
| M15 Config-Driven Turn Execution | `docs/specs/M15-config-driven-turn-execution.spec.md` | `docs/plans/M15-config-driven-turn-execution.plan.md` | Use effective runtime config to resolve providers, execute prompt turns, emit JSON events, and persist replayable turn records. |
| M16 Prompt Runtime and Dynamic Prompt Management | `docs/specs/M16-prompt-runtime-dynamic-management.spec.md` | `docs/plans/M16-prompt-runtime-dynamic-management.plan.md` | Add versioned prompt profiles, dynamic overlays, source diagnostics, and replayable prompt snapshots. |
| M17 Live Model Runtime and Streaming | `docs/specs/M17-live-model-runtime-streaming.spec.md` | `docs/plans/M17-live-model-runtime-streaming.plan.md` | Add real OpenAI-compatible HTTP transport, true model streaming, secret injection, and streamed CLI/JSON output. |
| M18 Agent Tool Loop Runtime | `docs/specs/M18-agent-tool-loop-runtime.spec.md` | `docs/plans/M18-agent-tool-loop-runtime.plan.md` | Build the comprehensive model-tool-observation loop with guards, idempotency, and replayable tool traces. |
| M19 Interactive Approval and Safe Local Actions | `docs/specs/M19-interactive-approval-safe-actions.spec.md` | `docs/plans/M19-interactive-approval-safe-actions.plan.md` | Turn policy asks into durable approval flows for file edits, commands, Git, MCP, plugins, and background work. |
| M20 Terminal UX and TUI REPL | `docs/specs/M20-terminal-ux-tui-repl.spec.md` | `docs/plans/M20-terminal-ux-tui-repl.plan.md` | Make `agent` open a Ratatui-powered terminal REPL while preserving batch, JSON, and plain REPL modes. |
| M21 Desktop GUI MVP | `docs/specs/M21-desktop-gui-mvp.spec.md` | `docs/plans/M21-desktop-gui-mvp.plan.md` | Build a runnable Tauri Desktop MVP over the shared runtime bridge. |
| M22 Extension UX and MCP Gateway | `docs/specs/M22-extension-ux-mcp-gateway.spec.md` | `docs/plans/M22-extension-ux-mcp-gateway.plan.md` | Make plugins, skills, and MCP servers installable, manageable, permissioned, and executable through a gateway. |
| M23 Memory Retrieval Quality | `docs/specs/M23-memory-retrieval-quality.spec.md` | `docs/plans/M23-memory-retrieval-quality.plan.md` | Add embeddings, hybrid retrieval, reranking hooks, retention controls, and auditable memory influence traces. |
| M24 Background Agents UX | `docs/specs/M24-background-agents-ux.spec.md` | `docs/plans/M24-background-agents-ux.plan.md` | Make long-running and scheduled agent tasks manageable across CLI, TUI, and Desktop-facing bridge surfaces. |
| M25 Packaging, Settings, and Updates | `docs/specs/M25-packaging-release-settings-updates.spec.md` | `docs/plans/M25-packaging-release-settings-updates.plan.md` | Prepare repeatable local distribution with packaging, settings, secrets, migrations, release channels, and update flows. |
| M26 Runtime Observability Evidence | `docs/specs/M26-runtime-observability-evidence.spec.md` | `docs/plans/M26-runtime-observability-evidence.plan.md` | Turn traces, telemetry, audit exports, and replay records into redacted evidence bundles for CLI/Desktop debugging and support. |

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

## M15 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M15-00 Config-Driven Turn Execution Technical Design | `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md` | `docs/plans/M15-00-config-driven-turn-execution-technical-design.plan.md` | Architecture baseline for provider resolution, invocation policy, configured turns, CLI prompt execution, JSON output, and persistence. |
| M15-01 Model Runtime Resolver | `docs/specs/M15-01-model-runtime-resolver.spec.md` | `docs/plans/M15-01-model-runtime-resolver.plan.md` | Effective config resolves to a registered executable provider with capability checks. |
| M15-02 Invocation Policy Runner | `docs/specs/M15-02-invocation-policy-runner.spec.md` | `docs/plans/M15-02-invocation-policy-runner.plan.md` | Retry and fallback model invocation behavior around registered providers. |
| M15-03 Configured Turn Runner | `docs/specs/M15-03-configured-turn-runner.spec.md` | `docs/plans/M15-03-configured-turn-runner.plan.md` | Turn execution through effective config, model registry, and `TurnEngine`. |
| M15-04 CLI Prompt Runtime | `docs/specs/M15-04-cli-prompt-runtime.spec.md` | `docs/plans/M15-04-cli-prompt-runtime.plan.md` | Text-mode `--prompt` executes a config-driven runtime turn. |
| M15-05 Persisted Turn Events | `docs/specs/M15-05-persisted-turn-events.spec.md` | `docs/plans/M15-05-persisted-turn-events.plan.md` | Config-driven turn events persist and replay through storage and JSON frames. |

## M16 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M16-00 Prompt Runtime Technical Design | `docs/specs/M16-00-prompt-runtime-technical-design.spec.md` | `docs/plans/M16-00-prompt-runtime-technical-design.plan.md` | Architecture baseline for prompt profiles, overlays, snapshots, diagnostics, replay, CLI, and UI boundaries. |
| M16-01 Prompt Profile Contracts | `docs/specs/M16-01-prompt-profile-contracts.spec.md` | `docs/plans/M16-01-prompt-profile-contracts.plan.md` | Versioned prompt profiles, layer ids, source metadata, visibility, sensitivity, policy, and validation contracts in `agent-context`. |
| M16-02 Prompt Overlays and Diagnostics | `docs/specs/M16-02-prompt-overlays-diagnostics.spec.md` | `docs/plans/M16-02-prompt-overlays-diagnostics.plan.md` | Deterministic overlay application, required-layer enforcement, budget omissions, and content-safe diagnostics. |
| M16-03 Prompt Snapshot Persistence | `docs/specs/M16-03-prompt-snapshot-persistence.spec.md` | `docs/plans/M16-03-prompt-snapshot-persistence.plan.md` | Stable prompt snapshots and persisted runtime events before model invocation. |
| M16-04 CLI Prompt Inspection | `docs/specs/M16-04-cli-prompt-inspection.spec.md` | `docs/plans/M16-04-cli-prompt-inspection.plan.md` | `agent prompt profiles` and `agent prompt inspect` commands with safe text and JSON output. |
| M16-05 UI Prompt Summary | `docs/specs/M16-05-ui-prompt-summary.spec.md` | `docs/plans/M16-05-ui-prompt-summary.plan.md` | Shared UI bridge prompt summary contracts for CLI JSON, TUI, and Desktop. |

## M17 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M17-00 Live Model Runtime Technical Design | `docs/specs/M17-00-live-model-runtime-technical-design.spec.md` | `docs/plans/M17-00-live-model-runtime-technical-design.plan.md` | Architecture baseline for streaming provider traits, reqwest transport, SSE parsing, retry, cancellation, CLI streaming, and fixture tests. |
| M17-01 Model Stream Trait | `docs/specs/M17-01-model-stream-trait.spec.md` | `docs/plans/M17-01-model-stream-trait.plan.md` | Replace collected provider results with true async model event streams while preserving fake-provider compatibility. |
| M17-02 OpenAI Reqwest Transport | `docs/specs/M17-02-openai-reqwest-transport.spec.md` | `docs/plans/M17-02-openai-reqwest-transport.plan.md` | Add reqwest-backed OpenAI-compatible streaming transport with late secret injection and offline tests. |
| M17-03 Live Provider Registration | `docs/specs/M17-03-live-provider-registration.spec.md` | `docs/plans/M17-03-live-provider-registration.plan.md` | Build CLI provider registries from effective config without provider-specific code leaking into core. |
| M17-04 CLI Streaming Output | `docs/specs/M17-04-cli-streaming-output.spec.md` | `docs/plans/M17-04-cli-streaming-output.plan.md` | Render streamed text and NDJSON frames incrementally from configured turns. |
| M17-05 Streaming Retry, Cancellation, and Replay | `docs/specs/M17-05-streaming-retry-cancel-replay.spec.md` | `docs/plans/M17-05-streaming-retry-cancel-replay.plan.md` | Make streamed model execution retryable, cancellable, and replayable through event-sourced traces. |

## M18 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M18-00 Agent Tool Loop Technical Design | `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md` | `docs/plans/M18-00-agent-tool-loop-technical-design.plan.md` | Architecture baseline for comprehensive model-tool-observation loops, tool call accumulation, observations, guards, idempotency, and recovery. |
| M18-01 Comprehensive Turn Runner | `docs/specs/M18-01-comprehensive-turn-runner.spec.md` | `docs/plans/M18-01-comprehensive-turn-runner.plan.md` | Core comprehensive runner contracts that stream one model turn and stop on final response or tool proposal. |
| M18-02 Streaming Tool Call Accumulator | `docs/specs/M18-02-streaming-tool-call-accumulator.spec.md` | `docs/plans/M18-02-streaming-tool-call-accumulator.plan.md` | Normalize model tool-call stream events into executable requests or recoverable observations. |
| M18-03 Tool Observation Context | `docs/specs/M18-03-tool-observation-context.spec.md` | `docs/plans/M18-03-tool-observation-context.plan.md` | Execute allowed tool calls and feed bounded observations back into continuation model turns. |
| M18-04 Loop Guards and Idempotency | `docs/specs/M18-04-loop-guards-idempotency.spec.md` | `docs/plans/M18-04-loop-guards-idempotency.plan.md` | Enforce loop budgets and make completed tool effects replay-safe. |
| M18-05 Local Coding Real Loop E2E | `docs/specs/M18-05-local-coding-real-loop-e2e.spec.md` | `docs/plans/M18-05-local-coding-real-loop-e2e.plan.md` | Prove the comprehensive loop through a deterministic local coding workflow using real built-in tools. |

## M19 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M19-00 Interactive Approval Technical Design | `docs/specs/M19-00-interactive-approval-technical-design.spec.md` | `docs/plans/M19-00-interactive-approval-technical-design.plan.md` | Architecture baseline for durable pending approvals, preview payloads, leases, resume, CLI prompts, and UI bridge contracts. |
| M19-01 Pending Approval Contracts | `docs/specs/M19-01-pending-approval-contracts.spec.md` | `docs/plans/M19-01-pending-approval-contracts.plan.md` | Durable approval records, response contracts, lease scope clamping, resume tokens, and status transitions. |
| M19-02 UI Approval Bridge | `docs/specs/M19-02-ui-approval-bridge.spec.md` | `docs/plans/M19-02-ui-approval-bridge.plan.md` | UI-safe approval request and response schemas shared by CLI, TUI, Desktop, and JSON clients. |
| M19-03 Tool Preview Payloads | `docs/specs/M19-03-tool-preview-payloads.spec.md` | `docs/plans/M19-03-tool-preview-payloads.plan.md` | Secret-safe file, command, Git, MCP, plugin, and background task previews for approval. |
| M19-04 CLI Approval Prompt | `docs/specs/M19-04-cli-approval-prompt.spec.md` | `docs/plans/M19-04-cli-approval-prompt.plan.md` | Plain terminal approval prompts while preserving batch and JSON safety. |
| M19-05 Approval Resume and Recovery | `docs/specs/M19-05-approval-resume-recovery.spec.md` | `docs/plans/M19-05-approval-resume-recovery.plan.md` | Persist pending approvals, suspend turns, resume approved work, and reject denied or expired work. |

## M20 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M20-00 Terminal UX and TUI Technical Design | `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md` | `docs/plans/M20-00-terminal-ux-tui-technical-design.plan.md` | Architecture baseline for the `agent` binary, TTY dispatch, `agent-tui`, Ratatui app state, terminal lifecycle, and bridge-driven panels. |
| M20-01 Agent Binary Entrypoint | `docs/specs/M20-01-agent-binary-entrypoint.spec.md` | `docs/plans/M20-01-agent-binary-entrypoint.plan.md` | Installed `agent` command, mode parsing, TTY dispatch, and script-safe batch/JSON preservation. |
| M20-02 Ratatui Shell | `docs/specs/M20-02-ratatui-shell.spec.md` | `docs/plans/M20-02-ratatui-shell.plan.md` | `agent-tui` crate, terminal lifecycle, pure app state, and base Ratatui layout. |
| M20-03 TUI Composer REPL | `docs/specs/M20-03-tui-composer-repl.spec.md` | `docs/plans/M20-03-tui-composer-repl.plan.md` | Composer state, multiline input, history, slash commands, and submit effects. |
| M20-04 Streaming Transcript | `docs/specs/M20-04-streaming-transcript.spec.md` | `docs/plans/M20-04-streaming-transcript.plan.md` | UI bridge frame reducer and live transcript rendering for model, tool, approval, diagnostic, and final events. |
| M20-05 Approval and Diff Panels | `docs/specs/M20-05-approval-diff-panels.spec.md` | `docs/plans/M20-05-approval-diff-panels.plan.md` | Pending approval and diff side panels with explicit allow/deny bridge commands. |
| M20-06 Trace and Task Panels | `docs/specs/M20-06-trace-task-panels.spec.md` | `docs/plans/M20-06-trace-task-panels.plan.md` | Trace timeline and task notification panels derived from UI-safe bridge events. |
| M20-07 Plain REPL Fallback | `docs/specs/M20-07-plain-repl-fallback.spec.md` | `docs/plans/M20-07-plain-repl-fallback.plan.md` | Line-oriented fallback REPL for constrained terminals and explicit plain mode. |

## M21 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M21-00 Desktop GUI Technical Design | `docs/specs/M21-00-desktop-gui-technical-design.spec.md` | `docs/plans/M21-00-desktop-gui-technical-design.plan.md` | Architecture baseline for Tauri Desktop, Rust commands, frontend layout, bridge sessions, settings, recovery states, and smoke tests. |
| M21-01 Tauri Shell | `docs/specs/M21-01-tauri-shell.spec.md` | `docs/plans/M21-01-tauri-shell.plan.md` | Runnable Tauri shell with safe bootstrap diagnostics and no runtime side effects. |
| M21-02 Desktop Bridge Session | `docs/specs/M21-02-desktop-bridge-session.spec.md` | `docs/plans/M21-02-desktop-bridge-session.plan.md` | Typed Desktop runtime session commands, UI command validation, and bounded UI-safe frame delivery. |
| M21-03 Conversation and Task UI | `docs/specs/M21-03-conversation-task-ui.spec.md` | `docs/plans/M21-03-conversation-task-ui.plan.md` | Conversation, transcript, composer, task list, and task detail views backed by snapshots and bridge frames. |
| M21-04 Approval and Diff UI | `docs/specs/M21-04-approval-diff-ui.spec.md` | `docs/plans/M21-04-approval-diff-ui.plan.md` | Desktop approval queue, approval detail, diff preview, and typed approval resolution command. |
| M21-05 Trace and Settings UI | `docs/specs/M21-05-trace-settings-ui.spec.md` | `docs/plans/M21-05-trace-settings-ui.plan.md` | Trace timeline, secret-safe settings view, validation diagnostics, and Desktop MVP smoke coverage. |

## M22 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M22-00 Extension UX and MCP Gateway Technical Design | `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md` | `docs/plans/M22-00-extension-ux-mcp-gateway-technical-design.plan.md` | Architecture baseline for extension registry, plugin install UX, skill roots, MCP gateway, capability review, health, and isolation. |
| M22-01 Extension Registry | `docs/specs/M22-01-extension-registry.spec.md` | `docs/plans/M22-01-extension-registry.plan.md` | Typed extension registry state for plugin, skill root, and MCP server records. |
| M22-02 Plugin Install Commands | `docs/specs/M22-02-plugin-install-commands.spec.md` | `docs/plans/M22-02-plugin-install-commands.plan.md` | CLI plugin install, list, inspect, enable, disable, remove, and capability review flows. |
| M22-03 Skill Root Management | `docs/specs/M22-03-skill-root-management.spec.md` | `docs/plans/M22-03-skill-root-management.plan.md` | Registry-backed skill root commands, diagnostics, and prompt source traceability. |
| M22-04 MCP Gateway Tools | `docs/specs/M22-04-mcp-gateway-tools.spec.md` | `docs/plans/M22-04-mcp-gateway-tools.plan.md` | MCP tool discovery, local tool registration, policy routing, execution, and audit metadata. |
| M22-05 Extension Health UI | `docs/specs/M22-05-extension-health-ui.spec.md` | `docs/plans/M22-05-extension-health-ui.plan.md` | UI-safe extension snapshots, health diagnostics, capability reviews, and CLI/TUI/Desktop views. |

## M23 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M23-00 Memory Retrieval Technical Design | `docs/specs/M23-00-memory-retrieval-technical-design.spec.md` | `docs/plans/M23-00-memory-retrieval-technical-design.plan.md` | Architecture baseline for embedding providers, hybrid memory retrieval, reranking, write proposals, deletion, rebuilds, and influence traces. |
| M23-01 Embedding Provider Contracts | `docs/specs/M23-01-embedding-provider-contracts.spec.md` | `docs/plans/M23-01-embedding-provider-contracts.plan.md` | Async embedding contracts, vector metadata, validation, redaction, and deterministic fixture provider. |
| M23-02 Memory Index and Hybrid Query | `docs/specs/M23-02-memory-index-hybrid-query.spec.md` | `docs/plans/M23-02-memory-index-hybrid-query.plan.md` | Local memory index, lexical retrieval, optional semantic scoring, hybrid fusion, and deterministic candidate ordering. |
| M23-03 Memory Rerank and Influence Trace | `docs/specs/M23-03-memory-rerank-influence-trace.spec.md` | `docs/plans/M23-03-memory-rerank-influence-trace.plan.md` | Deterministic reranking, selected/rejected reasons, privacy-safe influence traces, and prompt-layer conversion. |
| M23-04 Memory Write Proposals | `docs/specs/M23-04-memory-write-proposals.spec.md` | `docs/plans/M23-04-memory-write-proposals.plan.md` | Reviewed memory write proposals, explicit state transitions, conversion to records, and safe diagnostics. |
| M23-05 Memory Editor and Portability | `docs/specs/M23-05-memory-editor-portability.spec.md` | `docs/plans/M23-05-memory-editor-portability.plan.md` | Tombstones, rebuild/import/export safeguards, memory summaries, and CLI memory commands. |

## M24 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M24-00 Background Agents Technical Design | `docs/specs/M24-00-background-agents-technical-design.spec.md` | `docs/plans/M24-00-background-agents-technical-design.plan.md` | Architecture baseline for background task commands, runners, approvals, notifications, quotas, retries, recovery, artifacts, and reviews. |
| M24-01 Task Command Bridge | `docs/specs/M24-01-task-command-bridge.spec.md` | `docs/plans/M24-01-task-command-bridge.plan.md` | Task command contracts, UI-safe summaries, and CLI/TUI/Desktop command surfaces. |
| M24-02 Background Turn Runner | `docs/specs/M24-02-background-turn-runner.spec.md` | `docs/plans/M24-02-background-turn-runner.plan.md` | Scheduler leases connected to comprehensive turn execution through shared core runtime. |
| M24-03 Task Approvals and Notifications | `docs/specs/M24-03-task-approvals-notifications.spec.md` | `docs/plans/M24-03-task-approvals-notifications.plan.md` | Approval wait state, task notifications, and UI bridge notification frames. |
| M24-04 Task Quotas, Retries, and Recovery | `docs/specs/M24-04-task-quotas-retries-recovery.spec.md` | `docs/plans/M24-04-task-quotas-retries-recovery.plan.md` | Usage quotas, retry visibility, stale lease recovery, and cancellation behavior. |
| M24-05 Task Artifacts and Review | `docs/specs/M24-05-task-artifacts-review.spec.md` | `docs/plans/M24-05-task-artifacts-review.plan.md` | Task artifact metadata, review summaries, and final M24 smoke coverage. |

## M25 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M25-00 Packaging and Release Technical Design | `docs/specs/M25-00-packaging-release-technical-design.spec.md` | `docs/plans/M25-00-packaging-release-technical-design.plan.md` | Architecture baseline for platform paths, settings, keychain secrets, migrations, release artifacts, Desktop bundles, channels, and updates. |
| M25-01 Platform Paths and Settings | `docs/specs/M25-01-platform-paths-settings.spec.md` | `docs/plans/M25-01-platform-paths-settings.plan.md` | Shared platform directories, typed settings, safe mutations, atomic persistence, and UI-safe diagnostics. |
| M25-02 OS Keychain Secret Store | `docs/specs/M25-02-os-keychain-secret-store.spec.md` | `docs/plans/M25-02-os-keychain-secret-store.plan.md` | Secret store backend contracts, OS keychain boundary, environment fallback, and redacted diagnostics. |
| M25-03 Startup Migrations | `docs/specs/M25-03-startup-migrations.spec.md` | `docs/plans/M25-03-startup-migrations.plan.md` | Idempotent startup migrations across config, storage, memory, plugins, and prompts before runtime side effects. |
| M25-04 CLI Release Packaging | `docs/specs/M25-04-cli-release-packaging.spec.md` | `docs/plans/M25-04-cli-release-packaging.plan.md` | Repeatable CLI release artifacts, checksums, manifests, and local smoke tests. |
| M25-05 Desktop Bundle and Updater | `docs/specs/M25-05-desktop-bundle-updater.spec.md` | `docs/plans/M25-05-desktop-bundle-updater.plan.md` | Tauri Desktop bundle flow, update manifest validation, and Desktop release smoke checks. |

## M26 Slice Index

| Slice | Spec | Plan | Outcome |
|---|---|---|---|
| M26-00 Runtime Observability Technical Design | `docs/specs/M26-00-runtime-observability-technical-design.spec.md` | `docs/plans/M26-00-runtime-observability-technical-design.plan.md` | Architecture baseline for evidence bundles, privacy, correlation, replay semantics, CLI/Desktop surfaces, and verification gates. |
| M26-01 Telemetry Export Bundles | `docs/specs/M26-01-telemetry-export-bundles.spec.md` | `docs/plans/M26-01-telemetry-export-bundles.plan.md` | Redacted, bounded, filterable telemetry export bundles. |
| M26-02 Runtime Evidence Builder | `docs/specs/M26-02-runtime-evidence-builder.spec.md` | `docs/plans/M26-02-runtime-evidence-builder.plan.md` | Core evidence bundle from traces, audit exports, telemetry records, and replay summaries. |
| M26-03 CLI Observability Commands | `docs/specs/M26-03-cli-observability-commands.spec.md` | `docs/plans/M26-03-cli-observability-commands.plan.md` | Text and JSON observability commands for local debugging and support bundles. |
| M26-04 UI Observability Snapshots | `docs/specs/M26-04-ui-observability-snapshots.spec.md` | `docs/plans/M26-04-ui-observability-snapshots.plan.md` | Bounded UI-safe observability snapshots for TUI and Desktop. |
| M26-05 Replay Evidence Verification | `docs/specs/M26-05-replay-evidence-verification.spec.md` | `docs/plans/M26-05-replay-evidence-verification.plan.md` | Replay fixture verification proving evidence can be regenerated without side effects. |

## Active Work

Current implementation focus: `M26 complete; next milestone selection`.

Most recent completed implementation documents:

- `docs/specs/M26-05-replay-evidence-verification.spec.md`
- `docs/plans/M26-05-replay-evidence-verification.plan.md`

Most recent completed technical design documents:

- `docs/specs/M16-00-prompt-runtime-technical-design.spec.md` through `docs/specs/M26-00-runtime-observability-technical-design.spec.md`
- `docs/plans/M16-00-prompt-runtime-technical-design.plan.md` through `docs/plans/M26-00-runtime-observability-technical-design.plan.md`

Next implementation slice:

- To be selected after M26 observability evidence review.
