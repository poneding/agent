use ruse_mcp::{
    McpClientCapabilities, McpClientRootsCapability, McpClientSamplingCapability,
    McpCreateMessageRequestParams, McpCreateMessageResult, McpModelHint, McpModelPreferences,
    McpSamplingContent, McpSamplingContentList, McpSamplingIncludeContext, McpSamplingMessage,
    McpSamplingRole,
};
use serde_json::json;

#[test]
fn mcp_sampling_contracts_serialize_client_capabilities() {
    assert_eq!(
        serde_json::to_value(McpClientCapabilities::with_sampling(
            McpClientSamplingCapability::basic(),
        ))
        .expect("json"),
        json!({
            "sampling": {}
        })
    );

    assert_eq!(
        serde_json::to_value(McpClientCapabilities::with_sampling(
            McpClientSamplingCapability::with_tools(),
        ))
        .expect("json"),
        json!({
            "sampling": {
                "tools": {}
            }
        })
    );

    assert_eq!(
        serde_json::to_value(McpClientCapabilities::with_roots_and_sampling(
            McpClientRootsCapability::new(true),
            McpClientSamplingCapability::with_context(),
        ))
        .expect("json"),
        json!({
            "roots": {
                "listChanged": true
            },
            "sampling": {
                "context": {}
            }
        })
    );
}

#[test]
fn mcp_sampling_contracts_serialize_basic_create_message_request() {
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(
            "What is the capital of France?",
        ))],
        100,
    )
    .with_model_preferences(
        McpModelPreferences::new()
            .with_hint(McpModelHint::new("claude-3-sonnet").expect("hint"))
            .with_intelligence_priority(0.8)
            .with_speed_priority(0.5),
    )
    .with_system_prompt("You are a helpful assistant.");

    assert_eq!(
        serde_json::to_value(request).expect("json"),
        json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "What is the capital of France?"
                    }
                }
            ],
            "modelPreferences": {
                "hints": [
                    {
                        "name": "claude-3-sonnet"
                    }
                ],
                "speedPriority": 0.5,
                "intelligencePriority": 0.8
            },
            "systemPrompt": "You are a helpful assistant.",
            "maxTokens": 100
        })
    );
}

#[test]
fn mcp_sampling_contracts_serialize_result_with_stop_reason() {
    let result = McpCreateMessageResult::new(
        McpSamplingRole::Assistant,
        McpSamplingContent::text("The capital of France is Paris."),
        "claude-3-sonnet-20240307",
        Some("endTurn"),
    )
    .expect("result");

    assert_eq!(
        serde_json::to_value(result).expect("json"),
        json!({
            "role": "assistant",
            "content": {
                "type": "text",
                "text": "The capital of France is Paris."
            },
            "model": "claude-3-sonnet-20240307",
            "stopReason": "endTurn"
        })
    );
}

#[test]
fn mcp_sampling_contracts_serialize_content_blocks() {
    let image = McpSamplingContent::image("aW1hZ2U=", "image/png").expect("image");
    let audio = McpSamplingContent::audio("YXVkaW8=", "audio/wav").expect("audio");
    let tool_use = McpSamplingContent::tool_use(
        "call_abc123",
        "get_weather",
        json!({
            "city": "Paris"
        }),
    )
    .expect("tool use");
    let tool_result = McpSamplingContent::tool_result(
        "call_abc123",
        vec![McpSamplingContent::text("Weather in Paris: 18 C")],
    )
    .expect("tool result");

    assert_eq!(
        serde_json::to_value(McpSamplingContentList::from_many([
            image,
            audio,
            tool_use,
            tool_result,
        ]))
        .expect("json"),
        json!([
            {
                "type": "image",
                "data": "aW1hZ2U=",
                "mimeType": "image/png"
            },
            {
                "type": "audio",
                "data": "YXVkaW8=",
                "mimeType": "audio/wav"
            },
            {
                "type": "tool_use",
                "id": "call_abc123",
                "name": "get_weather",
                "input": {
                    "city": "Paris"
                }
            },
            {
                "type": "tool_result",
                "toolUseId": "call_abc123",
                "content": [
                    {
                        "type": "text",
                        "text": "Weather in Paris: 18 C"
                    }
                ]
            }
        ])
    );
}

#[test]
fn mcp_sampling_contracts_round_trip_include_context() {
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::assistant(McpSamplingContent::text(
            "Previous answer",
        ))],
        50,
    )
    .with_include_context(McpSamplingIncludeContext::ThisServer)
    .with_stop_sequence("</done>")
    .with_temperature(0.2);

    let value = serde_json::to_value(&request).expect("json");
    assert_eq!(value["includeContext"], "thisServer");
    assert_eq!(value["stopSequences"], json!(["</done>"]));
    assert_eq!(value["temperature"], json!(0.2));

    let decoded: McpCreateMessageRequestParams = serde_json::from_value(value).expect("decoded");
    assert_eq!(decoded, request);
}

#[test]
fn mcp_sampling_contracts_reject_invalid_public_labels() {
    assert!(McpModelHint::new(" ").is_err());
    assert!(McpSamplingContent::image("aW1hZ2U=", " ").is_err());
    assert!(McpSamplingContent::tool_use("call_1", "sk-secret-value", json!({})).is_err());
    assert!(
        McpCreateMessageResult::new(
            McpSamplingRole::Assistant,
            McpSamplingContent::text("ok"),
            "sk-secret-model",
            Some("endTurn"),
        )
        .is_err()
    );
}
