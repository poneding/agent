# M28-04 CLI Eval Suite Commands Spec

## Purpose

Expose scenario suites through CLI commands that can list, validate, run, and update goldens using the shared `agent-eval` suite contracts.

## Product Context

M27 added single-scenario commands. M28 needs suite commands so developers and CI can run the built-in corpus with one invocation.

## Scope

This spec delivers:

- `agent-cli eval suite list --suite <path>`
- `agent-cli eval suite validate --suite <path>`
- `agent-cli eval suite run --suite <path>`
- `agent-cli eval suite update-goldens --suite <path> [--dry-run]`
- text and JSON output with CI-safe exit codes

## Architecture

`agent-cli` parses suite commands and delegates to `agent-eval`.

## State and Failure Handling

- validation errors exit `2`
- failed/invalid runs exit `1`
- skipped-only warn runs exit `0`
- JSON output is parseable for all terminal states

## Security and Privacy

- CLI renders redacted diagnostics only
- update-goldens requires explicit command

## Testing Strategy

- parse tests
- text/JSON suite run tests
- invalid suite tests
- update dry-run tests

## Acceptance Criteria

- CLI suite commands run the default corpus and produce reports/artifacts.

## Completion Signal

M28-04 is complete when suite commands are tested through the real binary.
