# M28-05 Xtask Eval Gate Spec

## Purpose

Provide a first-class local and CI command that validates and runs the default evaluation corpus without requiring developers to remember raw CLI invocations.

## Product Context

The project already uses `cargo xtask check` for workspace gates. Evaluation should become an adjacent gate that can be adopted by CI workflows.

## Scope

This spec delivers:

- `cargo xtask eval`
- command plan tests
- README documentation
- final M28 status and workspace verification

## Architecture

`xtask` plans a CLI invocation against `docs/eval/default.suite.json`. It does not parse suite manifests itself.

## State and Failure Handling

- CLI non-zero exits propagate from `xtask`
- missing default suite fails clearly
- artifact output goes to `target/ruse-eval`

## Security and Privacy

- eval artifacts are redacted by `ruse-eval`
- `xtask` does not handle secrets

## Testing Strategy

- xtask parser/plan tests
- final `cargo xtask check`

## Acceptance Criteria

- `cargo xtask eval` is documented and planned.
- M28 is marked complete only after final checks pass.

## Completion Signal

M28-05 and M28 are complete when the eval gate works, docs are updated, and the workspace gate passes.
