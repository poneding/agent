# M63-02 CLI and JSON Team Workflow Commands Spec

## Purpose

Expose the M63 team workflow snapshot through stable CLI text and JSON commands before any side-effecting team execution is wired. This slice makes team status, handoff planning, blackboard inspection, reviewer decisions, and merge readiness inspectable through shared `agent-team` and `agent-ui-bridge` contracts.

## Scope

This slice delivers:

- `agent team status`
- `agent team handoff --objective <text> --role <role> --dry-run`
- `agent team blackboard`
- `agent team review --vote <vote> --summary <text> --dry-run`
- `agent team merge-readiness --dry-run`
- deterministic fixtures for sample, conflict, and policy-blocked team states
- a `team_workflow_snapshot` JSON frame in `agent-ui-bridge`
- focused CLI and bridge tests

## Non-Goals

- No live subagent scheduling.
- No merge execution.
- No durable reviewer vote mutation.
- No Desktop/TUI rendering beyond bridge-safe JSON contracts.
- No duplicate readiness, policy, conflict, or capability-ceiling logic in presentation layers.

## Command Contract

Text commands render concise human-readable summaries derived from the same typed snapshot used by JSON.

JSON commands emit one `UiJsonFrame::TeamWorkflowSnapshot` frame:

```json
{
  "type": "team_workflow_snapshot",
  "data": {
    "operation": "merge_readiness_dry_run",
    "dry_run": true,
    "snapshot": {
      "team_id": "...",
      "status": "merge_ready",
      "readiness": "merge_ready"
    },
    "errors": []
  }
}
```

Operations:

- `status`
- `handoff_dry_run`
- `blackboard_inspect`
- `review_vote_dry_run`
- `merge_readiness_dry_run`

Dry-run-only commands must reject missing `--dry-run`.

## Safety

- Commands use deterministic fixtures until durable team workflow assembly is wired.
- Capability ceilings come from `agent-team` contracts and never expand granted capabilities.
- Conflict readiness comes from `TeamWorkflowSnapshot`.
- Text output and JSON snapshots remain bounded and redacted by underlying contracts.
- Error output is structured in JSON mode.

## Acceptance Criteria

- CLI parses every M63-02 command.
- Text status shows status, readiness, capability ceiling, handoff, blackboard, merge review, conflicts, and diagnostics.
- JSON mode emits `team_workflow_snapshot` frames.
- Handoff, review, and merge-readiness commands require `--dry-run`.
- Conflict fixture reports `has_conflict`.
- Policy-blocked fixture reports `policy_blocked`.
- Review block dry-run reports blocked readiness.
- Focused tests pass for `agent-cli` and `agent-ui-bridge`.
