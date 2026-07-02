# M63-03 Reviewer Role and Merge Readiness Spec

## Purpose

Make reviewer decisions and merge readiness explicit enough for lead agents, CLI, TUI, Desktop, and later runtime orchestration to know why team output is or is not safe to consolidate.

M63-01 introduced workflow snapshots. M63-02 exposed them through CLI/JSON. M63-03 adds typed merge-readiness evidence: blocker reasons, artifact requirements, stale worktree detection, pending approval blocks, reviewer vote blocks, and conflict/policy blockers.

## Scope

This slice delivers:

- typed merge readiness reports in `agent-team`
- merge blocker kinds for policy, conflicts, pending approvals, stale worktrees, waiting/failed handoffs, blocking reviews, missing reviews, and missing artifacts
- `pending_approval_count` on team workflow snapshots
- readiness derivation that blocks stale worktrees and pending approvals
- deterministic `agent-team` tests proving merge-ready and blocked states

## Non-Goals

- No live Git merge execution.
- No durable reviewer vote storage beyond existing blackboard summaries.
- No TUI/Desktop full team panels; those belong to M63-04.
- No provider calls or autonomous subagent scheduling.

## Contracts

`TeamMergeReadinessReport` summarizes:

- team id and task id
- workflow readiness
- `merge_ready`
- pending approval count
- conflict count
- stale worktree count
- review counts
- artifact count
- blockers
- diagnostics

`TeamMergeBlockerKind` includes:

- `policy_blocked`
- `conflict`
- `pending_approval`
- `stale_worktree`
- `waiting_on_subagent`
- `failed_handoff`
- `blocking_review`
- `missing_review`
- `missing_artifact`

## Readiness Rules

Readiness remains deterministic:

1. Policy blocked or empty capability ceiling -> `policy_blocked`.
2. Conflicts -> `has_conflict`.
3. Pending approvals -> `blocked`.
4. Stale worktrees -> `blocked`.
5. Created/accepted handoffs -> `waiting_on_subagent`.
6. Failed/cancelled handoff or blocking review -> `blocked`.
7. Completed handoff without required review/artifacts -> `review_required`.
8. Completed, approved, artifact-backed, no-conflict state -> `merge_ready`.
9. No handoffs -> `ready_for_handoff`.

## Acceptance Criteria

- Merge-ready snapshots produce a merge-ready report with no blockers.
- Conflicts produce conflict blockers and `has_conflict` readiness.
- Pending approvals produce pending-approval blockers and blocked readiness.
- Stale worktrees produce stale-worktree blockers and blocked readiness.
- Blocking reviews produce blocking-review blockers and blocked readiness.
- Missing artifacts produce missing-artifact blockers and review-required readiness.
- Missing reviews produce missing-review blockers and review-required readiness.
- All report text is bounded/redacted.
