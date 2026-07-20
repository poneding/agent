# Configuration

Configuration controls runtime behavior without hard-coding product policy into crates.

## Configuration Scopes

Configuration is layered from broad to narrow:

1. built-in defaults
2. user config
3. workspace config
4. project config
5. session overrides
6. task overrides
7. explicit CLI flags or Desktop settings for the current action

Narrower scopes override broader scopes only for keys they explicitly set.

## Configuration Domains

| Domain | Examples |
|---|---|
| model | provider, model name, timeout, token budget |
| tool | enabled tools, default timeouts, output limits |
| policy | sandbox mode, approval behavior, capability grants |
| storage | database path, artifact path, retention |
| memory | enabled scopes, retrieval limits, retention |
| scheduler | concurrency, retry, lease duration |
| MCP | server config, allowed tools, auth |
| plugin | enabled plugins, compatibility, permissions |
| UI | renderer preferences, JSON mode, notification settings |

## Format

Use human-editable config files for local scopes. The default user config path is `~/.config/agent/config.yaml`; legacy `~/.config/agent/agent.toml` may be loaded as a fallback with a migration diagnostic, but new docs and generated examples should use YAML. Use JSON for wire/config exchange with external systems.

Config values that cross process boundaries must have typed serde structs.

## Secrets

Provider API keys may be configured directly as `apiKey` in local `config.yaml` for the default setup path. The loader converts those values into a redacted in-memory secret resolver before runtime use. Environment-backed and other secret-reference mechanisms remain available for advanced setups. Raw secret values must never appear in events, traces, snapshots, diagnostics, logs, support bundles, or memory records.

## Migration

Config schemas carry a version. Migration code belongs near the config domain owner and is tested with fixture files.

Migration rules:

- Parse old config.
- Produce new config.
- Preserve user intent.
- Record removed or renamed keys in diagnostics.
- Never silently enable broader permissions.

## Testing Requirements

- Layering tests prove override order.
- Unknown key behavior is explicit.
- Config migration fixtures cover every version step.
- Direct `apiKey` values and secret references remain redacted after serialization.
