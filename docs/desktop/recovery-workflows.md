# Desktop Recovery Workflows

Desktop recovery views are consumers of the same runtime and bridge contracts used by CLI and TUI. Desktop does not classify recovery readiness, retryability, or conflicts in React; it renders typed summaries from Rust.

## Restart Expectations

After a crash or app restart, Desktop should reopen the workspace/session shell and request runtime recovery state. The user-facing recovery state should make these facts visible when present:

- latest checkpoint and replay metadata
- recovery readiness
- pending tool calls
- reusable idempotent effects
- approval waits
- retryable command/test failures
- worktree conflict count, paths, kinds, and next actions
- bounded diagnostics and trace links

## Readiness States

Desktop should visually distinguish:

- `ready`: resume is safe when the runtime enables it
- `needs_approval`: user action is required in the approval panel
- `pending_tool_calls`: tool work needs runtime continuation or review
- `has_conflict`: automatic resume is blocked until conflicts are inspected
- `corrupt_checkpoint`, `storage_error`, or `blocked`: direct resume is unsafe

Unsafe states should explain the reason and expose the next typed action instead of offering a blind resume button.

## Task And Conflict Summaries

M62 exposes recovery conflicts through Desktop-safe task models. Task detail can show:

- conflict count
- relative path summary
- conflict kind such as `both_modified`
- next action such as `inspect_diff`

Desktop must not show raw file contents, patch hunks, provider prompts, raw command logs, or secret-like hashes in recovery summaries.

## Approvals And Retry

Approval waits remain explicit. Desktop should route the user to the approval detail panel, where allow/deny decisions keep policy rationale, preview summary, lease, and redaction metadata visible.

Retryable command/test failures should be shown as dry-run plans until execution is wired through policy-backed command/test paths. A retry plan should show previous attempt id, next attempt id, command summary, classification, readiness, exit code, and diagnostics.

## Replay And Fork

Replay inspect and fork are review actions. Desktop should treat them like CLI:

- inspect replay before resuming or forking
- fork only when the runtime returns a typed dry-run or explicit future execution command
- keep checkpoint, source turn, replay count, and diagnostics visible

## Closeout Smoke

The M62 recovery smoke is credential-free and runs outside Desktop:

```bash
cargo test -p ruse-core --test crash_resume_smoke
cargo xtask eval
```

The default eval suite includes `default-m62-recovery-crash-smoke`. Desktop productization can later add a visual recovery screen smoke, but M62 closes on the shared runtime/bridge recovery contract and the existing Desktop-safe task rendering fixtures.
