# Daily Driver CLI Recipes

## Interactive daily path (primary)

The production coding-agent surface is the **TUI**:

```bash
agent                 # interactive TTY → TUI (composer, tools, approvals)
```

That path runs the full agent loop (`AgentSession` / `ComprehensiveTurnRunner`): model stream → workspace tools → policy → approvals → resume. Prefer it for real coding work.

Supporting modes:

| Mode | Command | Tools | Notes |
|---|---|---|---|
| TUI (default) | `agent` | yes | Daily driver |
| Batch prompt | `agent -p "..."` | no | Model-only; scripts/CI |
| JSON stream | `agent --json -p "..."` | no | Machine-readable frames |
| Plain REPL | `agent repl --plain` | no | Lightweight chat, not full agent |

Do **not** expect tool use or approval queues from batch/plain REPL. Management commands below remain offline fixture recipes for CI and diagnostics.

### Workspace context (TUI)

Interactive coding turns load (when present, first match wins):

- `AGENTS.md` / `CLAUDE.md` / `AGENT.md` / `.agents/instructions.md`
- short `git status --short --branch` summary

Session history now reconstructs tool calls/results across restarts (not only final assistant text).

### Lean core (optional)

Interactive coding does not require MCP. Build/check a lean core with:

```bash
cargo check -p agent-core --no-default-features
```

Default features still enable `mcp` so CLI management commands keep working.

### Core UI frames (TUI hot path)

Interactive hosts should prioritize: `event`, `model_selection`, `approval_queue_snapshot`, `edit_diff_snapshot`, `terminal_session_snapshot`, `terminal_status_snapshot`, `done`, `error`. Project-context retrieval detail is one `project_context_detail` frame (not four). Other `UiJsonFrame` variants are management/admin snapshots (project-context × N, team, memory, eval, recovery reports) and must not block chat UX work. See `agent_ui_bridge::{is_core_ui_frame, CoreUiFrameKind}`.

---

These recipes use deterministic local commands and fixtures. They do not require provider credentials or network access.

## Session And Status

```bash
agent-cli repo session --workspace .
agent-cli --json repo session --workspace .
agent-cli startup-check
agent-cli --json startup-check
```

## Approval Queue

```bash
agent-cli approval queue --fixture sample
agent-cli --json approval queue --fixture sample
```

## Edit Preview

Create a temporary replacement file before running this against a real workspace. Use preview mode until the diff is reviewed.

```bash
agent-cli repo edit --workspace . --path README.md --replacement-file README.md --preview
agent-cli --json repo edit --workspace . --path README.md --replacement-file README.md --preview
```

## Command And Test Evidence

```bash
agent-cli repo command --workspace . --timeout-ms 30000 -- cargo fmt --all -- --check
agent-cli --json repo test --workspace . --timeout-ms 30000 -- cargo test -p agent-cli --test task_commands
```

## Task Lifecycle

```bash
agent-cli task create --prompt review-workspace-status
agent-cli task list
agent-cli --json task list
agent-cli task inspect 00000000-0000-0000-0000-000000002401
agent-cli task pause 00000000-0000-0000-0000-000000002401 --reason hold
agent-cli task resume 00000000-0000-0000-0000-000000002401 --reason continue
agent-cli task cancel 00000000-0000-0000-0000-000000002401 --reason stop
```

## Plan And Todo Evidence

```bash
agent-cli plan show
agent-cli --json plan show
agent-cli todo list
agent-cli --json todo list
agent-cli todo update --todo todo-1 --status in_progress --revision 1 --reason begin
agent-cli todo update --todo todo-1 --status completed --revision 1 --reason done
```

## Full Fixture Gate

```bash
cargo xtask eval
```

The eval gate runs credential-free fixture scenarios and writes redacted evidence under `target/agent-eval/`. Task review artifacts, trace ids, checkpoint ids, and diagnostics should be attached to plan/todo evidence through shared runtime contracts before a slice is marked complete.
