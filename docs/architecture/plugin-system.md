# Plugin System

Plugins extend `agent` without breaking runtime safety or crate boundaries.

## Plugin Types

| Type | Use |
|---|---|
| process plugin | default extension mechanism for local tools and integrations |
| WASM plugin | constrained extensions with a smaller capability surface |
| built-in plugin | trusted runtime-provided extension packaged in the workspace |

Dynamic libraries are reserved for trusted built-ins because they share process memory and have a larger failure surface.

## Manifest

Plugin manifests declare:

- name
- version
- runtime compatibility
- entrypoint
- isolation mode
- contributed tools
- hooks
- configuration schema
- requested capabilities
- required MCP servers or external binaries

The manifest is data. It does not grant permission by itself.

## Loading Flow

1. discover plugin manifest
2. validate schema
3. check version compatibility
4. resolve configuration
5. evaluate requested capabilities
6. start plugin in selected isolation mode
7. register declared contributions
8. route calls through policy and audit

## Tool Contributions

Plugin tools become normal runtime tools. They use the same schema validation, policy evaluation, sandboxing, timeout, cancellation, event emission, and audit path as built-in tools.

## Hooks

Hooks subscribe to lifecycle events. Hook execution has timeouts, capability limits, loop prevention, and trace records.

## Compatibility

Compatibility is checked against:

- manifest schema version
- runtime API version
- protocol version
- declared capability version

Breaking plugin API changes require a migration note and compatibility strategy.

## Testing Requirements

- manifest parser tests
- compatibility matrix tests
- process plugin fixture tests
- capability denial tests
- plugin tool audit tests
- hook timeout tests
