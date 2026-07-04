# Extension Management and Health

M66 productizes extension management for plugins, skill roots, and MCP servers. The goal is practical troubleshooting: users can inspect what is installed, understand compatibility and health, and collect support evidence without exposing raw secrets, raw manifests, raw prompt bodies, process logs, or unbounded output.

## Ownership Model

Extension state follows the repository crate boundaries:

- `agent-core` owns extension registry records, health aggregation, capability-review state, and shared extension snapshots.
- `agent-plugin` owns plugin manifest parsing, compatibility checks, and capability validation.
- `agent-skill` owns skill root discovery and skill diagnostics.
- `agent-mcp` owns MCP server status, export summaries, gateway health, and OAuth foundation contracts.
- `xtask` owns support bundle manifests and summaries.
- CLI, TUI, and Desktop render typed contracts from those crates.

Presentation layers should not duplicate compatibility rules, capability policy, secret handling, or support bundle collection.

## Plugin Management

Plugin commands are intentionally conservative:

```bash
agent-cli plugin install path/to/plugin
agent-cli plugin install path/to/plugin --enable --accept-capabilities
agent-cli plugin list
agent-cli plugin inspect hello-plugin
agent-cli plugin enable hello-plugin --accept-capabilities
agent-cli plugin disable hello-plugin
agent-cli plugin remove hello-plugin
```

Install without `--enable` records the plugin but keeps it disabled. Install or enable with plugin capabilities requires `--accept-capabilities`; accepted capabilities create deterministic extension leases from the manifest's declared permissions. Disabling or removing a plugin clears those leases through the shared extension registry.

Incompatible plugins remain disabled or fail enablement with stable diagnostics. CLI JSON output uses stable result types such as `plugin_install`, `plugin_enabled`, `plugin_disabled`, `plugin_removed`, and `plugin_error`, with diagnostics such as:

- `plugin.capability_review_required`
- `plugin.unsupported_compatibility`
- `plugin.installed_disabled`
- `plugin.enabled`

Plugin support evidence must not copy raw plugin manifests. Use registry records, compatibility summaries, capability-review state, and redacted diagnostics instead.

## Skill Root Management

Skill roots are managed through the existing CLI path:

```bash
agent-cli skill roots add path/to/skills
agent-cli skill roots list
agent-cli skill roots inspect <root-id>
agent-cli skill roots enable <root-id>
agent-cli skill roots disable <root-id>
agent-cli skill roots remove <root-id>
```

Listing and inspecting roots refreshes extension registry health from current skill discovery. Duplicate skill ids and discovery diagnostics become health diagnostics on the root record. Disabled roots remain inspectable but are excluded from active discovery.

Skill root support evidence must not include raw skill prompt bodies. Safe evidence is limited to root metadata, health status, bounded diagnostics, and stable ids.

## Extension Health

Use extension health commands to inspect all extension kinds together:

```bash
agent-cli extension list
agent-cli extension inspect <extension-id>
agent-cli extension health
agent-cli --json extension health
```

The health report is deterministic, bounded, redacted, and credential-free. It summarizes plugin, skill root, and MCP server records with counts by kind and status, bounded diagnostics, and stable next-action ids.

Common diagnostic ids include:

- `extension.none_configured`
- `extension.degraded`
- `extension.unhealthy`
- `extension.disabled`
- `extension.unknown`
- `extension.unsupported_compatibility`
- `extension.capability_review_required`

Desktop and other UI surfaces should render the shared extension snapshot contracts. They may display severity and truncation state, but they should not collect raw extension data locally.

## MCP Server Health

MCP server status remains a typed status report, not a background process launcher:

```bash
agent-cli mcp server status
agent-cli --json mcp server status
agent-cli mcp server profile inspect --profile path/to/mcp-server-exports.json
agent-cli mcp server stdio --profile path/to/mcp-server-exports.json --workspace .
```

M66 adds a health report to MCP server status summaries. It covers:

- not initialized
- no configured exports
- tool exports without matching server tool capability
- resource exports without matching server resource capability

Stable diagnostic ids include:

- `mcp.server.not_initialized`
- `mcp.server.no_exports`
- `mcp.server.tool_exports_without_capability`
- `mcp.server.resource_exports_without_capability`

The status command should not start live servers or collect raw process output. Export and gateway flows still go through MCP profile, gateway, policy, and redaction boundaries.

## OAuth Expectations

M66 does not add browser login, localhost callback hosting, Desktop account UX, durable OAuth token storage, refresh execution, revocation, or provider-specific OAuth UI. Existing `agent-mcp` OAuth contracts already cover metadata parsing, PKCE/state, authorization request planning, callback validation, token exchange construction/execution, token lease conversion, token source references, refresh planning, and bearer request construction.

New OAuth product work should start only when an installable MCP workflow requires it. When that happens, token values, authorization codes, PKCE verifiers, bearer headers, client secrets, refresh tokens, and raw provider responses must stay out of CLI output, Desktop frames, logs, support bundles, and tests.

## Support Evidence

Support bundles include an `extension_health_summary` section:

```bash
cargo xtask support-bundle --channel dev
```

This section may reference typed extension snapshots or metadata-only availability evidence. It must not include:

- raw plugin manifests
- raw skill prompt bodies
- raw extension process logs
- raw MCP stdout or stderr
- raw OAuth callback codes
- access tokens, refresh tokens, bearer headers, client secrets, or PKCE verifiers
- unbounded command output or environment dumps

Expected redaction diagnostics include:

- `raw_extension_manifests_omitted`
- `raw_extension_logs_omitted`
- `raw_oauth_material_omitted`

Redaction diagnostics are themselves useful support evidence because they explain omitted data classes without echoing sensitive values.

## Verification

Focused extension checks:

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-core extension
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-cli plugin_commands skill_root_commands extension_health_commands
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-mcp mcp_server_status_health
CARGO_TARGET_DIR=target/m66-dev cargo test -p xtask support_bundle
npm --prefix apps/desktop test -- extension-health-ui
```

Milestone closeout also runs workspace Rust checks, Desktop tests/build, default eval, support bundle generation, and whitespace validation.
