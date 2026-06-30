# Product Roadmap

This roadmap recenters `agent` on the product goal: a Claude Code / Codex Desktop-class local agent with one Rust runtime, strong safety, CLI and Desktop surfaces, and first-class support for multiple model providers.

## Current Product Assessment

The project has built a strong runtime and protocol foundation, but the recent milestone sequence over-invested in MCP details. M30-M54 should be treated as a completed extension track after M54 closeout, not the default direction for the next product work.

The percentages below are directional planning estimates based on the current repository docs, crate layout, tests, and milestone status. They must be validated by running real CLI/Desktop product workflows and end-to-end evaluations.

| Capability Domain | Current Completion | Evidence | Product Gap |
|---|---:|---|---|
| Runtime spine, typed events, replay, checkpoints | 80% | `agent-core`, `agent-types`, `agent-storage`, M01-M04, M26-M29 | Needs product-level recovery UX and larger replay scenarios. |
| Tool, policy, audit, sandbox foundations | 75% | M02, M03, M18, M19, `agent-tool`, `agent-policy`, `agent-sandbox` | Needs polished coding workflow defaults, stronger command streaming, and approval ergonomics. |
| Model provider abstraction | 70% | M13-M17, M57, fake provider, OpenAI-compatible, Anthropic, Gemini, local/Ollama-style providers, typed capabilities, fallback, config, and secrets | Needs broader live-provider smoke evidence, account setup UX, and Desktop provider onboarding. |
| Local coding agent golden path | 60% | M56 repo/session, edit/diff, command/test, final review, and default local-coding eval scenario | Needs daily-driver behavior beyond the deterministic fixture: richer tasks, multi-file edits, retry/recovery UX, and real-provider parity. |
| Context and memory | 50% | M05, M16, M23, prompt profiles, memory stores, retrieval design | Needs repo-aware context, semantic project indexing, memory review UX, and compaction quality gates. |
| CLI and TUI product UX | 50% | M06, M20, `agent-cli`, `agent-tui`, JSON mode | Needs session switching, diff viewer, approval queue, slash command completeness, command cancellation, and stable batch recipes. |
| Desktop product UX | 35% | M10, M21, `agent-desktop`, UI bridge contracts | Needs a real daily-use app: workspace dashboard, chat, diff review, permissions, provider settings, task monitor, logs, and recovery surfaces. |
| Background tasks and autonomy | 45% | M08, M24, scheduler/task contracts | Needs long-running task UX, notifications, quota visibility, resume controls, and failure recovery. |
| Worktree, subagents, teams | 40% | M09, `agent-worktree`, `agent-team` | Needs product workflows for parallel implementation, review, merge, and conflict resolution. |
| Plugins and skills | 55% | M11, M22, `agent-plugin`, `agent-skill` | Needs marketplace-like management, compatibility checks in UI, and real plugin developer loop. |
| MCP extension layer | 85% | M07, M22, M30-M54, `agent-mcp` | Sufficient for now. Remaining OAuth/token polish should be backlog unless unblocking product scenarios. |
| Observability, evals, CI gates | 75% | M26-M29, `agent-eval`, `xtask`, M56 local-coding golden path | Needs broader coding eval corpus and Desktop product smoke evidence. |
| Packaging, settings, updates | 50% | M25, settings/update crates and tests | Needs beta-grade install, upgrade, config migration, secrets setup, and provider onboarding. |

Overall status: the runtime platform is ahead of the product. The next milestones should deliver vertical product value, not more extension protocol depth.

## Roadmap Principles

- Build vertical slices that a developer can actually use in a repository.
- Keep CLI, TUI, and Desktop as clients of the same runtime events and commands.
- Treat model provider diversity as a core product requirement, not an adapter afterthought.
- Use MCP, plugins, and skills as extension surfaces after the core coding loop is excellent.
- Every milestone needs a concrete acceptance demo, product E2E test, and trace/audit evidence.
- Prefer fewer, larger product milestones with clear slices over long runs of narrow protocol milestones.

## MCP Track Decision

MCP work is paused as a primary roadmap track after M54 completion. Future MCP slices are product backlog unless a near-term CLI/Desktop workflow requires them.

MX01 is an MCP epilogue track for P1-P3 protocol hardening and bridge-safe contracts; the product roadmap remains M55+ and P0 OAuth/Auth product closure remains outside MX01.

The current MCP baseline is enough to support product discovery, local gateway execution, stdio/HTTP clients, server exports, sampling foundations, stream supervision, OAuth planning, and authorization-code token exchange into token leases. Further MCP OAuth lifecycle polish is valuable, but it does not improve the main product experience as much as coding-loop, provider, CLI, and Desktop work.

## Product Milestone Roadmap

