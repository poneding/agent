# M65-06 Beta Gate Docs and Closeout Spec

## Purpose

Close M65 by documenting the beta install, upgrade, provider setup, diagnostics, package smoke, Desktop smoke, and support bundle workflows, then running the final dev-channel beta gate.

## Product Context

M65-01 through M65-05 built the beta release baseline:

- CLI package/install smoke
- Desktop bundle/update smoke
- onboarding readiness for settings, secrets, migrations, and provider/model setup
- redacted support bundle evidence

M65-06 turns those slices into a coherent closeout so M66 can start from a credible beta path rather than unfinished release plumbing.

## Scope

This spec delivers:

- final install/upgrade/support workflow docs
- beta gate evidence checklist
- README status and command updates
- milestone/roadmap closeout status
- final M65 gates

Out of scope:

- hosted signing, notarization, public update hosting, or release publishing
- broad Desktop redesign
- live-provider benchmarking that requires user credentials
- new extension/MCP hardening beyond release health references

## Required User Workflows

M65 closeout docs must cover:

- build or plan a dev-channel CLI package
- build or plan a dev-channel Desktop bundle/update manifest
- run release-local package smoke checks
- run onboarding readiness diagnostics
- configure provider/model and secret references without exposing secret values
- understand migration readiness and blocked/degraded states
- generate a support bundle
- inspect release evidence paths
- recover from missing package, migration, keychain, provider, eval, or Desktop smoke evidence

## Final Dev-Channel Gate

M65 closeout requires:

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

## Security And Privacy

Closeout documentation and release evidence must preserve M65 privacy guarantees:

- no raw API keys, tokens, passwords, cookies, SSH keys, or keychain values
- no raw environment dumps
- no raw prompts or model responses
- no raw sensitive memory
- no embeddings
- no arbitrary workspace files
- no unbounded logs

## Acceptance Criteria

- M65-06 spec and plan exist.
- Install/upgrade/support docs exist and link CLI/Desktop package docs, beta gate docs, and support bundle docs.
- README reflects the current M65/M66 status and includes release/support commands.
- M65 milestone plan, milestone index, and product roadmap mark M65 complete.
- Final dev-channel gate commands pass or any failure is fixed before closeout.
- M66 Extensions and MCP Hardening is marked as the next active milestone.

## Completion Signal

M65 is complete when docs are updated, final gates pass, release/support evidence is generated, status files mark M65 complete, and M66 is the next active milestone.
