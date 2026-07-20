# M66-01 Extension Health Baseline and Smoke Spec

## Purpose

Define the first M66 slice: a deterministic, credential-free extension health baseline that summarizes plugin, skill root, and MCP server records through typed, bounded, redacted contracts.

M66-01 should make the existing extension foundations easier to verify and support without creating a parallel plugin, skill, or MCP management system.

## Product Context

M65 closed the beta packaging gate. M66 now returns to extension hardening, but only for installable user workflows. Existing foundations already include:

- `ruse-core` extension registry records and snapshots
- CLI `extension list`, `extension inspect`, and `extension health`
- plugin install/enable/disable/remove commands
- skill root management commands
- MCP server status/profile/export commands
- UI bridge extension and MCP management snapshots
- Desktop extension health rendering

M66-01 adds a stable health smoke report over those records so future slices can attach compatibility, support evidence, and richer surfaces without changing the underlying ownership boundaries.

## Scope

This slice delivers:

- a typed extension health smoke report in runtime code
- deterministic aggregation across `plugin`, `skill_root`, and `mcp_server` records
- status counts for healthy, degraded, unhealthy, disabled, and unknown records
- bounded diagnostics with stable codes and redacted messages
- stable next-action ids for empty, unhealthy/degraded, disabled, and capability-review states
- tests proving serialization shape, ordering, redaction, and bounds
- milestone/roadmap status updates

Out of scope:

- marketplace or hosted registry flows
- broad Desktop redesign
- new OAuth browser/token lifecycle work
- new MCP protocol capabilities
- replacing existing plugin, skill, or MCP registries
- support bundle integration beyond documenting the future M66-05 handoff

## Contract Requirements

The health smoke report must be:

- typed and serializable
- deterministic for stable tests and support evidence
- bounded in diagnostic count and diagnostic message length
- redacted for secret-like content
- additive to existing extension records and UI bridge snapshots

The report should include:

- schema version
- total extension count
- counts by kind
- counts by health status
- bounded diagnostics
- stable next-action ids

Diagnostic codes should be stable strings such as:

- `extension.degraded`
- `extension.unhealthy`
- `extension.disabled`
- `extension.unknown`
- `extension.unsupported_compatibility`
- `extension.capability_review_required`
- `extension.none_configured`

Next-action ids should be stable strings such as:

- `extension.add_first`
- `extension.inspect_unhealthy`
- `extension.review_capabilities`
- `extension.enable_when_ready`

## Runtime Boundary

`ruse-core` owns the aggregation because the report summarizes core extension records and will later feed CLI/bridge/support evidence. Presentation layers may render the report or existing snapshots, but they must not implement independent health policy.

## Security And Privacy

- Do not include raw API keys, OAuth tokens, refresh tokens, passwords, or secret-like values.
- Do not include raw manifests, process logs, command output, prompts, or environment dumps.
- Diagnostics must be bounded by count and message length.
- Messages and display fields must pass through existing redaction helpers.
- Unsupported compatibility should disable records and clear leases as existing registry behavior already does.

## Acceptance Criteria

- `ruse-core` exposes a typed extension health smoke report.
- The report covers plugin, skill root, and MCP server records in one deterministic aggregation.
- Empty registries produce a safe diagnostic and next action.
- Unhealthy/degraded/disabled/unknown records produce stable diagnostics.
- Missing capability leases produce a capability-review diagnostic/action.
- Unsupported compatibility produces a compatibility diagnostic.
- Tests prove the report is serializable, sorted/deterministic, bounded, and redacted.
- M66 milestone docs and roadmap identify M66-01 as active.

## Verification

Focused checks:

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p ruse-core extension
CARGO_TARGET_DIR=target/m66-dev cargo test -p ruse-cli extension_health_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p ruse-core -p ruse-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```
