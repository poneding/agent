# M64-05 Memory Smoke, Docs, and Closeout Spec

## Purpose

Close M64 by proving memory personalization works through credential-free smoke evidence and by documenting CLI/Desktop memory workflows, privacy controls, proposal review, import/export, tombstones, rebuilds, and influence traces.

## Product Context

M64-01 added safe memory product snapshots. M64-02 added CLI/JSON review commands. M64-03 added runtime influence/proposal evidence. M64-04 exposed shared bridge snapshots in TUI/Desktop. M64-05 turns those slices into milestone-level confidence: the default eval suite should prove the memory loop without provider credentials, and user-facing docs should describe what users can inspect and control.

## Scope

This spec delivers:

- a deterministic `memory_smoke` eval scenario in the default suite
- eval expectations for retrieval, influence, proposal decisions, import/export, tombstones, rebuild evidence, disabled-memory behavior, and redaction
- golden output for the default memory smoke
- Desktop memory workflow documentation
- updates to existing CLI memory docs if new closeout details are needed
- M64 roadmap/milestone closeout status after final gates pass

Out of scope:

- live-provider memory quality evaluation
- cross-device memory sync
- remote managed memory
- packaging/keychain/update work
- broad Desktop redesign

## Acceptance Criteria

- `cargo xtask eval` includes a credential-free M64 memory smoke scenario.
- The smoke proves selected/rejected retrieval counts, influence evidence, proposal review/commit evidence, import/export evidence, delete/tombstone behavior, rebuild stale-removal evidence, disabled-memory diagnostics, and redaction.
- CLI and Desktop docs explain memory review, proposal decisions, influence traces, privacy/redaction, retention, import/export, deletion, tombstones, rebuild, and disabled-memory states.
- M64 is marked complete only after final Rust, Desktop, eval, and whitespace gates pass.

## Security and Privacy

- Eval fixtures and goldens must not contain raw sensitive memory text, raw proposal content, embeddings, raw prompts, or secret-like values.
- The smoke must intentionally include secret-like input and assert only redacted summaries survive in serialized evidence.
- Docs must describe defaults precisely: sensitive memory is hidden from default summaries, proposal content is redacted, and export excludes sensitive records unless policy allows otherwise.

## Testing Strategy

Focused checks:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test runner memory
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test manifest memory
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test corpus
cargo xtask eval
```

Final M64 gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
git diff --check
```

## Completion Signal

M64 is complete when the memory smoke is in the default eval suite, docs are updated, final gates pass, milestone/roadmap status is complete, and M65 Packaging, Release, and Beta Gate becomes the next active milestone.
