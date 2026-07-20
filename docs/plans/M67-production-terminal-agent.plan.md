# M67 Production Terminal Agent Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `agent` interactive TTY mode a production fullscreen terminal app backed by `ruse-tui`, `ruse-ui-bridge`, and `ruse-core`, while preserving script-safe prompt, JSON, MCP stdio, non-TTY, and explicit plain REPL behavior.

**Architecture:** `ruse-cli` dispatches modes only. `ruse-tui` owns terminal lifecycle and rendering. `ruse-ui-bridge` owns typed UI frames and commands. `ruse-core` owns turn execution, tool/policy orchestration, checkpointing, cancellation, and recovery. Runtime logic must not be implemented in `plain_repl.rs` or terminal rendering code.

**Tech Stack:** Rust 2024 edition, Crossterm, Ratatui, typed serde contracts, existing `TurnEventStream`, Cargo fmt, Cargo clippy, targeted Cargo tests, and later PTY smoke tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M67-production-terminal-agent.spec.md`
- Read: `docs/specs/M20-terminal-ux-tui-repl.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/README.md`
- Read: `crates/ruse-cli/src/bin/agent.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-tui/src/app.rs`
- Read: `crates/ruse-tui/src/render.rs`
- Read: `crates/ruse-tui/src/terminal.rs`
- Read: `crates/ruse-core/src/turn.rs`

- [x] **Step 1: Compare benchmarks and local implementation**

External findings:

- Codex CLI: terminal-first local coding surface with approvals, sandboxing, slash commands, MCP/customization, noninteractive mode, and multiple product surfaces sharing behavior.
- Claude Code: terminal, IDE, Desktop, and web share one engine; CLI exposes permission modes, plan mode, slash commands, MCP, sessions, background tasks, and recovery workflows.
- Gemini CLI: emphasizes terminal-first automation, built-in search/file/shell/web tools, MCP, checkpointing, custom context, and scriptable noninteractive use.
- Aider: focuses on repo map, git workflow, lint/test loop, broad provider support, images/web pages, and developer control.
- OpenCode: uses a TUI, build/plan agent modes, subagent affordances, and a desktop companion.

Local gap:

- `ruse-tui` already has app state, composer, transcript, panels, terminal guard, and render tests.
- `ruse-ui-bridge` already has typed frames and slash catalog contracts.
- `ruse-core` already has turn streams and runtime events.
- `ruse-cli` still sends interactive no-arg sessions to `plain_repl`, which is the wrong product shell.

### Task 2: M1 Terminal App Shell

**Files:**
- Create: `crates/ruse-tui/src/terminal_app.rs`
- Modify: `crates/ruse-tui/src/lib.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-cli/src/bin/agent.rs`
- Modify: `crates/ruse-cli/src/bin/ruse-cli.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/src/lib.rs`
- Modify: `crates/ruse-cli/tests/agent_entrypoint.rs`
- Create: `crates/ruse-tui/tests/terminal_app.rs`

- [x] **Step 1: Add terminal runner and key mapping**

Add a Crossterm/Ratatui runner that enters raw mode and alternate screen through `TerminalModeGuard`, renders `TuiApp`, handles resize/key events, and restores on exit.

- [x] **Step 2: Route interactive `agent` into terminal app**

Update both binaries so `agent` and `ruse-cli` interactive no-args call the terminal app runner. Keep `agent repl --plain` on the existing plain REPL path.

- [x] **Step 3: Preserve compatibility**

Keep non-TTY no-arg help, `--prompt`, `--json --prompt`, MCP stdio, and explicit `repl --plain` unchanged.

- [x] **Step 4: Add focused tests**

Cover key mapping, submit-to-transcript behavior, clear/quit behavior, terminal guard reuse, and CLI dispatch marker behavior.

### Task 3: M2 Streaming Turn Bridge

**Files:**
- Modify: `crates/ruse-tui/src/terminal_app.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Read/modify as needed: `crates/ruse-core/src/turn.rs`

- [x] **Step 1: Connect submitted prompts to runtime**

