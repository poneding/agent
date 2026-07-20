# M67 Terminal Manual Smoke Checklist

Use this checklist before declaring M67 terminal UX production-ready. Prefer a real interactive terminal, because alternate screen, raw mode, resize, clipboard, and provider streaming cannot be fully proven from unit tests alone.

## Setup

- Build the local CLI with `cargo build -p agent-cli --bin agent`.
- Use a workspace with a valid `~/.config/agent/config.yaml`.
- Keep one run with the normal terminal and one run with `AGENT_NO_EMOJI=1`.
- Keep one narrow terminal run around 60 columns and one wide run around 120 columns.

## Fullscreen Shell

- `agent` opens a fullscreen alternate-screen terminal app when stdin/stdout are TTYs.
- The previous terminal contents are restored after `/exit`.
- Automated guard coverage proves raw/alternate-screen state is restored during panic unwinding.
- `Ctrl+C` exits when idle and cancels the active turn when a turn is running.
- Resize does not lose composer text, transcript entries, or active status.
- A fresh session shows the welcome/workbench view with Agent branding, selected provider/model, model health, plugin/MCP summary, current directory, session state, and context window instead of an empty Transcript box.
- The composer has no placeholder, no left/right box borders, and uses only the `❯` prompt.
- The bottom status bar shows only provider, model, current directory, and current/total context length; connection, permission, theme, and mode details stay in `/status`.
- Submitted user text immediately leaves the composer and appears as a transcript record.
- A prompt submitted while another turn is active is shown as queued and starts after the active turn finishes.
- Long transcripts show the latest messages by default; PageUp/PageDown scroll by page and Home/End jump to oldest/latest content.

## Streaming Turns

- A simple prompt streams assistant output incrementally instead of waiting for a full final response.
- Final assistant output is recorded as the latest completed output.
- `/copy` copies only the latest completed assistant output and ignores in-progress streaming text.
- `Ctrl+O` triggers the same copy affordance as `/copy`.
- Copy diagnostics report OSC52 and, when available, platform clipboard fallback such as `pbcopy`, `wl-copy`, `xclip`, `xsel`, or PowerShell `Set-Clipboard`.
- `/theme` and `Ctrl+T` cycle `system`, `light`, `dark`, and `high-contrast`; status line and status panel update without restarting the app, and a typed config diagnostic reports whether `terminal.theme` was persisted.
- A config with `terminal: { theme: dark }` or `terminal: { theme: high-contrast }` starts the TUI in that theme before any `/theme` command is run.
- After `/theme` persists successfully, restarting `agent` starts with the selected theme from `config.yaml`.
- Provider transport, invalid stream event, timeout, rate-limit, and fallback diagnostics render as recoverable terminal/status diagnostics when applicable.

## Tool, Approval, Diff, and Command UX

- A read/search/git-status tool call renders as a single lifecycle card from start to observation/result.
- Shell/edit tools request approval before side effects.
- Approval allow resumes the same turn and feeds the tool observation back to the model.
- Approval deny records a visible denied decision and resumes the model loop with the denial observation.
- Resolved/expired approvals remain read-only and cannot be double-submitted.
- Long command output starts folded with byte/character counts.
- Diff previews start bounded/folded and can be expanded without corrupting transcript layout.

## Slash, Session, and Recovery

- `/status` opens status/recovery information for the current terminal conversation.
- `/compact` records a typed compaction report for the current terminal conversation and shows source, dropped, summary, and token counts without leaking secret-like text.
- `/resume` opens the sessions panel and emits a typed recovery resume report for the current conversation.
- `/sessions` shows recent sessions when available.
- `/retry` executes the latest non-succeeded command evidence through typed argv, then produces a typed retry plan and fresh command evidence; without command evidence it produces a typed missing-evidence plan.
- `/retry` plans from the current TUI command evidence and does not report deterministic fixture diagnostics in terminal mode.
- `/clear` clears the visible transcript without breaking the active terminal app.
- `/help` shows the typed slash catalog.
- `/exit` exits cleanly.

## Compatibility Paths

- `agent --prompt "hello"` remains non-fullscreen and script-safe.
- `agent --json --prompt "hello"` remains machine-readable JSON.
- Non-TTY no-arg invocation prints stable usage/help and exits non-zero.
- MCP stdio mode is not wrapped in TUI formatting.
- `agent repl --plain` remains a text fallback/test harness.
- `AGENT_NO_EMOJI=1 agent repl --plain` and `TERM=dumb agent repl --plain` use `[model]` status labels instead of emoji.

## Closeout Evidence

- Record the provider/model used for the smoke.
- Record the terminal app used for the smoke.
- Automated PTY smoke covers `agent` no-args startup, alternate-screen entry, `/exit`, and alternate-screen restore for the built binary.
- Record any provider-side degraded behavior separately from terminal UI failures.
- Do not mark M67 complete until the focused automated tests and this manual smoke pass.
