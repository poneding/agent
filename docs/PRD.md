# agent PRD

## 1. Product Overview

agent is a local-first, extensible runtime for building secure, auditable, high-performance AI agents. It provides the core execution engine for CLI, terminal TUI, Desktop, background automations, coding agents, multi-agent teams, MCP integrations, plugins, tools, memory, and traceable long-running workflows.

The product is a platform layer rather than a single agent application. Its core value is a durable runtime spine that turns model output into controlled, observable, recoverable actions.

## 2. Target Users

| User | Need |
|---|---|
| Individual developers | Run coding agents safely inside local repositories through CLI, terminal TUI, or Desktop. |
| Agent product builders | Build custom agents without reimplementing model, tool, memory, policy, and recovery infrastructure. |
| Platform teams | Standardize permissions, audit, plugins, MCP, tasks, and traceability across agent workflows. |
| Power users | Inspect, replay, debug, schedule, and extend agent behavior. |

## 3. Product Goals

- Provide one shared Rust runtime for CLI, terminal TUI, and Desktop.
- Support complete agent turns with model streaming, tool use, permission gating, recovery, memory, and persistence.
- Make every meaningful runtime action observable, auditable, and replayable.
- Enable provider-agnostic model integration and tool-agnostic execution.
- Support safe long-running agents, background tasks, cron jobs, and subagent teams.
- Enable a controlled extension ecosystem through plugins, skills, and MCP.

## 4. Core User Journeys

### 4.1 Interactive Coding Turn

1. User asks the agent to inspect or modify a repository.
2. Runtime assembles context from prompt layers, memory, project files, and current task state.
3. Model streams reasoning-safe responses and proposes tool calls.
4. Permission engine evaluates file, command, network, Git, MCP, or browser actions.
5. Approved tools execute in sandboxed environments.
6. Results return to the model until the turn completes.
7. Runtime persists events, artifacts, memory updates, context summaries, and checkpoints.

### 4.2 Background Task

1. User creates a task or schedule.
2. Scheduler leases the task with quotas for time, tools, model cost, and concurrency.
3. Agent runs with a durable conversation and checkpoint chain.
4. Required approvals pause the task and notify the user.
5. Completion produces a trace, summary, artifacts, and optional Git output.

### 4.3 Team Workflow

1. Lead agent decomposes work into subtasks.
2. Worker agents run with reduced capabilities in isolated worktrees or contexts.
3. Reviewer agent inspects results through shared blackboard entries, diffs, and traces.
4. Lead agent merges accepted outputs into the primary task.

### 4.4 Debug and Replay

1. User opens a turn trace.
2. Runtime shows model events, tool calls, permission decisions, tool outputs, compacted context, and memory writes.
3. User replays from a checkpoint, forks a trace, or dry-runs a tool under simulated policy.

## 5. Functional Requirements

### 5.1 Agent Core Runtime

- Manage Workspace, Session, Conversation, Turn, Task, Agent, and Team entities.
- Execute turn lifecycle from input acceptance through final persistence.
- Emit typed runtime events for model, tool, policy, memory, context, scheduler, UI, and error activity.
- Support cancellation, pause, resume, checkpoint, replay, and fork.

### 5.2 Model Adapter

- Provide a unified interface for streaming model responses, tool calls, structured outputs, token accounting, and provider capabilities.
- Support provider-specific adapters while keeping provider details inside the adapter boundary.
- Support fallback, retry, timeout, and model capability negotiation.

### 5.3 Tool System

- Register built-in, plugin, MCP, and user-defined tools.
- Validate tool input and output with JSON Schema or strongly typed Rust schemas.
- Support idempotency keys, timeouts, cancellation, artifacts, stdout/stderr capture, and result compression.
- Route every tool call through policy evaluation and sandbox execution.

### 5.4 Permission and Policy Engine

- Evaluate capabilities for files, commands, network, Git, MCP, browser, Desktop APIs, secrets, background tasks, and subagents.
- Return allow, deny, ask, or defer decisions.
- Issue scoped permission leases.
- Persist permission traces and audit records.
- Support policy inheritance and capability reduction for subagents.

### 5.5 Sandbox and Isolation

