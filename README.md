# agent

agent is a local-first runtime for building reliable AI agents. It provides the shared core for CLI, Desktop, background tasks, tools, memory, MCP, plugins, and multi-agent workflows.

The project focuses on one thing first: make agent execution observable, permissioned, recoverable, and replayable.

## Status

M01-M25 are implemented. M26 is in progress, adding redacted observability evidence bundles and CLI/Desktop debug surfaces.

## Core Ideas

- One runtime core for CLI, TUI, and Desktop.
- Every important action becomes a typed event.
- Tool use goes through policy, sandboxing, and audit.
- Long-running work is checkpointed and recoverable.
- Models, tools, storage, UI, MCP, memory, and plugins stay replaceable.

## Tech Stack

- Rust workspace
- Tokio for async runtime paths
- Serde for durable and wire schemas
- SQLite first for local persistence
- Structured tracing and audit logs
- MCP-compatible extension surface
- CLI and TUI first, Desktop later through a shared UI bridge

## Architecture

```mermaid
flowchart TD
  UI[CLI / Desktop / API] --> Bridge[UI Bridge]
  Bridge --> Core[Core Runtime]
  Core --> Model[Model Adapter]
  Core --> Tools[Tool Kernel]
  Core --> Policy[Permission Engine]
  Core --> Memory[Memory Engine]
  Core --> Context[Context Manager]
  Core --> Scheduler[Scheduler]
  Core --> Plugins[Plugin Runtime]
  Core --> MCP[MCP Layer]
  Core --> Storage[Storage]
  Core --> Telemetry[Telemetry]
```

## Planned Layout

```text
crates/
  agent-types/
  agent-core/
  agent-model/
  agent-storage/
  agent-tool/
  agent-policy/
  agent-sandbox/
  agent-context/
  agent-memory/
  agent-scheduler/
  agent-worktree/
  agent-team/
  agent-mcp/
  agent-skill/
  agent-plugin/
  agent-ui-bridge/
  agent-cli/
  agent-tui/
  agent-desktop/
docs/
  PRD.md
  architecture/
  adr/
  milestones/
  specs/
  plans/
  templates/
xtask/
```

## Development

Use the Rust-native xtask entrypoint for cross-platform workspace commands:

```bash
cargo xtask check
cargo xtask fmt
cargo xtask fmt-check
cargo xtask clippy
cargo xtask test
cargo xtask desktop-test
cargo xtask cli-prompt -- "hello agent"
cargo xtask cli-json -- "hello agent"
cargo xtask package-cli --channel dev
cargo xtask package-desktop --channel dev
agent-cli --observability-summary
agent-cli --observability-json
```

Run `cargo xtask help` for the full command list.

## CLI Release Packaging

Build and smoke-test a local CLI artifact:

```bash
cargo xtask package-cli --channel dev
```

Outputs are written to `target/agent-release/`:

- `agent-<version>-<target>-<channel>`: packaged CLI binary
- `agent-<version>-<target>-<channel>.sha256`: SHA-256 checksum
- `agent-<version>-<target>-<channel>.json`: release manifest

The packaging flow runs `--version`, `--config-check`, `startup-check`, and a fake-provider JSON prompt smoke before writing the manifest.

## Desktop Release Packaging

Build local Desktop release metadata and validate updater-safe manifests:

```bash
cargo xtask package-desktop --channel dev
```

Outputs are written to `target/agent-release/`:

- `agent-desktop-<version>-<target>-<channel>.plan.txt`: local bundle plan and smoke summary
- `agent-desktop-<version>-<target>-<channel>.update.json`: local update manifest
- `desktop-smoke-root/`: isolated Desktop smoke config/data/cache/log root

## Documentation

- [PRD](docs/PRD.md)
- [Architecture guide](docs/architecture/README.md)
- [Architecture decisions](docs/adr/)
- [Milestones](docs/milestones/README.md)
- [Latest completed spec](docs/specs/M26-03-cli-observability-commands.spec.md)
- [Latest completed plan](docs/plans/M26-03-cli-observability-commands.plan.md)
- [M19 technical design](docs/specs/M19-00-interactive-approval-technical-design.spec.md)
- [Templates](docs/templates/)
- [Agent notes](AGENTS.md)

## License

MIT. See [LICENSE](LICENSE).
