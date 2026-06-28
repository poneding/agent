# M28-02 Built-in Eval Corpus Spec

## Purpose

Check in a small default evaluation corpus that proves the suite loader, M27 runner, evidence artifacts, and replay fixture compatibility through real repository files.

## Product Context

Developers need a known-good suite that can run without external services. The built-in corpus becomes the default target for CLI and `xtask` gates.

## Scope

This spec delivers:

- `docs/eval/default.suite.json`
- scenario manifests for replay and evidence-only checks
- copied or referenced fixtures/goldens that are deterministic and secret-safe
- smoke tests proving the corpus loads and runs

Owned by other specs:

- suite manifest contracts: M28-01
- golden update workflow: M28-03
- CLI/xtask command surfaces: M28-04 and M28-05

## Architecture

The corpus is repository data. Tests load it through `agent-eval::suite`, not bespoke file parsing.

## File Layout

```text
docs/eval/
  default.suite.json
  scenarios/
  fixtures/
  goldens/
```

## Public Contracts

The default suite ID is `default-regression`.

## State and Failure Handling

- missing corpus files fail tests
- corpus scenarios must be offline/read-only by default
- scenario IDs remain stable across edits

## Security and Privacy

- no live secrets or raw private prompts in checked-in corpus
- goldens contain summaries or redacted reports only

## Testing Strategy

- corpus load test
- corpus run test
- golden comparison smoke test

## Acceptance Criteria

- default suite exists and runs locally.
- focused `agent-eval` corpus tests pass.

## Completion Signal

M28-02 is complete when a default offline corpus is checked in and verified through the suite loader.