- Support workspace read/write boundaries.
- Support process sandboxing with environment, filesystem, network, and timeout controls.
- Support Git worktree isolation, temporary directory isolation, file locks, conflict detection, and cleanup.
- Expose platform capability metadata for macOS, Linux, and Windows.

### 5.6 Memory Engine

- Support working memory, conversation memory, project memory, user preference memory, semantic memory, experience memory, and team blackboard memory.
- Store memory with source, scope, confidence, sensitivity, retention policy, and revision metadata.
- Support hybrid retrieval, reranking, pinning, redaction, export, import, and deletion.

### 5.7 Context Manager

- Assemble prompts from system, product, project, user, skill, tool, memory, and runtime layers.
- Enforce token budgets and priority ordering.
- Support rolling summary, hierarchical summary, semantic compression, tool result compression, and message pinning.
- Validate compaction quality against pinned constraints and recovery requirements.

### 5.8 Scheduler and Background Tasks

- Support one-shot tasks, recurring schedules, paused tasks, resumable tasks, and long-running agents.
- Track task state, leases, retries, quotas, cancellation, notifications, and artifacts.
- Support concurrent execution limits by workspace, user, model provider, tool class, and system resources.

### 5.9 Subagents and Teams

- Define agent roles, capabilities, prompts, models, memory scopes, and tool scopes.
- Support handoff envelopes with task goal, context, constraints, permissions, artifacts, and expected output.
- Support shared blackboard, review, voting, arbitration, and lead-agent consolidation.

### 5.10 Skills and Plugin Runtime

- Discover skills from configured roots and plugin packages.
- Load skill metadata, instructions, assets, prompts, hooks, and tool extensions.
- Support plugin manifests with name, version, capabilities, permissions, compatibility, entrypoint, and isolation mode.
- Support process plugins first, WASM plugins for constrained extensions, and dynamic libraries for trusted built-ins.

### 5.11 MCP Layer

- Support MCP client connections to external servers.
- Expose selected local tools/resources through an MCP server mode.
- Provide an MCP gateway that normalizes server tools into local tool specs and local policy decisions.
- Support MCP version negotiation, server health checks, and per-server permissions.

### 5.12 REPL and Debug Console

- Support slash commands, multiline input, history, completion, status inspection, tool dry-run, permission simulation, breakpoints, checkpoint replay, trace replay, and plugin debugging.
- Provide the same debugging capabilities in CLI and Desktop.

### 5.13 CLI and Terminal TUI

- Support batch mode, JSON event stream mode, plain REPL mode, and full-screen TUI REPL mode.
- Make `agent` with no arguments enter the TUI REPL when stdin and stdout are interactive terminals.
- Support slash commands for model, tools, permissions, memory, compact, tasks, agents, MCP, plugins, trace, replay, and status.
- Display streaming output, diffs, approvals, task status, errors, and verification results in terminal-appropriate forms.

### 5.14 Desktop

- Provide visual conversations, task boards, background automations, permission dialogs, diff review, trace timeline, memory editor, settings, MCP management, and plugin management.
- Consume the same runtime events and commands as CLI through the UI bridge.

## 6. Data Model

| Entity | Description |
|---|---|
| Workspace | Trust boundary, repository root, policy scope, sandbox profile, memory namespace. |
| Session | Active UI/runtime connection with actor identity and presentation preferences. |
| Conversation | Durable thread containing turns, summaries, checkpoints, memory links, and artifacts. |
| Turn | One complete agent loop triggered by user input, automation, or subagent handoff. |
| Task | Durable unit of work with schedule, status, quotas, retries, artifacts, and conversation link. |
| Agent | Role, model, prompt profile, tools, memory scope, and capability scope. |
| Team | Coordinated set of agents with protocol, shared blackboard, and team-level capability ceiling. |
| Tool | Named capability with schema, executor, permission needs, sandbox profile, and result contract. |
| MemoryRecord | Scoped knowledge item with source, confidence, sensitivity, retention, embedding, and revision. |
| RuntimeEvent | Append-only record of state transitions and observations. |
| Checkpoint | Recoverable snapshot of turn/task state with artifact references and idempotency markers. |

