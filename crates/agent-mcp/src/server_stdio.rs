use std::collections::BTreeMap;

use agent_policy::PolicyEngine;
use agent_tool::ToolRegistry;
use agent_types::{ActorRef, CorrelationId};
use serde_json::{Map, Value, json};

use crate::{
    McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcRequest, McpJsonRpcSuccess,
    McpPromptExportRegistry, McpResourceExportRegistry, McpServerMethod, McpServerState,
    McpToolExportRegistry,
};

const MAX_FRAME_BYTES: usize = 64 * 1024;

pub struct McpServerFixture {
    state: McpServerState,
    tools: ToolRegistry,
    tool_exports: McpToolExportRegistry,
    prompt_exports: McpPromptExportRegistry,
    resource_exports: McpResourceExportRegistry,
    policy: Box<dyn PolicyEngine>,
    actor: ActorRef,
}

impl McpServerFixture {
    #[must_use]
    pub fn new<P>(
        state: McpServerState,
        tools: ToolRegistry,
        tool_exports: McpToolExportRegistry,
        resource_exports: McpResourceExportRegistry,
        policy: P,
    ) -> Self
    where
        P: PolicyEngine + 'static,
    {
        Self {
            state,
            tools,
            tool_exports,
            prompt_exports: McpPromptExportRegistry::default(),
            resource_exports,
            policy: Box::new(policy),
            actor: ActorRef::system(),
        }
    }

    #[must_use]
    pub fn with_actor(mut self, actor: ActorRef) -> Self {
        self.actor = actor;
        self
    }

    #[must_use]
    pub fn with_prompt_exports(mut self, prompt_exports: McpPromptExportRegistry) -> Self {
        self.prompt_exports = prompt_exports;
        self
    }

    #[must_use]
    pub const fn state(&self) -> &McpServerState {
        &self.state
    }
}

pub fn handle_mcp_server_frame(server: &mut McpServerFixture, frame: &str) -> String {
    let response = match parse_frame(frame) {
        Ok(request) => dispatch_request(server, request),
        Err(error) => Some(failure(McpJsonRpcId::Null, error)),
    };

    response.map_or_else(String::new, render_response)
}

fn parse_frame(frame: &str) -> Result<McpJsonRpcRequest, McpJsonRpcError> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err(McpJsonRpcError::invalid_request("JSON-RPC frame too large"));
    }

    let request = serde_json::from_str::<McpJsonRpcRequest>(frame.trim_end())
        .map_err(|_| McpJsonRpcError::parse_error("invalid JSON-RPC frame"))?;
    if request.jsonrpc != "2.0" {
        return Err(McpJsonRpcError::invalid_request(
            "JSON-RPC version must be 2.0",
        ));
    }
    Ok(request)
}

fn dispatch_request(
    server: &mut McpServerFixture,
    request: McpJsonRpcRequest,
) -> Option<JsonRpcResponse> {
    let id = request.id?;
    let method = match McpServerMethod::parse(&request.method) {
        Ok(method) => method,
        Err(error) => {
            return Some(failure(
                id,
                McpJsonRpcError::method_not_found(request.method, error.to_string()),
            ));
        }
    };

    let result = match method {
        McpServerMethod::Initialize => initialize(server, &request.params),
        McpServerMethod::PromptsList => {
            require_initialized(server).and_then(|()| list_prompts(server, &request.params))
        }
        McpServerMethod::PromptsGet => {
            require_initialized(server).and_then(|()| get_prompt(server, &request.params))
        }
        McpServerMethod::CompletionComplete => {
            require_initialized(server).and_then(|()| complete(server, &request.params))
        }
        McpServerMethod::ToolsList => {
            require_initialized(server).and_then(|()| list_tools(server, &request.params))
        }
        McpServerMethod::ToolsCall => {
            require_initialized(server).and_then(|()| call_tool(server, &request.params))
        }
        McpServerMethod::ResourcesList => {
            require_initialized(server).and_then(|()| list_resources(server, &request.params))
        }
        McpServerMethod::ResourcesTemplatesList => require_initialized(server)
            .and_then(|()| list_resource_templates(server, &request.params)),
        McpServerMethod::ResourcesRead => {
            require_initialized(server).and_then(|()| read_resource(server, &request.params))
        }
    };

    Some(match result {
        Ok(result) => success(id, result),
        Err(error) => failure(id, error),
    })
}

fn initialize(server: &mut McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    require_object_params(params)?;
    server.state.initialized = true;
    Ok(json!({
        "protocol_version": server.state.info.protocol_version,
        "server_info": server.state.info,
        "capabilities": server.state.capabilities,
        "initialized": server.state.initialized,
    }))
}

fn list_prompts(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    require_object_params(params)?;
    serde_json::to_value(server.prompt_exports.list_prompts())
        .map_err(|_| McpJsonRpcError::internal_error("failed to serialize prompt list"))
}

fn get_prompt(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    let params = require_object_params(params)?;
    let name = required_string_param(params, "name")?;
    let arguments = optional_string_arguments_param(params)?;
    let result = server
        .prompt_exports
        .get_prompt_with_arguments(name, &arguments)
        .map_err(|error| McpJsonRpcError::invalid_params(error.to_string()))?;

    serde_json::to_value(result)
        .map_err(|_| McpJsonRpcError::internal_error("failed to serialize prompt result"))
}

