# Daily Driver CLI Recipes

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
