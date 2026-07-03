# Support Bundle

M65 support bundles collect bounded, redacted evidence for beta troubleshooting. A support bundle is not a debug dump; it is an allowlisted artifact that helps maintainers understand installation, provider setup, migration, package, eval, and Desktop health without exposing private user data.

## Command

Generate a local support bundle manifest and summary:

```bash
cargo xtask support-bundle --channel dev
```

Optional flags:

```bash
cargo xtask support-bundle --channel beta --out-dir target/agent-support
```

Outputs:

- `support-bundle-manifest.json`
- `support-bundle-summary.txt`

The first implementation records allowlisted artifact references and checksums. It does not copy raw logs or arbitrary files.

## Required Sections

Support bundles should include these sections when available:

- app name, version, release channel, target triple, and build metadata
- platform path summary for config, data, cache, logs, plugins, and release smoke roots
- config summary with secret references, not secret values
- onboarding readiness snapshot for settings, paths, secret store health, migrations, and provider/model support
- provider/model readiness and capability diagnostics
- startup migration report and schema/config versions
- package artifact manifest and checksum references
- Desktop update manifest validation summary
- default eval summary and artifact paths
- recent bounded diagnostics
- extension health summary when available
- redaction diagnostics explaining omitted or scrubbed fields

## Forbidden Content

Support bundles must not include:

- raw API keys, tokens, passwords, cookies, SSH keys, or credentials
- raw prompts
- raw model responses beyond bounded summaries
- raw tool input JSON
- raw sensitive memory text
- embeddings
- unbounded command output, stdout, stderr, or logs
- full environment dumps
- arbitrary workspace files
- local keychain values

## Redaction Rules

Redaction should happen before evidence crosses durable, CLI, Desktop, or support-bundle boundaries. The bundle may include stable redaction markers and diagnostics such as:

- `secret_like_value_redacted`
- `sensitive_memory_omitted`
- `log_truncated`
- `section_unavailable`
- `provider_secret_missing`
- `onboarding_secret_store_unavailable`
- `onboarding_migration_blocked`

Redaction diagnostics are useful evidence. They should explain what class of data was omitted without echoing the omitted value.

## Failure Handling

Support bundle generation should continue when optional sections are unavailable. The bundle summary should record:

- section id
- status: `included`, `omitted`, `failed`, or `not_applicable`
- redacted diagnostic message
- next action if one is known

Required release sections should fail the beta gate when missing. Optional sections should produce degraded diagnostics.

## CLI And Desktop Behavior

CLI and Desktop may expose a support bundle command or button only after a typed runtime/xtask contract exists. Presentation layers should show:

- destination path
- included section count
- omitted/failed section count
- redaction diagnostics
- next action

They should not assemble raw support data themselves.

## Onboarding Snapshot Input

M65-04 provides `OnboardingReadinessSnapshot` in `agent-core` and a UI-safe bridge frame. M65-05 support bundle generation records an `onboarding_readiness` section status and must use that typed snapshot when a caller supplies it, instead of collecting raw config files, raw environment variables, keychain values, migration text, or provider endpoints.

The snapshot is already bounded and redacted, but support bundle generation should still treat it as allowlisted evidence and preserve redaction diagnostics.

## M65 Ownership

| Area | Owner |
|---|---|
| bundle orchestration | `xtask` |
| runtime/config/provider diagnostics | `agent-core`, `agent-model`, `agent-storage` |
| telemetry summaries | `agent-telemetry` |
| Desktop update validation | `agent-desktop` |
| CLI rendering | `agent-cli` |
| Desktop rendering | `agent-ui-bridge`, `apps/desktop` |

M65-05 adds the concrete `xtask` contracts, tests, and local command path. M65-06 should fold the command into final beta closeout docs and release evidence.
