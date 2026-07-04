# M66-05 CLI/Desktop Extension Surfaces and Support Evidence Spec

## Purpose

Expose M66 extension health evidence through user-facing surfaces and support bundle contracts without collecting raw extension manifests, process logs, prompts, OAuth material, or secrets.

M66-01 through M66-03 added the typed health and management contracts. M66-05 wires those contracts into support evidence and lightly improves the Desktop extension health rendering.

## Scope

This slice delivers:

- support bundle section for extension health evidence
- redaction diagnostics for omitted raw extension manifests, raw extension logs, and raw OAuth material
- Desktop extension health diagnostics rendering severity and truncation state
- docs describing support-safe extension evidence
- focused xtask and Desktop UI tests

Out of scope:

- marketplace or hosted extension registry
- copying raw extension files into support bundles
- collecting process logs
- Desktop redesign
- OAuth setup UI

## Support Bundle Contract

Support bundles must include an allowlisted `extension_health_summary` section. The section may be an included metadata-only section when a live extension snapshot is not supplied to `xtask`.

The diagnostic must communicate:

- extension health uses typed snapshots
- raw extension manifests are omitted
- raw extension process logs are omitted
- OAuth/token material is omitted

Redaction diagnostics must include:

- `raw_extension_manifests_omitted`
- `raw_extension_logs_omitted`
- `raw_oauth_material_omitted`

## Desktop Surface

Desktop already renders extension summaries, diagnostics, and capability reviews from `UiExtensionSnapshot`. M66-05 adds:

- diagnostic severity display
- truncation state display
- tests proving redacted values remain absent

## Acceptance Criteria

- `cargo xtask support-bundle --channel dev` manifests include `extension_health_summary`.
- Support bundle redaction diagnostics include extension/OAuth omission markers.
- Support bundle docs explain allowed and forbidden extension evidence.
- Desktop extension health UI renders severity and truncated diagnostics from typed snapshots.
- No raw secret, raw manifest, raw extension log, raw OAuth token, or unbounded output is collected.

## Verification

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p xtask support_bundle
npm --prefix apps/desktop test -- extension-health-ui
npm --prefix apps/desktop run build
cargo fmt --all -- --check
git diff --check
```
