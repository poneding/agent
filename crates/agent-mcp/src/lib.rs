#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod config;
pub mod error;
pub mod gateway;
pub mod policy;
pub mod server;
pub mod server_completion;
pub mod server_manifest;
pub mod server_profile;
pub mod server_prompt;
pub mod server_resource;
pub mod server_session;
pub mod server_status;
pub mod server_stdio;
pub mod server_tool;
pub mod server_trace;
pub mod tool;
pub mod tool_mapping;

pub use client::{
    FixtureMcpClient, McpClient, McpGatewayClient, McpInitializeResult, McpServerCapabilities,
    McpServerHealth, McpServerStatus, check_mcp_server_health,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use gateway::{McpGateway, McpGatewayDiagnostic, McpGatewayReport, McpGatewayToolDescriptor};
pub use policy::{
    McpPermissionProfile, McpPolicyMapping, default_policy_mapping_for_mcp_tool,
    permission_request_for_mcp_mapping,
};
pub use server::{
    MCP_SERVER_PROTOCOL_VERSION, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId,
    McpJsonRpcRequest, McpJsonRpcSuccess, McpServerInfo, McpServerMethod, McpServerState,
};
pub use server_completion::{
    McpCompletionArgument, McpCompletionRef, McpCompletionRequest, McpCompletionResult,
    McpCompletionValues,
};
pub use server_manifest::{
    MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION, McpServerBuiltinToolExportCatalogConfig,
    McpServerExportManifest, McpServerExportProfileConfig, McpServerProfileInspection,
    McpServerProfileInspectionDiagnostic, McpServerProfileInspectionOptions,
    McpServerProfileInspectionSeverity, McpServerProfilePromptArgumentInspection,
    McpServerProfilePromptInspection, McpServerProfileResourceInspection,
    McpServerProfileResourceTemplateInspection, McpServerProfileToolInspection,
    McpServerPromptArgumentConfig, McpServerPromptExportConfig, McpServerPromptMessageConfig,
    McpServerResourceExportConfig, McpServerResourceTemplateExportConfig,
    McpServerToolExportCatalog, McpServerToolExportConfig,
    build_mcp_server_builtin_tool_export_catalog, build_mcp_server_profile_from_manifest,
    build_mcp_server_resource_profile_from_manifest, build_mcp_server_tool_profile_from_manifest,
    inspect_mcp_server_export_profile, parse_mcp_server_export_manifest_json,
    select_mcp_server_export_profile,
};
pub use server_profile::{
    McpServerExportProfile, McpServerExportProfileBuilder, McpServerProfileRuntime,
    empty_mcp_server_export_profile,
};
pub use server_prompt::{
    McpPromptArgument, McpPromptDescriptor, McpPromptExport, McpPromptExportRegistry,
    McpPromptGetParams, McpPromptGetResult, McpPromptListResult, McpPromptMessage,
};
pub use server_resource::{
    McpResourceExport, McpResourceExportRegistry, McpResourceReadResult, McpResourceTemplateExport,
    McpResourceTemplateMatch,
};
pub use server_session::{
    McpServerSession, McpServerSessionConfig, McpServerSessionStatus, McpServerSessionSummary,
};
pub use server_status::{
    McpServerStatusCapabilities, McpServerStatusSummary, mcp_server_status_summary,
};
pub use server_stdio::{McpServerFixture, handle_mcp_server_frame};
pub use server_tool::{McpToolExport, McpToolExportRegistry, exported_tool_from_definition};
pub use server_trace::{
    McpServerTraceEntry, McpServerTraceSnapshot, format_mcp_server_trace_summary,
};
pub use tool::{McpToolCallRequest, McpToolCallResult, McpToolDescriptor, McpToolListResult};
pub use tool_mapping::{
    McpLocalToolName, McpToolDefinition, local_tool_name_for_mcp_tool,
    mcp_descriptor_to_tool_definition,
};
