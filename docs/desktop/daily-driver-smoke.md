# Desktop Daily Driver Smoke

M60 uses a deterministic Desktop smoke path to prove the app can present an M56-shaped local coding workflow without provider credentials, network access, or real workspace side effects.

Run the Desktop product gate with:

```bash
cargo xtask desktop-test
```

The gate runs:

```bash
cargo test -p agent-desktop
cargo test -p agent-ui-bridge --test desktop_daily_driver
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
```

## Smoke Coverage

The smoke fixture covers:

- workspace dashboard readiness with connected session state, provider readiness, task counts, and pending approval count
- conversation state with submitted prompt evidence, accepted command acknowledgement, transcript progress, and final response
- approval review with explicit allow/deny controls, lease metadata, policy rationale, and redacted preview text
- visual diff review with file summary, bounded hunk lines, selected file, and decision state
- provider/settings diagnostics with fake provider, `fake-scripted` model, streaming/tool capabilities, and no credential fields
- task monitor evidence for running, waiting-approval, succeeded, and failed tasks
- trace timeline and runtime logs for context, permission, command/test, warning, and final response evidence
- bridge command/frame coverage for Desktop start-turn, approval resolution, task notification, approval requested/resolved, final response, done frames, and one-line JSON encoding

## Fixture Rules

The fixture is credential-free and deterministic:

- stable ids and timestamps
- fake provider only
- no live model calls
- no network access
- no raw command output or stderr
- no raw API keys, tokens, passwords, hidden prompt text, tool input JSON, or unredacted diffs
- bounded and redacted text before it crosses Desktop, Tauri, or React boundaries

## Relation To M56

M56 proves the local coding-agent golden path through CLI/eval evidence. The Desktop smoke path proves the same product shape through UI-safe contracts:

- open or resume workspace state
- submit a coding prompt
- observe progress and task state
- review permission and diff evidence
- see command/test evidence
- inspect final trace/log evidence

This is not a live provider Desktop run. Live provider and packaged Desktop beta smokes remain future M65 hardening work.
