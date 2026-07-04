# Golden Path Smoke Matrix

M55 defines the smoke coverage needed to prove that `agent` is becoming a daily coding-agent product rather than only a collection of runtime crates. This matrix is conservative: `Pass` means an existing command or test already covers the named scope; `Documented` means a command path exists but M55 did not execute it; `Blocked` means a dependency must be implemented first; `Missing` means no current command or contract proves the workflow.

## Status Legend

| Status | Meaning |
|---|---|
| Pass | Existing tests or gates cover the stated workflow scope. |
| Documented | A command path exists, but this M55 slice did not run it. |
| Blocked | The smoke requires a missing product capability or configuration. |
| Missing | No current smoke path or contract exists. |

## Smoke Matrix

| ID | Workflow | Surface | Command or Verification Path | Fixtures / Configuration | Expected Evidence | Current Status | Gap Owner |
|---|---|---|---|---|---|---|---|
| GP-01 | Repository session bootstrap | Core/CLI | `cargo test -p agent-core comprehensive_local_coding_e2e` | Temp Git repo fixture, fake provider | Workspace config, Git state, durable ids, tool registry, and final replay evidence | Pass | M56-01 |
| GP-02 | Fake-provider prompt streaming | CLI JSON | `cargo xtask cli-json -- "hello agent"` or `cargo test -p agent-cli json_mode_prompt_runtime_emits_delta_frames_before_done` | Built-in fake provider | NDJSON model selection, delta, final response, done frame | Pass | M56 |
| GP-03 | Local coding summary smoke | CLI text | `cargo test -p agent-cli local_coding_cli_prints_fixture_summary` | Built-in local summary fixture | Changed files, command result, audit event count | Pass | M56 |
| GP-04 | File edit preview/apply | Tool/core | `cargo test -p agent-tool file_edit_tool` and `cargo test -p agent-core comprehensive_local_coding_e2e` | Workspace sandbox fixture | Edit preview/apply result, bounded observation, workspace confinement | Pass | M56-02 |
| GP-05 | Diff review evidence | Tool/TUI/Desktop contracts | `cargo test -p agent-tool git_tools`; `cargo test -p agent-tui diff_panel`; `cargo test -p agent-desktop permissions_diffs` | Git fixture and UI snapshot fixtures | Git diff summary and UI-safe diff/permission payloads | Pass | M56-02, M59, M60 |
| GP-06 | Shell command and test loop | Tool/core | `cargo test -p agent-tool shell_command_tool`; `cargo test -p agent-core comprehensive_local_coding_e2e` | Workspace sandbox fixture | Command exit status, stdout/stderr bounds, observation returned to model loop | Pass | M56-03 |
| GP-07 | Command streaming and cancellation | Core | `cargo test -p agent-core streaming_retry_cancel_replay` | Fake streaming provider and runtime fixture | Retry/cancel/replay events with reconstructed final response | Pass | M56-03, M62 |
| GP-08 | Permission ask/allow/deny | Policy/core/CLI/TUI | `cargo test -p agent-policy`; `cargo test -p agent-core approval_resume_recovery`; `cargo test -p agent-cli approval_prompt`; `cargo test -p agent-tui approval_panel` | Approval request fixtures | Safe approval prompts, bridge commands, checkpointed wait, single execution after approval | Pass | M56, M59 |
| GP-09 | Audit and trace summary | CLI/core | `cargo test -p agent-cli trace_commands`; `cargo test -p agent-core audit_export` | Local coding trace fixture | Ordered tool/permission/final trace and redacted audit export | Pass | M56, M62 |
| GP-10 | Checkpoint resume | Core/storage | `cargo test -p agent-core recovery_flow`; `cargo test -p agent-storage checkpoint_store` | In-memory/SQLite stores | Recovery plan from latest checkpoint and replay tail | Pass | M62 |
| GP-11 | Replay compatibility | Storage/eval | `cargo test -p agent-storage replay_compatibility`; `cargo xtask eval` | `docs/eval/default.suite.json`, M04 replay/evidence fixtures, M56 local-coding fixture | Replay fixtures parse, local-coding eval runs, and trace/audit/replay evidence matches goldens | Pass | M56, M62, M65 |
| GP-12 | CLI TTY and batch modes | CLI | `cargo test -p agent-cli cli_modes`; `cargo test -p agent-cli plain_repl` | CLI parser/render fixtures | Prompt, batch, and plain REPL behavior remain stable | Pass | M59 |
| GP-13 | CLI JSON event stream | CLI/UI bridge | `cargo test -p agent-cli json_mode`; `cargo test -p agent-ui-bridge json_frame` | UI bridge frame fixtures | Parseable NDJSON frames with redacted secrets and done count | Pass | M59 |
| GP-14 | TUI daily-driver panels | TUI | `cargo test -p agent-tui` | Panel state/render fixtures | Transcript, composer, approvals, diffs, shell, task, trace, extension panels render from bridge frames | Pass | M59 |
| GP-15 | Desktop bridge and snapshots | Desktop/UI bridge | `cargo xtask desktop-test`; `cargo test -p agent-ui-bridge --test desktop_daily_driver` | Desktop snapshot, bridge frame, Tauri command, and React fixtures; no provider credentials | Workspace dashboard, conversation/task lists, permissions/diffs, provider/settings, trace timeline, runtime logs, and frontend build evidence | Pass | M60 |
| GP-16 | Desktop golden path simulation | Desktop app | `cargo xtask desktop-test`; `npm --prefix apps/desktop test -- desktop-smoke` | Credential-free M56-equivalent Desktop fixture over shared bridge/Desktop contracts | User-facing Desktop shell navigates dashboard, conversation, tasks, approvals, visual diff, trace/logs, and settings with redacted evidence | Pass | M60 |
| GP-17 | Provider capability diagnostics | Model/CLI | `cargo test -p agent-model provider_registry`; `cargo test -p agent-cli live_provider_registration` | Fake/OpenAI-compatible config fixtures | Provider config, registry, secret-safe diagnostics | Pass | M57 |
| GP-18 | Native Anthropic/Gemini/local provider smoke | Model/CLI | None yet | Requires provider adapters and opt-in credentials or local endpoint | Tool-capable providers run the same M56 golden path or fail before turn with clear diagnostics | Missing | M57 |
| GP-19 | Context and memory contribution visibility | Context/memory/CLI | `cargo test -p agent-context`; `cargo test -p agent-memory`; `cargo test -p agent-cli memory_commands` | Prompt/memory fixtures | Prompt layers, budget planner, memory retrieval, memory command output | Pass | M58, M64 |
| GP-20 | Repo-aware relevant-file retrieval | Context/core | None yet | Requires repo index/search quality path | Coding task selects relevant files with context quality trace | Missing | M58 |
| GP-21 | Background task lifecycle | Scheduler/core/CLI | `cargo test -p agent-scheduler`; `cargo test -p agent-core background_turn_runner`; `cargo test -p agent-cli task_commands` | Task fixtures | Queued/running/waiting/resume/cancel/notification/artifact states | Pass | M61 |
| GP-22 | Worktree/subagent workflow | Worktree/team/CLI eval | `cargo test -p agent-worktree`; `cargo test -p agent-team`; `cargo xtask eval` (`default-m63-team-workflow-smoke`) | Worktree, blackboard, merge-readiness, and eval smoke fixtures | Isolated worktree management, conflict detection, handoff, blackboard entries, reviewer votes, merge-readiness blockers, policy ceilings, and redaction | Pass | M63 |
| GP-23 | Packaging smoke | xtask | `cargo xtask package-cli --channel dev`; `cargo xtask package-desktop --channel dev` | Fake-provider CLI smoke; Desktop update manifest fixture | Release artifacts or bundle plan under `target/agent-release/` | Documented | M65 |
| GP-24 | Extension/MCP health | CLI/MCP/plugin/Desktop support | `cargo test -p agent-core extension`; `cargo test -p agent-cli plugin_commands skill_root_commands extension_health_commands`; `cargo test -p agent-mcp mcp_server_status_health`; `cargo test -p xtask support_bundle`; `npm --prefix apps/desktop test -- extension-health-ui` | Plugin/skill/MCP/support fixtures | Typed extension health, plugin leases, skill diagnostics, MCP status health, Desktop rendering, and support bundle redaction evidence | Pass | M66 |

## Command Notes

- Real provider smokes must be opt-in and must never require secrets for default CI.
- `cargo xtask ci` remains the broad quality gate, but it is not sufficient by itself to prove the M56 product golden path.
- `cargo xtask eval` validates replay/evidence fixtures and the M56 local-coding golden path.
- Desktop product smoke is covered by `cargo xtask desktop-test`; live-provider packaged Desktop smoke remains future M65 hardening.

## Immediate M56-M60 Implications

- M56 has the GP-01 through GP-11 workflow covered by the deterministic local-coding eval fixture.
- M57 should close GP-18 with provider capability diagnostics and native adapters.
- M58 should close GP-20 with repo-aware context selection and quality traces.
- M59 should make GP-12 through GP-14 feel like one terminal workflow, not separate panels/tests.
- M60 turned GP-15/GP-16 into a credential-free Desktop flow with visual approvals and diffs.
