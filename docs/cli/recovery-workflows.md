# CLI Recovery Workflows

These workflows use typed recovery contracts from the shared runtime. They are safe to run without provider credentials when using fixture-backed state, and JSON output is bounded and redacted.

Use stable ids from the target conversation or task when running against real state.

## Inspect Readiness

```bash
agent-cli recovery status --conversation 00000000-0000-0000-0000-000000000062
agent-cli --json recovery status --conversation 00000000-0000-0000-0000-000000000062 --turn 00000000-0000-0000-0000-000000000063
```

Readiness can be `ready`, `needs_approval`, `pending_tool_calls`, `has_conflict`, `corrupt_checkpoint`, `storage_error`, or `blocked`. Automatic resume should only proceed when the runtime reports a safe readiness state.

## Resume Dry Run

```bash
agent-cli recovery resume --conversation 00000000-0000-0000-0000-000000000062 --dry-run
agent-cli --json recovery resume --conversation 00000000-0000-0000-0000-000000000062 --checkpoint 00000000-0000-0000-0000-000000000064 --dry-run
```

Resume is dry-run only in M62. The report shows replay count, pending tool count, reusable idempotent effect count, readiness, approvals, conflicts, and diagnostics before any future execution path resumes work.

## Replay Inspect And Fork

```bash
agent-cli replay inspect --conversation 00000000-0000-0000-0000-000000000062
agent-cli replay inspect --conversation 00000000-0000-0000-0000-000000000062 --from-checkpoint 00000000-0000-0000-0000-000000000064
agent-cli replay fork --conversation 00000000-0000-0000-0000-000000000062 --turn 00000000-0000-0000-0000-000000000063 --checkpoint 00000000-0000-0000-0000-000000000064 --label follow-up --dry-run
```

Replay inspect explains which event sequence will be replayed. Replay fork remains explicit and dry-run in this product slice so a user can review the source turn, checkpoint, label, and diagnostics first.

## Retry Failed Commands

```bash
agent-cli command retry test-failed --reason fixed-assertion --dry-run
agent-cli --json command retry cargo-test-101 --reason retry-after-recovery --dry-run
```

Retry planning uses previous command/test evidence and reports whether the target is retryable, already succeeded, waiting on approval, policy blocked, missing, or invalid. Retried execution remains future policy-backed work; M62 exposes the safe plan and attempt linkage.

## Inspect Conflicts

```bash
agent-cli conflict inspect --fixture sample
agent-cli --json conflict inspect --conversation 00000000-0000-0000-0000-000000000062 --turn 00000000-0000-0000-0000-000000000063 --fixture sample
```

Conflict reports include relative paths, conflict kind, side summaries, safe hashes, next action, and diagnostics. They never include raw file contents or patch hunks. `has_conflict` blocks automatic resume until the user inspects or resolves the overlap.

## Crash Or Restart

After a crash, restart, or interrupted terminal session:

1. Run `agent-cli recovery status --conversation <id>`.
2. Run `agent-cli recovery resume --conversation <id> --dry-run`.
3. Inspect approval waits, pending tools, reusable effects, conflicts, and diagnostics.
4. Use `command retry --dry-run`, `replay inspect`, or `replay fork --dry-run` when readiness explains that direct resume is not appropriate.

The deterministic closeout gate is:

```bash
cargo test -p agent-core --test crash_resume_smoke
cargo xtask eval
```

Both gates are credential-free. The default eval suite includes `default-m62-recovery-crash-smoke`, which proves checkpoint replay, pending tool recovery, idempotent effect reuse, conflict blocking, and retry planning without live provider calls.

## Redaction

Recovery output is intended for support and review, but it is still bounded and redacted. Paths and hashes that look secret-like are replaced with `[redacted]`; raw tool input, provider prompts, command logs, file contents, and unbounded stderr are not emitted by these recovery surfaces.
