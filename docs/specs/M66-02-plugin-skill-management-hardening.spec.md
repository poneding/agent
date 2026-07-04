# M66-02 Plugin and Skill Management Hardening Spec

## Purpose

Harden user-visible plugin and skill root management now that M66-01 provides a shared extension health baseline.

This slice keeps the existing CLI/runtime ownership model and closes the highest-value management gaps: accepted plugin capabilities must be reflected in extension leases, incompatible plugins must not be re-enabled, and skill root registry health should refresh during management flows.

## Scope

This slice delivers:

- deterministic capability leases when plugin capabilities are accepted during install or enable
- compatibility blocking when enabling an installed plugin
- stable install diagnostics for capability review and unsupported compatibility
- skill root registry refresh on list/management paths so extension health does not go stale
- focused CLI tests for plugin capability acceptance, incompatible enablement, stable JSON diagnostics, and skill root registry refresh
- docs and milestone status updates

Out of scope:

- executing plugin code
- marketplace sync or remote install
- persistent plugin store migration
- Desktop redesign
- MCP OAuth changes

## Behavior

Plugin install:

- `plugin install <path>` keeps plugins disabled by default.
- `plugin install <path> --enable` requires `--accept-capabilities`.
- When capabilities are accepted, the installed record includes deterministic `granted_leases` for declared permissions.
- Unsupported runtime compatibility records remain disabled and unhealthy.
- JSON install output includes stable diagnostics when capability review is required or compatibility is unsupported.

Plugin enable:

- `plugin enable <id>` requires `--accept-capabilities`.
- Enabling an incompatible plugin fails with a redacted diagnostic.
- Enabling a compatible plugin grants deterministic leases for accepted capabilities and marks health healthy.

Plugin disable/remove:

- Disable continues to revoke leases through the shared extension registry.
- Remove continues to delete the extension record.

Skill root management:

- Listing roots refreshes extension registry records from current root inspections.
- Duplicate skill ids detected by `inspect_all` should mark affected extension records unhealthy through existing diagnostics.
- Skill root JSON/text output remains stable and does not include raw skill prompt content.

## Contract Requirements

- Lease ids must be deterministic and not include raw secrets.
- Public JSON fields must remain typed/serializable and redacted.
- Diagnostics should use stable codes:
  - `plugin.capability_review_required`
  - `plugin.unsupported_compatibility`
  - `plugin.enabled`
  - `plugin.installed_disabled`
  - `skill.root_registry_refreshed`
- Presentation layers must not duplicate compatibility or capability policy.

## Acceptance Criteria

- Accepted plugin capabilities produce `granted_leases` in registry/JSON output.
- Disabling a plugin clears granted leases.
- Re-enabling a plugin grants leases again only after explicit capability acceptance.
- Incompatible plugins cannot be enabled through the management command.
- Plugin install JSON contains stable diagnostics for review-required and unsupported compatibility states.
- Skill root list refreshes the extension registry and surfaces duplicate skill-id health through existing extension records.
- Focused plugin and skill root command tests pass.

## Verification

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-cli plugin_commands skill_root_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p agent-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```
