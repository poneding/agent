# M66-06 Hardening Docs and Closeout Spec

## Purpose

Close M66 by documenting the completed extension management workflows, support evidence boundaries, OAuth expectations, and verification gates for plugin, skill root, and MCP server hardening.

M66-06 does not add a new extension runtime. It proves that the M66 slices are understandable, supportable, and ready to hand off to future product work without reopening broad MCP protocol expansion.

## Scope

This slice delivers:

- user-facing extension management and health documentation
- final M66 status updates in roadmap and milestone indexes
- support bundle and smoke matrix alignment for extension evidence
- final gate checklist and closeout evidence
- explicit reminder that Desktop Productization remains a separate track

Out of scope:

- marketplace or hosted extension registry
- large Desktop redesign
- new MCP protocol methods
- OAuth browser/login UX, token vaults, refresh execution, or account settings
- collecting raw manifests, skill prompt bodies, process logs, OAuth materials, or unbounded output

## Workflow Documentation Requirements

The closeout documentation must describe:

- plugin install, inspect, list, enable, disable, and remove flows
- capability acceptance and deterministic lease behavior
- incompatible plugin enablement blocking
- skill root add/list/inspect/enable/disable/remove flows
- skill root registry health refresh and duplicate skill diagnostics
- extension health snapshots across plugin, skill root, and MCP server records
- MCP server status health diagnostics for not initialized, no exports, and capability/export mismatches
- support bundle extension evidence and redaction diagnostics
- OAuth deferral rules for installable MCP workflows

## Contract Boundaries

M66 remains inside the established ownership model:

- `agent-core` owns extension registry and extension health aggregation.
- `agent-plugin` owns plugin manifest, compatibility, and capability validation.
- `agent-skill` owns skill root discovery and skill diagnostics.
- `agent-mcp` owns MCP status, export, gateway, and OAuth foundation contracts.
- `xtask` owns support bundle manifests and release evidence.
- CLI and Desktop render typed, redacted contracts.

Public and bridge-facing evidence must remain typed, serializable, bounded, and redacted.

## Security Requirements

Documentation and support evidence must reinforce these constraints:

- no raw API keys, bearer tokens, refresh tokens, OAuth codes, PKCE verifiers, client secrets, or keychain values
- no raw plugin manifests in support bundles
- no raw skill prompt bodies in support bundles
- no raw MCP process logs or unbounded stdout/stderr
- no raw environment dumps
- no automatic plugin enablement without explicit capability acceptance
- no duplicated policy logic in CLI, TUI, or Desktop presentation code

## Acceptance Criteria

- `docs/extensions/management-health.md` documents plugin, skill root, MCP health, support, and OAuth expectations.
- `README.md`, roadmap, milestone index, backlog, and smoke matrix reflect M66 completion.
- M66-06 spec and plan exist and are marked complete.
- M66 master plan marks M66-06 and final gates complete after verification.
- Final M66 gates pass or any unrelated failure is documented with focused passing evidence.
- `tmp/` remains uncommitted.

## Verification

Closeout gates:

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m66-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m66-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask support-bundle --channel dev
git diff --check
```