## 7. State Machines

### 7.1 Turn State

`Accepted -> Prepared -> ContextBuilt -> ModelStreaming -> ToolProposed -> PermissionEvaluating -> ToolExecuting -> Observed -> Finalizing -> Persisted -> Done`

Waiting and terminal states:

`WaitingApproval`, `WaitingIO`, `Cancelled`, `Failed`, `Recovered`

### 7.2 Task State

`Created -> Queued -> Running -> WaitingApproval -> Running -> Succeeded`

Additional states:

`Paused`, `Sleeping`, `Failed`, `Cancelled`, `Expired`

## 8. Non-Functional Requirements

| Category | Requirement |
|---|---|
| Performance | Stream model output, execute independent work concurrently, keep local tool dispatch low-latency, and bound compaction overhead. |
| Reliability | Persist event logs and checkpoints, support crash recovery, use idempotency keys, and reconcile external side effects. |
| Security | Enforce least privilege, sandbox execution, scoped secrets, explicit permission leases, and prompt-injection-aware boundaries. |
| Observability | Emit structured traces, metrics, audit logs, tool traces, permission traces, and replayable turn records. |
| Testability | Support unit, integration, end-to-end, property, fuzz, concurrency, snapshot, replay, and policy regression tests. |
| Extensibility | Keep provider, tool, UI, storage, MCP, plugin, memory, and scheduler implementations replaceable. |
| Portability | Support macOS, Linux, and Windows with explicit platform capability reporting. |
| Migration | Version schemas for events, checkpoints, memory, config, plugin manifests, and policies. |

## 9. MVP Scope

### Included

- Rust workspace and crate skeleton.
- Event model and durable event store.
- Single-agent turn engine.
- Fake model provider for deterministic testing.
- At least one real model provider adapter.
- Built-in file, search, shell, and Git tools.
- Basic permission engine and approval flow.
- Local process sandbox profile.
- CLI TTY mode.
- Basic context assembly and compaction.
- Conversation persistence and checkpoint recovery.
- Structured tracing and audit logs.

### Acceptance Criteria

- A user can run an interactive CLI session in a local repository.
- The agent can inspect files, propose edits, run safe commands, and report results.
- Tool calls are schema-validated, permission-checked, sandboxed, traced, and persisted.
- A crashed or interrupted turn can resume from a checkpoint.
- A deterministic fake-provider test can replay a complete turn from stored events.
- `cargo fmt`, `cargo clippy`, and `cargo test` pass across the workspace.

## 10. Beta Scope

- MCP client.
- Skills discovery and loading.
- Hooks and lifecycle events.
- Project memory and user preference memory.
- Worktree isolation.
- JSON event stream CLI mode.
- Durable background task scheduler.
- Trace replay and REPL debug commands.
- Plugin manifest parser and process plugin prototype.

## 11. Production Scope

- Desktop application using the shared runtime bridge.
- Background automations and cron scheduler.
- Subagent and team workflows.
- WASM plugin isolation.
- MCP server and gateway.
- Audit export and telemetry dashboards.
- Schema and config migration tooling.
- Memory import/export and retention controls.

## 12. Success Metrics

| Metric | Target Direction |
|---|---|
| Turn recoverability | Complete turns can replay from event logs and resume from checkpoints. |
| Tool safety | Every side-effecting tool has policy, sandbox, trace, and audit coverage. |
| Latency | CLI streams first model output quickly and tool dispatch remains responsive. |
| Extension quality | New providers, tools, plugins, and MCP servers integrate without core rewrites. |
| Debuggability | Users can inspect why a model acted, which context it saw, which tools ran, and which permissions applied. |
| Cross-surface consistency | CLI and Desktop show the same runtime truth through shared events. |

## 13. Release Readiness

| Release | Readiness Signal |
|---|---|
| MVP | Local coding loop is usable, testable, persisted, and recoverable. |
| Beta | External tools, memory, tasks, hooks, and trace debugging are stable enough for daily development. |
| Production | Desktop, automations, teams, plugins, telemetry, migrations, and audit workflows are production-grade. |
| Extensibility | Third-party providers, MCP servers, plugins, and skills can be added through documented contracts. |