| Milestone | Name | Goal | Key Acceptance |
|---|---|---|---|
| M55 | Product Baseline and Roadmap Reset | Convert the current repo into a measurable product baseline and freeze MCP expansion. | Capability audit report, golden-path smoke matrix, milestone index updated, and next specs/plans created from product priorities. |
| M56 | Coding Agent Golden Path v1 | Make `agent` useful as a local coding assistant in one repository session. | Deterministic CLI/JSON eval proves repo/session, edit/diff, command/test, final review, trace, audit, and replay evidence. |
| M57 | Multi-Provider Runtime | Support real provider choice across OpenAI, Anthropic, Gemini, OpenAI-compatible, and local/Ollama-style endpoints. | Provider capability matrix, account diagnostics, model selection, streaming, tool-call compatibility, fallback, and redacted error reporting. |
| M58 | Project Context Intelligence | Build repo-aware context assembly for coding tasks. | File tree summaries, relevant-file retrieval, semantic/lexical search, pinned context, budget planning, and context quality traces. |
| M59 | CLI/TUI Daily Driver | Turn terminal usage into a polished developer workflow. | Slash commands, session restore, approval queue, diff viewer, command streaming/cancel, task status, and JSON/batch stability. |
| M60 | Desktop Daily Driver | Deliver a usable Desktop app over the shared runtime. | Workspace dashboard, conversations, composer, streaming transcript, approvals, diff review, provider settings, logs, and task monitor. |
| M61 | Plan, Todo, and Task UX | Make plans and tasks first-class across interactive and background work. | TodoWrite-compatible task state, plan updates, task artifacts, background approval waits, pause/resume/cancel, and notifications. |
| M62 | Reliability and Recovery Productization | Make real work recoverable after interruption, crashes, and conflicts. | Checkpoint resume UX, replay/fork commands, idempotent tool recovery, conflict diagnostics, failed-command retry, and crash smoke tests. |
| M63 | Worktree and Agent Team Workflows | Productize parallel implementation and review. | Isolated worktrees, subagent handoff, shared blackboard, reviewer role, merge review, conflict handling, and team-level permission ceilings. |
| M64 | Memory and Personalization | Make memory useful, reviewable, and privacy-safe. | Project memory, user preferences, semantic recall, memory proposals, retention controls, import/export, and influence traces in CLI/Desktop. |
| M65 | Packaging, Release, and Beta Gate | Prepare a beta users can install and trust. | CLI packages, Desktop bundles, config migrations, keychain setup, update channel, release smoke suite, and support bundle generation. |
| M66 | Extensions and MCP Hardening | Return to plugin/MCP polish after product beta paths are credible. | Finish only extension work that improves installable user workflows: MCP OAuth completion, plugin management, compatibility, and extension health. |

## Near-Term Implementation Focus

### M55 Product Baseline and Roadmap Reset

Status: Complete. The baseline, smoke matrix, product backlog, and M56 bootstrap docs now define the mainline product direction.

Purpose: stop roadmap drift and produce a clear baseline before new feature work.

Required slices:

| Slice | Outcome |
|---|---|
| M55-01 Capability Inventory | Map crates, tests, commands, and docs into product capability domains. |
| M55-02 Golden Path Smoke Matrix | Define current CLI, TUI, Desktop, provider, coding-loop, and recovery smoke checks. |
| M55-03 Product Gap Backlog | Convert gaps into prioritized product backlog items with owners by crate/surface. |
| M55-04 M56 Bootstrap | Update milestone docs, reconcile roadmap status, and create M56 milestone spec/plan plus the first implementation slice spec/plan. |

Acceptance:

- The milestone index no longer points to MCP as the next implementation track.
- MCP is explicitly marked as parked after M54 unless product scenarios require it.
- M56 has implementation-ready spec and plan files.
- Product acceptance tests are defined before code changes resume.

### M56 Coding Agent Golden Path v1

Status: Complete. `cargo xtask eval` now runs the default suite with the deterministic `default-m56-golden-path` local-coding scenario.

Purpose: prove the project can behave like a real coding agent, not only a runtime library.

Required slices:

| Slice | Outcome |
|---|---|
| M56-01 Repo Session Bootstrap | Open a workspace, load config, detect Git state, and start a durable conversation. |
| M56-02 Edit and Diff Flow | Strengthen file edit previews, apply flow, diff summaries, and approval payloads. |
| M56-03 Command/Test Loop | Stream shell output, cancel commands, classify failures, and feed bounded observations back to the model. |
| M56-04 Final Review Summary | Produce changed-file summary, test evidence, unresolved risks, and next actions. |
| M56-05 Golden Path E2E | Run deterministic coding-agent scenarios through CLI/JSON with trace and audit evidence. |
| M56-06 Hardening Docs and Verification | Document the acceptance command, run final gates, and hand off to M57. |

