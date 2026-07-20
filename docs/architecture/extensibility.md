# Extensibility

`agent` is designed to accept new providers, tools, MCP servers, skills, plugins, storage backends, and UI clients without rewriting `ruse-core`.

## Extension Rule

Extensions register capabilities. They do not mutate core state directly.

Extension paths:

- model providers through `ruse-model`
- tools through `ruse-tool`
- policy rules through `ruse-policy`
- sandbox executors through `ruse-sandbox`
- MCP servers through `ruse-mcp`
- plugins through `ruse-plugin`
- skills through context/prompt contribution contracts
- UI clients through `ruse-ui-bridge`
- storage backends through `ruse-storage`

## Provider Extension

Provider adapters normalize:

- request shape
- streaming events
- tool call proposals
- token accounting
- finish reasons
- provider errors

Provider-specific data can be stored in metadata maps but must not leak into core state machines.

## Tool Extension

Tools declare:

- name
- version
- input schema
- output schema
- permission requirements
- timeout profile
- artifact behavior

Tool execution always routes through policy and audit.

## Skill Extension

Skills contribute instructions, prompt fragments, workflow rules, and optional assets. Skill loading records source metadata so context assembly remains explainable.

## UI Extension

UI clients consume runtime events and send UI commands through `ruse-ui-bridge`. UI clients do not call tool, storage, or model providers directly.

## Versioning

Version public contracts that cross storage, process, plugin, MCP, or UI boundaries. Prefer additive changes and explicit migrations for breaking changes.
