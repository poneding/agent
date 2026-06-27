# M16-04 CLI Prompt Inspection Spec

## Purpose

Expose prompt profile and effective prompt diagnostics through CLI commands without revealing hidden or sensitive prompt content.

## Product Context

Users need to understand why the agent behaved a certain way. M16-04 gives CLI users inspectable prompt composition while preserving the same privacy boundaries that TUI and Desktop will later use.

## Scope

This slice delivers:

- CLI parsing for `agent prompt profiles`
- CLI parsing for `agent prompt inspect`
- JSON output for prompt inspection
- text output for prompt inspection
- safe diagnostics for hidden and secret-like layers
- tests proving prompt inspection does not call model providers

Owned by later slices:

- TUI prompt panel
- Desktop prompt panel
- prompt editing UX

## Architecture

`agent-cli` owns command parsing and rendering. It asks `agent-context` to assemble a built-in/default prompt inspection fixture using current effective config. The CLI does not persist events or invoke model providers for inspection. JSON output should use existing `UiJsonFrame` patterns where practical, but it may return a CLI-specific summary until M16-05 adds a formal UI bridge payload.

## CLI Contracts

Commands:

```bash
agent prompt profiles
agent prompt inspect
agent prompt inspect --json
agent --json prompt inspect
```

Text output includes:

- selected profile id and version
- display name
- included layer ids
- omitted layer ids
- estimated token counts
- diagnostics
- bundle hash when M16-03 is available

Text output excludes:

- hidden layer content
- secret-like layer content
- raw environment values
- raw provider secrets

## State And Failure Handling

- Unknown prompt subcommands return `CliError::UnknownArgument`.
- Prompt inspection fails before model invocation if prompt profile assembly fails.
- JSON mode emits parseable frames and an error frame on failure.
- `--config-check` remains separate and does not inspect prompts.

## Security And Privacy

Prompt inspection treats project files, skills, memory, and config overlays as untrusted. Hidden layers show ids and source labels only. Secret-like layers show redacted labels and hashes only. CLI error messages must not include prompt content.

## Testing Strategy

- command parser accepts `prompt profiles`
- command parser accepts `prompt inspect`
- text `prompt inspect` renders profile id and diagnostics
- JSON `prompt inspect` emits parseable frames
- prompt inspection output excludes hidden content
- prompt inspection output excludes secret-like content
- prompt inspection does not invoke model providers

## Acceptance Criteria

- CLI users can list prompt profiles and inspect the effective prompt composition.
- Text and JSON output are stable enough for future TUI/Desktop bridge reuse.
- Hidden and sensitive content remain protected.

## Completion Signal

M16-04 is complete when focused CLI prompt inspection tests pass, workspace gates pass, and active implementation advances to M16-05.
