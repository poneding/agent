# Desktop Plan/Todo/Task Smoke

M61 extends the Desktop daily-driver smoke path with plan/todo/task evidence. The smoke remains deterministic and credential-free: fake provider data, stable ids, stable timestamps, bounded summaries, and no live model or network calls.

Run the Desktop product gate with:

```bash
cargo xtask desktop-test
```

For the full M61 closeout, also run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

## Required Evidence

The Desktop M61 smoke fixture must show:

- plan progress for at least one task
- an active todo with evidence count
- a waiting approval task with approval/notification evidence
- a succeeded task with artifact count
- a task review artifact with safe path, hash, and summary metadata
- trace/log evidence for command/test completion
- redaction of secret-like text before it reaches React state or rendered UI

## Runtime Ownership

Desktop receives this state from `agent-ui-bridge` snapshots and Desktop Rust view models. Plan/todo transitions, task lifecycle controls, approval waits, task review summaries, and artifact evidence stay in shared runtime crates.

## Secret Safety

Smoke fixtures and docs must not include raw API keys, tokens, passwords, hidden prompts, raw tool input JSON, unbounded command output, or raw artifact contents. Artifact evidence is metadata only: artifact id, kind, optional safe path, optional hash, missing flag, diagnostics, and bounded summary.
