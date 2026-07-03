# Desktop Release Packaging

M65 productizes Desktop package/update smoke evidence over the existing Tauri 2 app.

## Command

```bash
cargo xtask package-desktop --channel dev
```

By default the command produces a dev-channel release plan and update manifest. To request a real Tauri bundle build:

```bash
cargo xtask package-desktop --channel dev --bundle
```

## Artifacts

The command writes artifacts under the selected output directory:

- Desktop release plan text
- Desktop update manifest JSON
- release-local Desktop smoke root

The release plan records:

- app version
- target triple
- release channel
- whether a real bundle was requested
- smoke root
- smoke command list
- Desktop release smoke summary with check ids and Tauri config path

## Update Manifest

Desktop update manifests are validated by `agent-desktop`. They include:

- release channel
- target version
- artifact platform
- artifact URL
- SHA-256 checksum
- optional signature

Local dev-channel manifests can use `file://` URLs. Hosted stable-channel signing and update distribution are outside M65 unless added by a later release slice.

## Smoke Checks

The Desktop package smoke runs required checks with release-local `HOME` and `USERPROFILE` values:

- `cargo test -p agent-desktop update_manifest`
- `cargo test -p agent-desktop settings`

Tauri metadata is validated before release evidence is written. Desktop frontend UI is not redesigned in this slice.

## Privacy

Desktop release evidence must not include raw keychain values, API keys, tokens, passwords, memory payloads, prompts, workspace files, or unbounded logs. The update manifest remains update metadata only; policy and validation stay in `agent-desktop` and `xtask`.

## Verification

Focused checks:

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask --test package_desktop
CARGO_TARGET_DIR=target/m65-dev cargo test -p agent-desktop update_manifest
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p xtask -p agent-desktop --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```
