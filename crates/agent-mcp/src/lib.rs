#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod client_http;
pub mod client_roots;
pub mod client_sampling;
pub mod client_sampling_stdio;
pub mod client_stdio;
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
    FixtureMcpClient, McpClient, McpClientRootProvider, McpGatewayClient, McpInitializeResult,
    McpSamplingProvider, McpServerCapabilities, McpServerHealth, McpServerStatus,
    check_mcp_server_health, create_mcp_sampling_message, list_mcp_client_roots,
};
pub use client_http::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClientHttpAuthorization,
    McpClientHttpAuthorizationDiagnostic, McpClientHttpAuthorizationDiagnosticCode,
    McpClientHttpAuthorizationDiagnosticSeverity, McpClientHttpAuthorizationReport,
    McpClientHttpCheckpointSummary, McpClientHttpOAuthAuthorizationRequestInput,
    McpClientHttpOAuthAuthorizationRequestPlan, McpClientHttpOAuthMetadata,
    McpClientHttpOAuthPkceChallenge, McpClientHttpOAuthPkceVerifier,
    McpClientHttpOAuthRefreshAction, McpClientHttpOAuthRefreshPlan, McpClientHttpOAuthState,
    McpClientHttpOAuthTokenDiagnostic, McpClientHttpOAuthTokenDiagnosticCode,
    McpClientHttpOAuthTokenDiagnosticSeverity, McpClientHttpOAuthTokenLease,
    McpClientHttpOAuthTokenSourceRef, McpClientHttpOAuthTokenStatus,
    McpClientHttpProtectedResourceMetadata, McpClientHttpReconnectAttempt,
    McpClientHttpReconnectAttemptOutcome, McpClientHttpReconnectDecision,
    McpClientHttpReconnectFailureKind, McpClientHttpReconnectPolicy, McpClientHttpReconnectReport,
    McpClientHttpReplayDiagnostic, McpClientHttpReplayStatus, McpClientHttpRequest,
    McpClientHttpReqwestTransport, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpResponseFrame, McpClientHttpResponseMode, McpClientHttpSessionMetadata,
    McpClientHttpSseEvent, McpClientHttpSseResponse, McpClientHttpStreamDrainReport,
    McpClientHttpStreamOpen, McpClientHttpStreamReconnectReport, McpClientHttpStreamRequest,
    McpClientHttpStreamSupervisorConfig, McpClientHttpStreamSupervisorControl,
    McpClientHttpStreamSupervisorReport, McpClientHttpStreamSupervisorStatus,
    McpClientHttpStreamSupervisorTick, McpClientHttpStreamTransport, McpClientHttpTransport,
    McpHttpClient, build_mcp_client_http_authorization_report,
    build_mcp_client_http_oauth_authorization_request_plan,
    build_mcp_client_http_oauth_refresh_plan, build_mcp_client_http_oauth_token_status,
    build_mcp_client_http_request_with_oauth_token,
    build_mcp_client_http_stream_request_with_oauth_token,
    classify_mcp_client_http_reconnect_error, find_mcp_client_http_response_frame,
    parse_mcp_client_http_oauth_metadata_json,
    parse_mcp_client_http_protected_resource_metadata_json, parse_mcp_client_http_response_body,
    parse_mcp_client_http_response_mode_frames, parse_mcp_client_http_response_text,
    parse_mcp_client_http_session_metadata, parse_mcp_client_http_sse_events,
    render_mcp_client_http_request_body, validate_mcp_client_http_status,
};
pub use client_roots::{
    MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD, McpClientCapabilities, McpClientRootCandidate,
    McpClientRootInspection, McpClientRootInspectionDiagnostic, McpClientRootInspectionEntry,
    McpClientRootInspectionSeverity, McpClientRootRegistry, McpClientRootsCapability, McpRoot,
    McpRootListChangedNotification, McpRootListResult,
};
pub use client_sampling::{
    McpClientSamplingCapability, McpCreateMessageRequestParams, McpCreateMessageResult,
    McpModelHint, McpModelPreferences, McpSamplingContent, McpSamplingContentList,
    McpSamplingDiagnostic, McpSamplingDiagnosticSeverity, McpSamplingIncludeContext,
    McpSamplingMessage, McpSamplingRole, McpSamplingValidationPolicy, McpSamplingValidationReport,
    validate_mcp_sampling_request,
};
pub use client_sampling_stdio::{
    MCP_SAMPLING_CREATE_MESSAGE_METHOD, handle_mcp_sampling_stdio_frame,
    parse_mcp_sampling_stdio_request_frame,
};
pub use client_stdio::{
    FixtureMcpClientStdioTransport, McpClientStdioProcessTransport, McpClientStdioResponseFrame,
    McpClientStdioSession, McpClientStdioSessionStatus, McpClientStdioSessionSummary,
    McpClientStdioTransport, McpStdioClient, parse_mcp_client_stdio_response_frame,
    render_mcp_client_stdio_request_frame,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use gateway::{
    McpGateway, McpGatewayDiagnostic, McpGatewayReport, McpGatewayRootDescriptor,
    McpGatewaySamplingDescriptor, McpGatewayToolDescriptor,
};
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
    McpServerPromptArgumentCompletionConfig, McpServerPromptArgumentConfig,
    McpServerPromptExportConfig, McpServerPromptMessageConfig, McpServerResourceExportConfig,
    McpServerResourceTemplateCompletionConfig, McpServerResourceTemplateExportConfig,
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
    McpPromptArgument, McpPromptArgumentCompletionSource, McpPromptDescriptor, McpPromptExport,
    McpPromptExportRegistry, McpPromptGetParams, McpPromptGetResult, McpPromptListResult,
    McpPromptMessage,
};
pub use server_resource::{
    McpResourceExport, McpResourceExportRegistry, McpResourceReadResult,
    McpResourceTemplateCompletionSource, McpResourceTemplateExport, McpResourceTemplateMatch,
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
