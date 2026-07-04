# Install, Upgrade, and Support

This page describes the M65 dev-channel beta workflow. It is local and deterministic by default: release smokes use isolated roots, fake or missing-credential provider behavior, and redacted evidence.

## CLI Package

Build and smoke-test the CLI package:

```bash
cargo xtask package-cli --channel dev
```

The command writes release evidence under `target/agent-release/`:

- packaged `agent` binary
- checksum file
- release manifest JSON
- release-local smoke root and config
- install smoke summary

The smoke checks run:

- `--version`
- `--config-check --config <release-local config>`
- `startup-check`
- `--json --prompt "release smoke"`

## Desktop Bundle or Plan

Generate a Desktop dev-channel bundle plan and update manifest:

```bash
cargo xtask package-desktop --channel dev
```

Request a real Tauri bundle when local tooling is available:

```bash
cargo xtask package-desktop --channel dev --bundle
```

The default dev-channel path may produce a plan-equivalent artifact instead of a real signed installer. The evidence still records Tauri metadata validation, update manifest validation, smoke root, and required Desktop smoke check ids.

## Upgrade and Onboarding Diagnostics

Run config diagnostics before using a release artifact against real work:

```bash
agent --config-check --config path/to/agent.toml
agent --json --config-check --config path/to/agent.toml
```

JSON mode emits an onboarding readiness snapshot with:

- settings profile and release channel
- platform path summary
- secret store/keychain health and env fallback availability
- startup migration status
- provider/model support against coding-agent requirements
- stable diagnostics and next-action ids

Readiness statuses:

| Status | Meaning |
|---|---|
| `ready` | Required settings, migrations, secret store, and selected provider/model are usable. |
| `degraded` | Runtime can continue, but diagnostics need attention, such as keychain fallback or provider capability gaps. |
| `blocked` | Startup or selected provider readiness is unsafe; fix diagnostics before continuing. |

Secret values are never printed. Stable diagnostic codes may mention classes such as `secret_store`, but messages are redacted and bounded.

## Provider Setup

Provider setup is validated before release evidence or support artifacts should be trusted:

- provider id and model id must be valid
- selected provider must be enabled
- remote providers must use secret references, not raw secret values
- capability mismatches are reported before a coding-agent turn depends on them
- missing credentials should fail before network access

Use env or keychain-backed secret references according to the configured provider. Do not paste raw API keys into support reports or release docs.

## Support Bundle

Generate a support bundle manifest and summary:

```bash
cargo xtask support-bundle --channel dev
```

Outputs:

- `target/agent-support/support-bundle-manifest.json`
- `target/agent-support/support-bundle-summary.txt`

The bundle records allowlisted artifact references, checksums, section statuses, and redaction diagnostics. It does not copy raw logs, raw environments, prompts, keychain values, memory payloads, embeddings, or workspace files.

## Final Dev-Channel Gate

Before marking M65 complete, run:

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m65-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m65-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask package-cli --channel dev
cargo xtask package-desktop --channel dev
cargo xtask support-bundle --channel dev
git diff --check
```

## Troubleshooting

| Symptom | Check |
|---|---|
| CLI package smoke fails | Inspect the release manifest and release-local config path under `target/agent-release/`. |
| Desktop update manifest fails | Re-run `cargo xtask package-desktop --channel dev` and inspect the generated update manifest. |
| Onboarding is degraded | Review the JSON onboarding diagnostics and next-action ids. |
| Onboarding is blocked | Fix migration or selected provider diagnostics before using the runtime. |
| Support bundle omits a section | Missing optional artifacts are recorded as omitted; run the corresponding package, eval, or quality command first. |

M65 dev-channel evidence is local release readiness, not hosted publishing. Signing, notarization, and public update hosting remain future release work unless explicitly added later.
