# M29-05 Release Readiness Docs Spec

## Purpose

Close M29 by documenting local and hosted quality gates and marking milestone status from verified evidence.

## Product Context

The project now has runtime features, packaging commands, evaluation suites, and quality evidence. Release-readiness docs should tell maintainers what to run and where to inspect artifacts.

## Scope

This spec delivers:

- README quality gate section updates
- milestone status updates
- final verification notes
- release checklist linkages to `cargo xtask ci`

Owned by other specs:

- report contracts and CI runner: M29-01 and M29-02
- workflow configuration: M29-03

## Architecture

Documentation only; no runtime changes.

## Public Contracts

```bash
cargo xtask ci
```

## State and Failure Handling

- docs only claim commands that are verified in the current workspace
- incomplete gates remain marked in progress

## Security and Privacy

- docs warn that artifacts are redacted summaries, not raw logs

## Testing Strategy

- final `cargo xtask check`
- final `cargo xtask eval`
- optional `cargo xtask ci` after implementation

## Acceptance Criteria

- README and milestones identify M29 as complete.
- final gates pass.

## Completion Signal

M29-05 and M29 are complete when final checks pass, docs are updated, and changes are committed.
