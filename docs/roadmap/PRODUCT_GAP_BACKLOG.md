# Product Gap Backlog

This backlog converts the M55 capability baseline and smoke matrix into product-shaped work for M56-M66. Items are written as user-visible outcomes with acceptance signals so future implementation can pull scoped work without redoing the roadmap reset.

## Priority Legend

| Priority | Meaning |
|---|---|
| P0 | Blocks a credible local coding-agent daily-driver path. |
| P1 | Materially improves provider choice, context quality, CLI/Desktop UX, recovery, or release readiness. |
| P2 | Improves polish, extension depth, or non-blocking hardening. |

## Backlog Items

| ID | Priority | Domain | User-Visible Outcome | Current Evidence | Gap | Target Milestone | Acceptance Signal | Dependencies | Status |
|---|---|---|---|---|---|---|---|---|---|
| M56-GP-01 | P0 | Local coding loop | User starts `agent` in a repo and gets a durable repository session with Git/config/provider diagnostics. | `ruse-core/tests/comprehensive_local_coding_e2e.rs`, `ruse-cli/tests/local_coding.rs`, GP-01 | Repo session bootstrap is fixture-level and not yet a first-class product entrypoint. | M56 | CLI/JSON command creates or resumes a durable conversation and emits repo/session diagnostics without secrets. | M55 baseline and smoke matrix | Planned |
| M56-GP-02 | P0 | Edit and diff flow | User reviews file edit previews, applies changes, and sees changed-file/diff evidence. | `ruse-tool/tests/file_edit_tool.rs`, `ruse-tool/tests/git_tools.rs`, GP-04, GP-05 | Edit/diff pieces exist but are not integrated into one daily coding workflow. | M56 | Golden path applies at least one file edit, shows diff summary, and records approval/audit evidence. | M56-GP-01 | Planned |
| M56-GP-03 | P0 | Command/test loop | User sees command execution, bounded output, failure classification, and model observation feedback. | `ruse-tool/tests/shell_command_tool.rs`, `ruse-core/tests/comprehensive_local_coding_e2e.rs`, GP-06, GP-07 | Command streaming/cancel exists in pieces; product loop must make test feedback visible and recoverable. | M56 | Golden path runs a verification command, captures status/output, and includes evidence in trace and final summary. | M56-GP-01 | Planned |
| M56-GP-04 | P0 | Final review summary | User receives a final summary with changed files, tests run, unresolved risks, and next actions. | `ruse-cli/tests/local_coding.rs`, trace/audit tests, GP-03, GP-09 | Current summaries are fixture-level and not a standard final review contract. | M56 | CLI/JSON final output includes changed files, commands, audit/trace ids, and remaining risks. | M56-GP-02, M56-GP-03 | Planned |
| M56-GP-05 | P0 | Product E2E and replay | One deterministic coding-agent scenario proves prompt, tools, permissions, diff, test, final summary, trace, and replay. | `docs/eval/default.suite.json`, `ruse-core/tests/evidence_replay_verification.rs`, GP-11 | Eval corpus is replay-focused, not yet a full coding-agent product scenario. | M56 | New or updated eval/smoke scenario passes with redacted evidence under `target/`. | M56-GP-01 through M56-GP-04 | Planned |
| M57-PROV-01 | P0 | Provider runtime | User sees a provider capability matrix before a turn starts. | `ruse-model/tests/provider_registry.rs`, GP-17 | Capability mismatch diagnostics are incomplete for product routing. | M57 | CLI/JSON reports streaming/tool/context capability mismatches before model invocation. | M56 golden path | Planned |
| M57-PROV-02 | P0 | Provider adapters | User can configure Anthropic, Gemini, OpenAI-compatible, and local/Ollama-style providers. | OpenAI-compatible tests, fake provider tests, GP-18 | Native Anthropic/Gemini/local profiles are missing. | M57 | Provider-specific fixture tests pass and unsupported capabilities fail safely. | M57-PROV-01 | Planned |
| M57-PROV-03 | P1 | Provider diagnostics | User gets redacted account/config diagnostics and fallback behavior. | `ruse-cli/tests/live_provider_registration.rs`, secret resolver tests | Provider errors and fallback are not yet a polished product surface. | M57 | CLI/JSON support output redacts secrets and explains selected provider/model/fallback. | M57-PROV-01 | Planned |
| M58-CTX-01 | P0 | Project context | Coding tasks select relevant files from a repo without manual file lists. | Context budget/pinning tests, GP-20 | Repo-aware relevant-file retrieval is missing. | M58 | Golden coding scenario finds target files through lexical/semantic project context and emits context trace. | M56 golden path | Planned |
| M58-CTX-02 | P1 | Context quality | User can inspect what context was included, omitted, pinned, or compacted. | `ruse-context/tests/*`, prompt snapshot tests | Context quality trace is not productized. | M58 | CLI/JSON trace shows selected files, memory items, budget decisions, and pinned constraints. | M58-CTX-01 | Planned |
| M59-CLI-01 | P0 | CLI/TUI daily driver | User can run the M56 golden path from terminal without hidden fixture commands. | CLI/TUI tests, GP-12 through GP-14 | Terminal flows are broad but not one cohesive daily workflow. | M59 | Documented CLI/TUI recipe completes golden path with approvals, diffs, command output, and final summary. | M56 | Planned |
| M59-CLI-02 | P1 | Terminal ergonomics | User can restore/switch sessions, inspect approvals, view diffs, and cancel commands. | `ruse-tui/tests/*`, CLI approval/trace/task tests | Session and approval queue ergonomics are incomplete. | M59 | TTY/JSON/TUI modes expose equivalent state for session, approval, diff, command, and task surfaces. | M59-CLI-01 | Planned |
| M60-DESK-01 | P0 | Desktop daily driver | User can perform the M56-equivalent coding path in Desktop with visual approvals and diffs. | Desktop snapshot tests, GP-15, GP-16 | Real Desktop golden path is blocked behind app-shell integration. | M60 | Desktop bridge simulation or app test completes workspace, conversation, approval, diff, command, trace, and summary flow. | M56, M59 bridge contracts | Planned |
| M60-DESK-02 | P1 | Desktop settings and logs | User can configure providers, inspect logs, and review task/trace state without duplicating runtime logic. | `ruse-desktop/tests/settings.rs`, `trace_timeline.rs`, `conversation_tasks.rs` | Settings/log/task UX is contract-level, not daily-use product UX. | M60 | Desktop surfaces consume shared runtime snapshots/events and pass bridge-safe tests. | M57, M60-DESK-01 | Planned |
| M61-TASK-01 | P1 | Plan/todo/task UX | User sees plans, todos, task state, artifacts, and approval waits across interactive/background work. | Scheduler/task tests, CLI task commands, GP-21 | TodoWrite-like planning and task UX are not first-class product surfaces. | M61 | CLI/TUI/Desktop bridge exposes task lifecycle, plan updates, artifacts, pause/resume/cancel, and notifications. | M56, M59, M60 | Planned |
| M62-REC-01 | P1 | Recovery UX | User can resume, replay, fork, or retry interrupted work with clear side-effect safety. | Recovery, replay, idempotency tests, GP-10, GP-11 | Recovery is strong internally but not productized for daily use. | M62 | Crash/restart smoke proves checkpoint resume and idempotent completed-effect handling. | M56 | Planned |
| M63-TEAM-01 | P1 | Worktree/team workflows | User can delegate parallel implementation/review to isolated worktrees and merge safely. | Worktree/team tests, GP-22 | Product flow for parallel work, review, and merge is missing. | M63 | Workflow creates isolated worktree, runs reduced-capability subagent, records blackboard output, and reports conflicts. | M56, M62 | Planned |
| M64-MEM-01 | P1 | Memory review and personalization | User can review memory proposals, accept/reject preferences, and see memory influence. | Memory tests, CLI memory commands, GP-19 | Memory exists but needs transparent product controls and privacy UX. | M64 | CLI/Desktop memory review shows source, sensitivity, retention, influence trace, import/export, and deletion. | M58 | Planned |
| M65-REL-01 | P1 | Beta install and release gate | User can install CLI/Desktop and run release smoke with support artifacts. | `xtask` package tests, GP-23 | Packaging exists, but beta gate lacks product golden-path evidence. | M65 | Release gate packages CLI/Desktop, runs M56 smoke/eval, validates migrations, and writes redacted support bundle. | M56-M60 | Planned |
| M66-EXT-01 | P2 | Extension management | User can manage plugins, skills, and MCP servers with health, compatibility, and safe diagnostics. | Plugin/skill/MCP tests, extension health docs, support bundle evidence, GP-24 | Closed by M66 extension hardening. | M66 | CLI/Desktop extension management shows health, permissions, compatibility, and redacted diagnostics. | M59, M60 | Complete |
| MX01-DEFER-01 | P2 | MCP epilogue | MCP P1-P3 hardening remains separate unless needed by product workflows. | M30-M54 MCP depth; MX01 drafts may exist outside mainline | MCP epilogue should not pull M55-M65 off product value. | M66 or separate MX01 | Revive only when linked to a user-visible CLI/Desktop workflow or extension health gap. | Product scenario evidence | Deferred |