fn complete(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    let params = require_object_params(params)?;
    let reference = params
        .get("ref")
        .and_then(Value::as_object)
        .ok_or_else(|| McpJsonRpcError::invalid_params("ref must be an object"))?;
    let argument = params
        .get("argument")
        .and_then(Value::as_object)
        .ok_or_else(|| McpJsonRpcError::invalid_params("argument must be an object"))?;
    let reference_type = required_string_param(reference, "type")?;
    let argument_name = required_string_param(argument, "name")?;
    let argument_value = required_string_param_allow_empty(argument, "value")?;

    let result = match reference_type {
        "ref/prompt" => {
            let name = required_string_param(reference, "name")?;
            server
                .prompt_exports
                .complete_prompt_argument(name, argument_name, argument_value)
        }
        "ref/resource" => {
            let uri = required_string_param(reference, "uri")?;
            server.resource_exports.complete_resource_template_variable(
                uri,
                argument_name,
                argument_value,
            )
        }
        other => Err(crate::McpError::InvalidServerRequest(format!(
            "unsupported MCP completion ref type: {other}"
        ))),
    }
    .map_err(|error| McpJsonRpcError::invalid_params(error.to_string()))?;

    serde_json::to_value(result)
        .map_err(|_| McpJsonRpcError::internal_error("failed to serialize completion result"))
}

fn list_tools(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    require_object_params(params)?;
    Ok(json!({
        "tools": server.tool_exports.list_tools(),
    }))
}

fn call_tool(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    let params = require_object_params(params)?;
    let name = required_string_param(params, "name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = server
        .tool_exports
        .call_tool(
            &server.tools,
            server.policy.as_ref(),
            server.actor.clone(),
            CorrelationId::new(),
            name,
            arguments,
        )
        .map_err(|error| McpJsonRpcError::invalid_params(error.to_string()))?;

    serde_json::to_value(result)
        .map_err(|_| McpJsonRpcError::internal_error("failed to serialize tool result"))
}

fn list_resources(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    require_object_params(params)?;
    Ok(json!({
        "resources": server.resource_exports.list_resources(),
    }))
}

fn list_resource_templates(
    server: &McpServerFixture,
    params: &Value,
) -> Result<Value, McpJsonRpcError> {
    require_object_params(params)?;
    Ok(json!({
        "resourceTemplates": server.resource_exports.list_resource_templates(),
    }))
}

fn read_resource(server: &McpServerFixture, params: &Value) -> Result<Value, McpJsonRpcError> {
    let params = require_object_params(params)?;
    let uri = required_string_param(params, "uri")?;
    let result = server
        .resource_exports
        .read_resource(
            server.policy.as_ref(),
            server.actor.clone(),
            CorrelationId::new(),
            uri,
        )
        .map_err(|error| McpJsonRpcError::invalid_params(error.to_string()))?;

    serde_json::to_value(result)
        .map_err(|_| McpJsonRpcError::internal_error("failed to serialize resource result"))
}

fn require_initialized(server: &McpServerFixture) -> Result<(), McpJsonRpcError> {
    if server.state.initialized {
        return Ok(());
    }
    Err(McpJsonRpcError::invalid_request(
        "MCP server must be initialized before this method",
    ))
}

fn require_object_params(params: &Value) -> Result<&Map<String, Value>, McpJsonRpcError> {
    params
        .as_object()
        .ok_or_else(|| McpJsonRpcError::invalid_params("params must be an object"))
}

fn required_string_param<'a>(
    params: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, McpJsonRpcError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| McpJsonRpcError::invalid_params(format!("missing string param: {name}")))
}

fn required_string_param_allow_empty<'a>(
    params: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, McpJsonRpcError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| McpJsonRpcError::invalid_params(format!("missing string param: {name}")))
}

fn optional_string_arguments_param(
    params: &Map<String, Value>,
) -> Result<BTreeMap<String, String>, McpJsonRpcError> {
    let Some(arguments) = params.get("arguments") else {
        return Ok(BTreeMap::new());
    };
    let arguments = arguments
        .as_object()
        .ok_or_else(|| McpJsonRpcError::invalid_params("arguments must be an object"))?;
    arguments
        .iter()
        .map(|(name, value)| {
            value
                .as_str()
                .map(|value| (name.clone(), value.to_string()))
                .ok_or_else(|| {
                    McpJsonRpcError::invalid_params(format!("argument must be a string: {name}"))
                })
        })
        .collect()
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum JsonRpcResponse {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

fn success(id: McpJsonRpcId, result: Value) -> JsonRpcResponse {
    JsonRpcResponse::Success(McpJsonRpcSuccess::new(id, result))
}

fn failure(id: McpJsonRpcId, error: McpJsonRpcError) -> JsonRpcResponse {
    JsonRpcResponse::Failure(McpJsonRpcFailure::new(id, error))
}

fn render_response(response: JsonRpcResponse) -> String {
    let mut rendered =
        serde_json::to_string(&response).expect("JSON-RPC responses are serializable");
    rendered.push('\n');
    rendered
}
