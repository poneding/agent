# Desktop Team Workflows

M63 adds a Desktop-safe team workflow panel in the Tasks route. The panel renders `team_workflow_snapshot` bridge frames produced from shared runtime/team contracts. Desktop does not implement readiness, policy, conflict, merge-review, or capability-ceiling logic.

## Where It Appears

Open the Tasks view. When the conversation state has a team workflow snapshot, Desktop shows a Team Workflow panel above the task list and task detail.

The panel shows:

- operation name and dry-run state
- workflow status and readiness
- pending approval count
- capability ceiling grant count and policy-blocked state
- merge review approvals, blocking reviews, and artifact count
- blackboard entry, approval, change-request, and block counts
- handoff role, status, and objective
- conflict path, kind, and next action
- diagnostics and bridge command errors

## Readiness States

Readiness values come from `agent-team`:

- `ready_for_handoff`
- `waiting_on_subagent`
- `review_required`
- `merge_ready`
- `has_conflict`
- `policy_blocked`
- `blocked`

Desktop displays the value from the snapshot. If a readiness value looks surprising, inspect the CLI JSON output or trace evidence; do not patch Desktop to derive a different state.

## Dry-Run Behavior

Team workflow actions exposed during M63 are inspection or dry-run oriented. A dry-run snapshot can show what a handoff, review vote, or merge-readiness check would report without mutating durable worktree state.

Desktop should treat `dry_run=true` as evidence that the panel is previewing workflow consequences. It should not offer merge execution unless a later runtime contract adds a typed command for it.

## Conflicts And Blockers

Conflict rows show the bridge-safe `RecoveryConflictSummary` fields:

- path
- kind
- next action

Merge blockers are represented through readiness, merge review counts, conflicts, pending approvals, diagnostics, and capability ceiling state. Desktop should keep raw logs and unbounded diffs out of this panel; detailed diff review belongs in the existing diff/approval surfaces.

## Redaction

Team snapshots are bounded and redacted before reaching Desktop. The UI should preserve that behavior by rendering snapshot fields directly and avoiding raw tool input, raw command logs, provider prompts, or secret values.

## Smoke Evidence

The default eval suite includes `default-m63-team-workflow-smoke`, and the Desktop UI has focused tests for rendering team workflow snapshots in the Tasks route. Run:

```bash
npm --prefix apps/desktop test -- --run src/__tests__/team-workflow-ui.test.tsx
cargo xtask eval
```
