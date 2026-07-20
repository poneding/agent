# M61-05 Artifacts, Smoke, Docs, and Closeout Spec

## Purpose

Close M61 by linking task reviews and artifacts into plan/todo evidence, documenting the CLI/Desktop product workflow, and proving the full plan/todo/task UX with credential-free smoke and final milestone gates.

## Product Context

M61-01 through M61-04 made plan/todo state, task summaries, wait controls, and notifications first-class across runtime and UI surfaces. M61-05 completes the milestone by connecting completion artifacts and task review metadata back to the plan/todo evidence model so users can see not only that work finished, but which trace, checkpoint, artifacts, and diagnostics support it.

## Scope

This slice delivers:

- runtime-owned helper logic that appends task review, artifact, trace, checkpoint, and diagnostic evidence to a plan/todo item in a revision-safe update
- focused tests proving evidence linking is bounded, redacted, serde-friendly, and preserves completed todo state
- bridge/CLI/Desktop smoke coverage that exercises plan/todo progress, active todo evidence counts, artifact counts, notifications, and task review artifacts
- CLI and Desktop documentation for credential-free plan/todo/task workflows
- milestone status updates and final M61 verification evidence

Out of scope:

- crash recovery, fork/replay UX, and failed-command retry productization, which begin in M62
- native OS notifications, packaged installers, or live provider Desktop runs
- new artifact storage backends or file materialization beyond existing safe metadata contracts
- team/worktree artifact ownership, which belongs to M63

## Public Contracts

`ruse-core` adds a runtime helper that updates the existing `PlanTodoSnapshot` contract without changing its serde shape:

```rust
pub fn append_task_review_evidence(
    snapshot: &PlanTodoSnapshot,
    item_id: PlanTodoItemId,
    expected_revision: u64,
    review: &TaskReviewSummary,
    artifacts: &[TaskArtifact],
    reason: impl Into<String>,
) -> Result<PlanTodoUpdateOutcome, PlanTodoError>;
```

The helper appends `PlanTodoEvidence` entries with these existing kinds:

- `Task` for the reviewed task id and final state
- `Trace` for `trace_id`
- `Checkpoint` for `checkpoint_id`
- `Artifact` for each artifact in the review
- `Diagnostic` for review diagnostics and missing artifact metadata

If the target todo item has no task id, it is linked to `review.task_id`. Existing non-matching task ids are preserved and represented by diagnostic evidence instead of being overwritten.

## State and Failure Handling

- The helper checks `expected_revision` against the current snapshot and returns `PlanTodoError::StaleRevision` on stale updates.
- Missing todo item ids return `PlanTodoError::MissingItem`.
- Evidence references and summaries are normalized through the existing plan/todo redaction and bounding rules.
- Completed or cancelled todo status is preserved; appending evidence does not reopen or mutate terminal status.
- Missing artifact metadata becomes diagnostic evidence rather than hiding the task review.
- Duplicate artifact ids in the review are appended deterministically in review order; future deduplication can be a separate compatibility-preserving enhancement.

## Security and Privacy

- Artifact paths, hashes, summaries, diagnostics, trace ids, checkpoint ids, and task ids are bounded and redacted before UI exposure.
- Raw artifact contents, raw command output, provider credentials, tool input JSON, and unbounded logs are not exposed by this slice.
- CLI/Desktop docs and smoke fixtures use fake providers and deterministic ids only.
- Presentation layers render the typed evidence; they do not create runtime truth or mutate task/plan state directly.

## Testing Strategy

- `ruse-core` tests for review-to-plan evidence linking, stale revisions, missing artifacts, redaction, and terminal todo preservation
- `ruse-ui-bridge` task review snapshot coverage for safe review/artifact metadata
- `ruse-cli` daily-driver recipe parsing for new plan/todo/task review workflow commands
- `ruse-desktop` smoke fixture coverage for active todo evidence counts and task review artifact conversion
- `apps/desktop` Vitest smoke coverage for plan/todo/task evidence rendering without secret leaks
- final M61 gates: Rust fmt, workspace clippy, workspace tests, Desktop Vitest, Desktop build, and `cargo xtask eval`

## Acceptance Criteria

- Plan/todo evidence can be populated from task review artifacts, trace ids, checkpoint ids, and diagnostics through shared core logic.
- CLI docs include parseable credential-free plan/todo/task workflow recipes.
- Desktop docs describe the M61 plan/todo/task smoke path and artifact evidence expectations.
- Deterministic smoke tests cover plan progress, active todo evidence counts, approval waits, notifications, artifact counts, and task review artifacts.
- Final M61 gates pass before roadmap and milestone docs mark M61 complete.