## Cross-Milestone Dependencies

| Dependency | Affects | Why It Matters |
|---|---|---|
| M56 golden path before broad UX polish | M59, M60, M65 | CLI/TUI/Desktop/release gates need one stable workflow to exercise. |
| Provider capability diagnostics before native adapters are productized | M57, M56 real-provider smoke | Users need safe failure before turns consume tokens or expose confusing provider errors. |
| Repo context before memory personalization | M58, M64 | Project relevance should be reliable before long-term memory influences coding behavior. |
| Recovery UX before team workflows | M62, M63 | Parallel agents amplify side effects; resume/replay/idempotency must be product-visible first. |
| Core product credibility before extension hardening | M66, MX01 | Extensions should support user workflows rather than dominate roadmap sequencing. |

## M56 Immediate Start List

Start M56 with these implementation-ready outcomes:

- create `M56-01 Repo Session Bootstrap` to open a workspace, load config, detect Git state, and create a durable conversation
- follow with edit/diff, command/test, final review, and deterministic E2E slices
- wire fake-provider evidence first; document real-provider opt-in path without requiring secrets in default CI
- carry trace, audit, permission, replay, and recovery evidence through the whole workflow

## Deferred MCP Notes

MCP remains a strong extension layer but is not the mainline bottleneck. P1-P3 MCP epilogue work should stay separate unless one of these conditions is true:

- M56-M60 user workflows require an MCP behavior to complete the coding-agent path
- M65 beta release gates expose an extension health or safety issue that blocks installable workflows
- M66 explicitly resumes extension/MCP management after CLI/Desktop/product provider paths are credible
