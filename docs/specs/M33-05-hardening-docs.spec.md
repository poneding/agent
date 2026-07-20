# M33-05 Hardening Docs Spec

## Purpose

Complete M33 user documentation, milestone status, and final quality evidence.

## Product Context

After built-in tool export works, users need the exact manifest and CLI invocation shape, plus maintainers need evidence that the feature remains policy-gated and script-safe.

## Scope

This spec delivers:

- README example for built-in tool export with `--workspace`
- milestone status updates for M33
- final focused tests and workspace quality gate evidence

Owned by other specs:

- runtime behavior implementation
- future Desktop MCP management

## Architecture

Documentation only. Runtime ownership remains in `ruse-mcp`/`ruse-tool`; CLI remains process IO.

## File Layout

```text
README.md
docs/milestones/README.md
docs/plans/M33-mcp-builtin-tool-export-runtime.plan.md
docs/plans/M33-05-hardening-docs.plan.md
```

## Public Contracts

README must show:

```bash
ruse-cli mcp server stdio --profile path/to/mcp-server-exports.json --workspace .
```

and a manifest tool declaration for `workspace.read_file`.

## State and Failure Handling

- M33 remains in progress until final `cargo xtask ci` passes.
- The plan records exact commands run and quality-gate summary.

## Security and Privacy

- Docs must state that built-in tool exports are allow-listed and policy-gated.
- Docs must not suggest exporting shell or edit tools through M33.

## Testing Strategy

- focused M33 tests
- `CARGO_TARGET_DIR=target/codex-m33 cargo xtask ci`

## Acceptance Criteria

- README documents profile tool export accurately.
- milestones mark M33 complete.
- quality gate summary is recorded in plan docs.

## Completion Signal

M33-05 is complete when M33 is committed as complete with passing quality gate evidence.
