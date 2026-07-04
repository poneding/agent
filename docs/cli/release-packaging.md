# CLI Release Packaging

M65 productizes the CLI package smoke so release reviewers can verify a packaged `agent` binary without using production user config or secrets.

## Command

```bash
cargo xtask package-cli --channel dev
```

Optional arguments:

```bash
cargo xtask package-cli --channel beta --target x86_64-unknown-linux-gnu --out-dir target/agent-release
```

## Artifacts

The command writes artifacts under the selected output directory:

- packaged `agent` binary
- `.sha256` checksum file
- release manifest JSON
- release-local smoke root
- release-local smoke config

The release manifest includes:

- schema version
- release channel
- artifact name, target triple, version, path, and checksum
- smoke command list
- install smoke summary with smoke root, config path, check ids, command shapes, and required flags

## Install Smoke Checks

The CLI install smoke runs required checks against the packaged binary:

- `--version`
- `--config-check --config <release-local config>`
- `startup-check`
- `--json --prompt "release smoke"`

All checks run with release-local `HOME` and `USERPROFILE` values. They must not read or write production user config/data.

## Privacy

Release manifests store command shapes and artifact paths, not raw command output. They must not include raw API keys, tokens, passwords, environment dumps, raw prompts, sensitive memory, embeddings, or unbounded logs.

Provider or credential diagnostics should fail before network access when required credentials are missing, and any secret-like values must be redacted before becoming durable release evidence.

## Verification

Focused checks:

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask package_cli
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p xtask --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```
