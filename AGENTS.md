# AGENTS.md

This repository builds `agent`: a Rust-based runtime for CLI/Desktop agents with typed events, tool policy, memory, MCP, plugins, checkpoints, and replay.

Keep contributions aligned with the docs:

- Start from [README.md](README.md), [docs/PRD.md](docs/PRD.md), and [docs/milestones/README.md](docs/milestones/README.md).
- Follow the cross-cutting guidance in [docs/architecture/README.md](docs/architecture/README.md).
- Use `docs/specs/` for design specs and `docs/plans/` for implementation plans.
- Use `docs/templates/` for new specs, plans, ADRs, and plugin manifests.
- Keep runtime logic in crates, not in CLI/Desktop presentation layers.
- Keep public contracts typed, serializable when durable, and covered by tests.
- Run the standard Rust checks before finishing:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Prefer small, reviewable changes that move the active milestone forward.
