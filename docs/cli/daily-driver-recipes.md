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
cargo check -p ruse-core --no-default-features
```

Default features still enable `mcp` so CLI management commands keep working.

### Core UI frames (TUI hot path)

Interactive hosts should prioritize: `event`, `model_selection`, `approval_queue_snapshot`, `edit_diff_snapshot`, `terminal_session_snapshot`, `terminal_status_snapshot`, `done`, `error`. Project-context retrieval detail is one `project_context_detail` frame (not four). Other `UiJsonFrame` variants are management/admin snapshots (project-context × N, team, memory, eval, recovery reports) and must not block chat UX work. See `ruse_ui_bridge::{is_core_ui_frame, CoreUiFrameKind}`.

---

These recipes use deterministic local commands and fixtures. They do not require provider credentials or network access.

## Session And Status

```bash
ruse-cli repo session --workspace .
ruse-cli --json repo session --workspace .
ruse-cli startup-check
ruse-cli --json startup-check
```

## Approval Queue

```bash
ruse-cli approval queue --fixture sample
ruse-cli --json approval queue --fixture sample
```

## Edit Preview

Create a temporary replacement file before running this against a real workspace. Use preview mode until the diff is reviewed.

```bash
ruse-cli repo edit --workspace . --path README.md --replacement-file README.md --preview
ruse-cli --json repo edit --workspace . --path README.md --replacement-file README.md --preview
```

## Command And Test Evidence

```bash
ruse-cli repo command --workspace . --timeout-ms 30000 -- cargo fmt --all -- --check
ruse-cli --json repo test --workspace . --timeout-ms 30000 -- cargo test -p ruse-cli --test task_commands
```

## Task Lifecycle

```bash
ruse-cli task create --prompt review-workspace-status
ruse-cli task list
ruse-cli --json task list
ruse-cli task inspect 00000000-0000-0000-0000-000000002401
ruse-cli task pause 00000000-0000-0000-0000-000000002401 --reason hold
ruse-cli task resume 00000000-0000-0000-0000-000000002401 --reason continue
ruse-cli task cancel 00000000-0000-0000-0000-000000002401 --reason stop
```

## Plan And Todo Evidence

```bash
ruse-cli plan show
ruse-cli --json plan show
ruse-cli todo list
ruse-cli --json todo list
ruse-cli todo update --todo todo-1 --status in_progress --revision 1 --reason begin
ruse-cli todo update --todo todo-1 --status completed --revision 1 --reason done
```

## Full Fixture Gate

```bash
cargo xtask eval
```

The eval gate runs credential-free fixture scenarios and writes redacted evidence under `target/ruse-eval/`. Task review artifacts, trace ids, checkpoint ids, and diagnostics should be attached to plan/todo evidence through shared runtime contracts before a slice is marked complete.
