# Beta Gate

M65 defines the beta gate for installable `agent` builds. The beta gate is stricter than generic CI: it must prove packaged CLI/Desktop artifacts, release-local startup behavior, provider diagnostics, default eval evidence, and support artifacts.

## Current Baseline

Existing release foundations:

- `cargo xtask package-cli --channel dev` builds a CLI release artifact, writes a checksum and manifest, and runs release-local CLI smoke commands.
- `cargo xtask package-desktop --channel dev` validates Tauri release metadata, writes a Desktop release plan/update manifest, and runs Desktop update/settings smoke checks.
- `cargo xtask eval` runs the default regression suite with product smokes for local coding, recovery, team workflows, and memory.
- `npm --prefix apps/desktop test` and `npm --prefix apps/desktop run build` verify Desktop UI tests and production frontend build.
- storage/config migration tests exist in `agent-storage` and `agent-core`.
- Desktop update manifest validation exists in `agent-desktop`.

M65 turns these foundations into one beta-grade release workflow with explicit evidence and support artifacts.

## Artifact Matrix

| Artifact | Required | Owner | Evidence |
|---|---:|---|---|
| CLI package | Yes | `xtask`, `agent-cli` | artifact path, manifest, checksum, release-local smoke commands, install smoke summary |
| Desktop bundle or bundle plan | Yes | `xtask`, Tauri app, `agent-desktop` | bundle metadata, update manifest, Desktop smoke root |
| Default eval report | Yes | `agent-eval`, `xtask` | `target/agent-eval/eval-run-report.json`, summary text |
| Startup/migration diagnostics | Yes | `agent-core`, `agent-storage` | migrated config/schema report with diagnostics |
| Provider/keychain diagnostics | Yes | `agent-model`, CLI/Desktop renderers | provider readiness without raw secret values |
| Support bundle | Yes by M65-05 | runtime/telemetry/xtask | redacted archive or directory with support summary |

## Gate Status

| Status | Meaning |
|---|---|
| `passed` | Required artifact/check completed and produced bounded evidence. |
| `failed` | Required artifact/check failed or evidence is missing. |
| `degraded` | Optional evidence is unavailable, but required beta behavior still works and diagnostics explain the omission. |
| `not_run` | The check was intentionally skipped by configuration; beta closeout must not leave required checks in this state. |

## Required Dev-Channel Gate

M65 final closeout should run:

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m65-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m65-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask package-cli --channel dev
cargo xtask package-desktop --channel dev
git diff --check
```

Beta-channel publishing may add signing, notarization, or hosted update service checks later. Those are not required for the local M65 dev-channel gate unless they become repository-supported during M65.

## Smoke Root Rules

Release smokes must use isolated roots:

- no developer production config
- no developer production data
- no raw secrets
- no unbounded logs
- deterministic fake or missing-credential provider behavior by default

Provider checks should fail before network access when credentials are missing.

## Gap Mapping

| Gap | Slice |
|---|---|
| CLI package smoke needs beta-grade install/run evidence | M65-02 |
| Desktop bundle/update smoke needs beta-grade evidence | M65-03 |
| Settings, secrets, migrations, and provider onboarding need product diagnostics | M65-04 |
| Support bundle generation and release evidence need typed redaction | M65-05 |
| Install/upgrade/support docs and final status need closeout | M65-06 |
