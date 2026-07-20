# CLI Team Workflows

M63 adds credential-free team workflow commands for inspecting parallel worktree/subagent state and merge readiness. These commands render shared `ruse-team` contracts; the CLI does not calculate policy, conflict, review, or readiness rules locally.

## Status

```bash
ruse-cli team status
ruse-cli --json team status
```

The text view summarizes:

- workflow status and readiness
- worktree lease, base branch, worktree branch, and state
- handoff role, status, and objective
- capability ceiling grant count and policy-blocked state
- blackboard entry counts and latest entries
- merge review counts
- conflicts and diagnostics

JSON mode emits one `team_workflow_snapshot` frame. Use JSON mode for automation and Desktop/TUI bridge fixtures.

## Handoff Dry-Run

```bash
ruse-cli team handoff --role implementer --objective "Implement the task" --dry-run
ruse-cli team handoff --role reviewer --objective "Review the worktree output" --dry-run
```

Side-effecting handoff commands currently require `--dry-run`. The dry-run snapshot shows the reduced capability ceiling and resulting readiness without creating durable worktree or subagent state.

Supported roles include the typed `ruse-team` subagent roles such as implementer, reviewer, and researcher.

## Blackboard Inspect

```bash
ruse-cli team blackboard inspect
ruse-cli team blackboard inspect --fixture conflict
```

Blackboard output is bounded and redacted. It shows entry counts, latest summaries, artifact counts, trace-event counts, and review votes.

## Review Dry-Run

```bash
ruse-cli team review --vote approve --summary "Looks ready" --dry-run
ruse-cli team review --vote request-changes --summary "Needs tests" --dry-run
ruse-cli team review --vote block --summary "Unsafe merge" --dry-run
```

Review dry-runs show how votes affect blackboard counts, merge review evidence, and readiness. Blocking votes produce blocked merge-readiness evidence through shared `ruse-team` contracts.

## Merge Readiness Dry-Run

```bash
ruse-cli team merge-readiness --dry-run
ruse-cli team merge-readiness --fixture conflict --dry-run
ruse-cli team merge-readiness --fixture policy-blocked --dry-run
```

Merge readiness dry-runs expose blockers for:

- policy-blocked or empty capability ceilings
- unresolved worktree conflicts
- pending approvals
- stale worktrees
- waiting, failed, or cancelled handoffs
- blocking reviews
- missing approvals
- missing reviewed artifacts

The command does not merge, stage, commit, push, or mutate worktrees.

## Smoke Evidence

`cargo xtask eval` includes `default-m63-team-workflow-smoke`. The smoke proves isolated worktree evidence, handoff evidence, blackboard output, reviewer votes, conflict-aware merge readiness, policy blocking, and redaction without provider credentials.