Map `TuiEffect::SubmitPrompt` to a bridge command or direct terminal-runtime adapter that calls `ConfiguredTurnRunner::run_stream` without blocking rendering.

- [x] **Step 2: Stream frames into reducer**

Convert runtime events to `UiJsonFrame` and feed them into `TuiAction::BridgeFrame`, including model deltas, final response, turn state, progress, diagnostics, errors, and done. Cancellation remains Step 3.

- [x] **Step 3: Add cancellation**

Map `Ctrl+C` to cancel active turn; map second `Ctrl+C` or `/exit` to quit when idle.

- [x] **Step 4: Verify**

Run fake-provider streaming tests and targeted TUI/CLI tests.

Progress:

- [x] Terminal prompt streaming is wired through `ConfiguredTurnRunner::run_stream_with_options`.
- [x] `ruse-core` exposes a comprehensive turn streaming sink so model, tool, approval, observation, and guard-failure runtime events can be emitted live while still returning a typed `AgentLoopOutcome`.
- [x] The terminal runtime now prefers the comprehensive tool-loop path for tool-capable providers and falls back to model-only streaming when the selected runtime lacks `tool_calls`.
- [x] `ruse-model::ModelRequest` carries provider-neutral tool definitions, and OpenAI-compatible, Anthropic, Gemini, and local request builders emit tool schemas for real provider tool-calling.
- [x] OpenAI-compatible streaming parsers now map Responses and chat-completions function/tool call events back into typed `ModelStreamEvent::ToolCall`.
- [x] Interactive terminal turns share a stable workspace/conversation identity across prompts, while rotating turn/correlation ids per submitted prompt.
- [x] Batch `--prompt` remains a script-oriented one-shot path with fresh runtime ids per invocation.
- [x] Terminal runtime events are persisted to a platform data-directory SQLite event store so recovery, title reconstruction, and conversation replay survive process restart.
- [x] Model-turn count is a soft progress checkpoint rather than a hard `max_model_turns_exceeded` failure; cancellation and the remaining hard budgets retain bounded execution.
- [x] Tool continuations use typed assistant tool-call/tool-result messages after one current user message, internal model rounds do not duplicate `UserInputReceived` in durable history, and all tool calls from one model response are queued and executed in proposal order.

### Task 4: M3 Tool, Approval, Diff, and Command Rendering

