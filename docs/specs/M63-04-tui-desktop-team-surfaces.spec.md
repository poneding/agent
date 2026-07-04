# M63-04 TUI/Desktop Team Surfaces Spec

## Purpose

Render team workflow state in TUI and Desktop from shared bridge snapshots without reimplementing readiness, policy, conflict, or merge-review logic in presentation layers.

M63-01 through M63-03 made team workflow and merge readiness typed. M63-04 makes that state visible in local UI surfaces so users can see handoffs, blackboard evidence, capability ceilings, conflicts, pending approvals, and merge readiness.

## Scope

This slice delivers:

- TUI task panel rendering for `UiJsonFrame::TeamWorkflowSnapshot`
- Desktop typed team workflow snapshot interfaces
- Desktop team workflow component rendered in the Tasks view
- focused TUI and Desktop tests

## Non-Goals

- No broad Desktop redesign.
- No live provider calls.
- No local readiness calculation in TypeScript or TUI.
- No merge execution.
- No new Desktop routing beyond the current Tasks view.

## UI Contract

Both TUI and Desktop consume:

- `operation`
- `dry_run`
- `snapshot.status`
- `snapshot.readiness`
- `snapshot.pending_approval_count`
- `snapshot.worktrees`
- `snapshot.handoffs`
- `snapshot.blackboard`
- `snapshot.merge_review`
- `snapshot.capability_ceiling`
- `snapshot.conflicts`
- `snapshot.diagnostics`
- `errors`

The UI must display labels and counts from the snapshot directly. It may format text, but it must not derive a different readiness state.

## Acceptance Criteria

- TUI task panel renders team workflow snapshot state.
- TUI app reducer routes team frames into the task panel.
- Desktop Tasks view renders a team workflow panel when provided a team snapshot.
- Desktop panel shows readiness, operation, handoffs, blackboard counts, capability ceiling, merge review, conflicts, and diagnostics.
- TUI and Desktop tests cover team snapshot rendering.