Acceptance:

- A developer can run one command to verify the deterministic local coding path and inspect redacted eval artifacts.
- The fixture-driven turn is event-sourced, permissioned, auditable, replayable, and summarized.
- Provider capability diagnostics and fallback behavior are implemented in M57; broader live-provider smoke coverage remains future product hardening.

### M57 Multi-Provider Runtime

Status: Complete. Final closeout gates passed with provider diagnostics redacted, including `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo xtask eval`.

Purpose: make provider choice a product feature.

Required slices:

| Slice | Outcome |
|---|---|
| M57-01 Provider Capability Matrix | Model streaming, tool calling, structured output, context limits, image support, and cost metadata. |
| M57-02 Anthropic Adapter | Native Claude adapter with streaming and tool-use mapping. |
| M57-03 Gemini Adapter | Native Gemini adapter with streaming and tool-use mapping. |
| M57-04 Local Provider Adapter | Ollama/OpenAI-compatible local endpoint profile with diagnostics. |
| M57-05 Provider UX and Fallback | CLI/Desktop account diagnostics, model selection, fallback policy, and redacted support output. |

Acceptance:

- Users can configure and switch providers without runtime changes.
- Provider capability mismatches are diagnosed before a turn fails.
- Tool-calling models can run the same coding golden path where supported.

### M58 Project Context Intelligence

Status: Complete. M58 now includes bounded project summaries, lexical retrieval, optional semantic/hybrid retrieval, pinned context budget planning, context quality traces, and deterministic default-suite eval coverage.

Purpose: build repo-aware context assembly for coding tasks.

Initial slices:

| Slice | Outcome |
|---|---|
| M58-01 Repo Map and File Tree Summaries | Build typed project structure summaries that are bounded, redacted, and usable by CLI/Desktop. |
| M58-02 Relevant File Retrieval | Select likely relevant files for a task using lexical signals and workspace metadata. |
| M58-03 Semantic and Hybrid Search | Add embedding-backed retrieval where configured, with deterministic fixture coverage. |
| M58-04 Pinned Context and Budget Planning | Let users pin context and fit selected material into prompt budgets with diagnostics. |
| M58-05 Context Quality Traces and Eval | Add context assembly traces and product eval coverage for coding tasks. |

Acceptance:

- File tree summaries and relevant-file retrieval improve coding prompts without exposing raw secrets.
- Pinned context and budget planning are typed and visible through CLI/JSON and bridge contracts.
- Context quality is covered by deterministic tests and at least one eval scenario.

### M59 CLI/TUI Daily Driver

Status: Planned. Milestone-level spec/plan and the first session restore/switch slice spec/plan are ready; implementation starts with M59-01.

Purpose: make terminal use feel like a coherent daily coding workflow instead of separate command demos.

Initial slices:

| Slice | Outcome |
|---|---|
| M59-01 Session Restore and Switch | Add a typed terminal session directory, CLI/JSON list/resume commands, bridge switch command, and TUI session panel state. |
| M59-02 Slash Command Catalog and Status Hub | Make common terminal actions discoverable and consistent through typed slash command descriptors and status output. |
| M59-03 Approval Queue and Permission Review | Improve approval review with safe previews, focus movement, lease display, and batch-safe decisions. |
| M59-04 Diff Viewer and Command Streaming Controls | Improve diff navigation, command/test output state, and cancellation readiness in terminal panels. |
| M59-05 Task Status and JSON/Batch Recipes | Polish task status visibility and document/test stable JSON and batch workflows. |

Acceptance:

- A developer can restore or switch recent sessions from CLI/JSON and TUI.
- Slash commands, approvals, diffs, command/test status, and tasks are visible and actionable in terminal workflows.
- JSON and batch recipes are documented, deterministic, bounded, and redacted.

## Capability Gates

Each future milestone should ship with:

- A milestone spec in `docs/specs/` and implementation plan in `docs/plans/`.
- At least one user-visible workflow acceptance test or smoke script.
- Runtime trace, permission trace, and audit evidence where side effects occur.
- CLI/JSON coverage for every new runtime command.
- Desktop bridge contracts when the behavior should appear in Desktop.
- Documentation updates that describe product usage, not only internal contracts.

## Success Definition

The roadmap is on track when `agent` can:

- Run as a daily coding assistant from CLI/TUI.
- Run the same workflow from Desktop with visual approvals and diffs.
- Use multiple model providers through a shared capability model.
- Recover interrupted work without losing traceability.
- Explain what it did, what it changed, what it ran, and why permissions were required.
- Extend through MCP/plugins/skills without making extension work dominate the product roadmap.