**Files:**
- Modify: `crates/ruse-tui/src/transcript.rs`
- Modify: `crates/ruse-tui/src/approval_panel.rs`
- Modify: `crates/ruse-tui/src/diff_panel.rs`
- Modify: `crates/ruse-tui/src/command_panel.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Render lifecycle cards**

Render tool start, output chunk, success, failure, elapsed time, exit code, resource, and sandbox/policy metadata from typed frames.

Progress:

- [x] Preserve `ToolExecutionStarted` and `ToolObservationRecorded` as typed UI payloads instead of generic progress.
- [x] Merge proposed, started, observation, success, and failure updates into one transcript tool card keyed by `call_id`.
- [x] Show exit/resource/output metadata from UI-safe tool result summaries where available.
- [x] CLI terminal turns register workspace-confined read, search, git status/diff, edit, and shell tools with read-only tools allowed and side-effecting tools routed to approval.
- [x] Tool runtime events now carry UI-safe policy decision/resource, sandbox profile, and elapsed milliseconds; UI bridge maps them into typed tool frames and transcript/trace render them without owning tool semantics.

- [x] **Step 2: Render approval decisions**

Support focused approval panels with allow/deny controls, bounded previews, diff cards, expiration state, and no duplicate decisions.

Progress:

- [x] Approval panel keeps a bounded recent-decision list for resolved, denied, expired, cancelled, and consumed approvals.
- [x] Repeated resolution updates for the same request id are deduplicated.
- [x] Focused pending approvals still produce typed allow/deny effects without bypassing policy.
- [x] Comprehensive turns expose pending approval state before the approval request is streamed, and CLI terminal approval allow/deny now resolves through `ApprovalCoordinator` instead of returning a runtime-unavailable error.
- [x] Approved or denied tools now resume the comprehensive turn continuation, feed the observation back into the model loop, and keep the terminal turn active until the continuation finishes or reaches another approval.
- [x] Add richer keyboard affordance labels and explicit disabled/read-only states for expired or already-resolved approvals.
- [x] Route empty-composer approval shortcut keys to typed allow/deny/abort effects instead of inserting prompt text.
- [x] Approval abort now routes through the same terminal approval resolution path as deny, producing typed resolved/error frames instead of an unimplemented local diagnostic.

- [x] **Step 3: Fold long output**

Add bounded output, collapsed sections, and panel navigation without corrupting transcript layout.

Progress:

- [x] Command panel marks stdout/stderr folding with visible character and byte counts.
- [x] Diff panel marks folded previews with current line, total line count, and bounded character limit.
- [x] Add interactive expand/collapse navigation for long command and diff output.

- [x] **Step 4: Verify**

Run rendering fixture tests for tool, approval, diff, and command/test frames.

### Task 5: M4 Session, Slash, and Recovery

**Files:**
- Modify: `crates/ruse-tui/src/slash.rs`
- Modify: `crates/ruse-tui/src/session_panel.rs`
- Modify: `crates/ruse-tui/src/status_panel.rs`
- Modify: `crates/ruse-tui/src/trace_panel.rs`
- Modify: `crates/ruse-ui-bridge/src/slash.rs`
- Modify as needed: `crates/ruse-core/src/terminal_session.rs`

- [x] **Step 1: Expand slash commands**

Support `/status`, `/model`, `/permissions`, `/plan`, `/diff`, `/tools`, `/mcp`, `/memory`, `/compact`, `/retry`, `/clear`, and `/exit` through typed command descriptors.

- [x] **Step 2: Add resume and retry**

Expose recent sessions, restore/switch, failed-turn retry, recovery snapshots, and checkpoint hints in terminal panels.

- [x] **Step 3: Verify**

Run slash catalog, session restore, recovery, and retry tests.

Progress:

- [x] Slash catalog and TUI parser cover `/status`, `/model`, `/permissions`, `/plan`, `/diff`, `/tools`, `/mcp`, `/memory`, `/compact`, `/retry`, `/resume`, `/sessions`, `/copy`, `/theme`, `/clear`, and `/exit`.
- [x] Plain REPL slash compatibility uses the same typed catalog.
- [x] Slash catalog and TUI/CLI slash tests pass.
- [x] TUI startup can hydrate panels from initial typed frames.
- [x] TUI startup emits an `ExtensionSnapshot` frame and `/tools` plus `/mcp` open the extension/MCP panel backed by typed plugin/MCP health snapshots instead of falling back to trace-only navigation.
- [x] CLI TUI startup sends the current repo session snapshot to the sessions panel when available.
- [x] Empty-composer session panel shortcuts emit typed switch-session effects.
- [x] Session titles are deterministically rebuilt from the first persisted user message with bounded redaction; placeholder conversation-start events do not suppress title generation.
- [x] Session switching reopens persistent SQLite history, clears the prior transcript, cancels the superseded turn, and filters its late frames.
- [x] `/resume` routes to the typed terminal sessions panel rather than a local string-only diagnostic.
- [x] `/status` emits a typed `RecoverySnapshot` for the current terminal conversation, and `/resume` emits a typed `RecoveryResume` through `RecoveryEngine.resume` while still opening the sessions panel. Explicit CLI recovery commands keep dry-run inspection.
- [x] `/copy` records the latest completed assistant output, ignores in-progress streaming output, and emits a terminal clipboard effect through OSC52.
- [x] `/theme` has a typed catalog entry, cycles session-scoped terminal themes, updates the status panel, and exposes the current theme in the status line.
- [x] Terminal app has a command-runtime adapter slot; CLI validates session switching through the terminal session directory and reports typed errors for unsupported live approval mutation.
- [x] `/retry` emits a typed command-retry dry-run effect, targets the latest non-succeeded command snapshot when evidence exists, passes UI-safe evidence into the terminal command runtime, and falls back to a typed missing-evidence plan when it does not.
- [x] Command/test snapshots carry a `command_run_id` so TUI retry, command panels, trace, transcript, and JSON consumers share the same evidence key.
- [x] Terminal `/retry` planning no longer uses the CLI fixture runtime; it plans from live UI command evidence or a typed missing-evidence plan.
- [x] Command/test snapshots and retry command summaries now carry structured `program`, `args`, and `timeout_ms`, so retry execution can rerun typed argv instead of parsing display text in CLI/TUI presentation code.
- [x] Core-owned command retry execution can rerun retryable command/test evidence and emits a new command/test report keyed by the retry attempt id.
- [x] Terminal `/retry` executes when live typed evidence exists, emits a `CommandRetryPlan`, streams/persists retry runtime events, and emits a fresh `CommandTestSnapshot`; missing evidence still returns a typed missing-evidence plan.
- [x] Terminal recovery resume is wired through core-owned recovery semantics and renders a typed resume report.
- [x] `/compact` emits a typed conversation compaction effect, runs core-owned event-store compaction for the current terminal conversation, and renders a redacted `ConversationCompaction` frame in transcript and trace.
- [x] `/theme` now emits a typed terminal command effect and the CLI terminal runtime persists `terminal.theme` to `config.yaml` through a core-owned atomic YAML mutation helper, while explicit legacy TOML paths keep TOML write-back; success/failure are reported as typed config diagnostics.
- [x] Recovery resume reports now carry optional core-owned `RecoveryContinuation` evidence with bounded redacted summary, next-prompt payload, readiness counts, and a `can_start_turn` flag rendered by transcript and trace.
- [x] `Ctrl+R` explicitly starts a pending startable `RecoveryContinuation` through the existing terminal turn runtime and queue, without recording the internal recovery prompt as a user message or letting CLI/TUI presentation code auto-run it out of band.

### Task 6: M5 Polish and Robustness

**Files:**
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/src/terminal_app.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Terminal polish**

Handle Chinese wide characters, narrow terminals, no-emoji fallback, theme-aware styles, scrollback, copy affordances, queued input during active turns, and stable dimensions.

Progress:

- [x] Submitted prompts during an active turn are recorded as queued user messages and start automatically after the active turn completes.
- [x] Add render regression coverage for Chinese wide characters in narrow transcript layouts.
- [x] Add render regression coverage for long status text in narrow terminals.
- [x] Slash commands auto-open a catalog-backed suggestion list for `/`, filter by typed command prefix, support up/down selection plus Enter/Tab completion, and close with Esc without writing empty states into the transcript.
- [x] Transcript chat rendering no longer uses a `Transcript` title or border; messages render as a pure stream with theme-aware user, model, tool, approval, progress, and diagnostic styling.
- [x] Bottom status is split into provider/model on the left and real typed context window plus current directory on the right; unavailable context renders as pending/unknown rather than fabricated usage.
- [x] First copy affordance exists for the latest completed assistant output.
- [x] `Ctrl+O` routes to the same latest-output copy affordance as `/copy`.
- [x] Explicit plain REPL/fallback rendering supports `AGENT_NO_EMOJI=1` and `TERM=dumb` no-emoji status labels while preserving the requested Unicode default.
- [x] Transcript rendering is bottom-anchored by default, PageUp/PageDown scroll by terminal viewport height, and Home/End jump to oldest/latest transcript lines.
- [x] `/copy` emits OSC52 and attempts platform clipboard fallback (`pbcopy`, `wl-copy`, `xclip`, `xsel`, or PowerShell `Set-Clipboard`) with typed diagnostics.
- [x] Theme-aware terminal styling exists for `system`, `light`, `dark`, and `high-contrast` session themes through `/theme` and `Ctrl+T`.
- [x] Runtime config accepts `[terminal] theme = "system|light|dark|high-contrast"` and CLI TUI startup initializes the terminal theme from the effective config.
- [x] Theme write-back/config mutation from the live TUI is routed through the terminal command runtime and persists `terminal.theme` atomically when a config file is available.
- [x] Empty startup renders a welcome/workbench view instead of an empty Transcript, backed by typed config/status/extension/session state.
- [x] Composer rendering uses a lightweight `❯` prompt with no placeholder or side borders, and the status bar is reduced to provider/model/current directory/context window.

- [x] **Step 2: Diagnostics**

Surface config diagnostics, provider health, transport fallback, invalid stream events, and secret redaction clearly.

Progress:

- [x] CLI TUI startup emits typed config summary frames when config evaluation succeeds.
- [x] CLI TUI startup emits full typed config snapshot frames with provider support and fallback compatibility when config evaluation succeeds.
- [x] CLI TUI startup emits typed redacted config diagnostic frames when config evaluation fails.
- [x] Status panel renders provider health, selected/fallback/catalog provider roles, missing capabilities, and config diagnostics from typed frames.
- [x] Status panel renders live runtime `Error` and `Diagnostic` frames, current turn state, and recoverable/degraded provider failure state.
- [x] Terminal turn failures are classified into provider-facing error codes for transport, invalid stream event, fallback, rate-limit, timeout, and generic failures.
- [x] Terminal model-only turns emit typed `ModelInvocationAttempts` frames for started/failed attempts, with bounded redacted diagnostics rendered in status, trace, and transcript panels.
- [x] Comprehensive tool-loop outcomes now carry core-owned provider/model invocation attempts, including fallback success paths, and terminal mode emits typed attempt frames after successful, waiting-approval, failed, or cancelled outcomes.
- [x] Narrow/wide terminal rendering tests cover Unicode transcript text and status line stability.
- [x] Per-attempt elapsed time and provider-side request ids are exposed by core model invocation attempts and rendered as bounded/redacted UI metadata in status, trace, and transcript panels.

- [x] **Step 3: Verify**

Run render tests for narrow/wide terminals, Unicode text, status line truncation, and diagnostic frames.

### Task 7: M6 Production Tests and Closeout

**Files:**
- Modify/add tests under `crates/ruse-tui/tests/`
- Modify/add tests under `crates/ruse-cli/tests/`
- Modify runtime tests if bridge contracts change
- Create: `docs/plans/M67-terminal-manual-smoke-checklist.md`
- Update docs and milestone status as needed

- [x] **Step 1: Add PTY and regression tests**

Cover alternate-screen startup, resize, quit, panic restore, streaming, non-TTY help, JSON prompt, MCP stdio, provider fallback, and plain REPL compatibility.

Progress:

- [x] Added a real PTY smoke for `agent` no-args startup and `/exit`, proving alternate-screen enter and restore sequences are emitted by the built binary under an interactive TTY.
- [x] Added terminal state regressions proving resize preserves composer text, transcript records, and status, and `TerminalModeGuard` restores raw/alternate-screen state during panic unwinding.

- [x] **Step 2: Run final gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-cli -p ruse-tui -p ruse-ui-bridge --all-targets -- -D warnings
cargo test -p ruse-tui
cargo test -p ruse-cli --test agent_entrypoint --test plain_repl
```

If runtime contracts changed, also run the relevant `ruse-core`, `ruse-model`, and `ruse-ui-bridge` tests. Before closeout, run:

```bash
cargo test --workspace
```

### Completion Checklist

- [x] External benchmark research and local gap analysis recorded.
- [x] M67 spec exists.
- [x] M67 plan exists.
- [x] M1 terminal app shell implemented.
- [x] Interactive no-args dispatches to terminal app.
- [x] Explicit plain REPL remains available.
- [x] Streaming runtime bridge implemented, with prompt submission, model deltas, final response, errors, cancellation, and done wired.
- [x] Tool/approval/diff rendering productized for current typed UI-safe metadata.
- [x] TUI command effects are routed through a terminal command runtime adapter instead of local presentation diagnostics.
- [x] `/retry` is routed through typed command-retry planning/execution instead of local presentation diagnostics and no longer uses a hard-coded `last-failed` target.
- [x] Manual terminal smoke checklist exists for fullscreen startup, streaming, tools, approvals, recovery commands, compatibility modes, resize, and no-emoji fallback.
- [x] Session/slash/recovery workflows productized.
- [x] Polish and robustness tests complete.
- [x] Final workspace gates pass.
