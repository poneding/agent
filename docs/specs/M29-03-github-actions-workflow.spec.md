# M29-03 GitHub Actions Workflow Spec

## Purpose

Add hosted CI configuration that invokes the shared `xtask ci` quality gate.

## Product Context

M29-02 makes local CI behavior reproducible. M29-03 adds repository-level automation without duplicating gate logic in YAML.

## Scope

This spec delivers:

- `.github/workflows/ci.yml`
- Rust toolchain setup
- cache-safe Cargo invocation
- artifact upload for `target/ruse-ci`

Owned by other specs:

- quality gate execution: M29-02
- artifact content: M29-04

## Architecture

The workflow invokes `cargo xtask ci`. CI does not parse reports or call crate-specific commands directly.

## Public Contracts

```yaml
run: cargo xtask ci
```

## State and Failure Handling

- workflow fails when `xtask ci` exits non-zero
- artifacts upload with `if: always()`
- workflow remains minimal and platform-neutral for the first slice

## Security and Privacy

- no repository secrets are required
- artifacts contain redacted reports only

## Testing Strategy

- static workflow content test in `xtask`
- command string test ensures `cargo xtask ci` is present

## Acceptance Criteria

- workflow exists and points to the shared gate.

## Completion Signal

M29-03 is complete when CI configuration is present and statically verified.
