use agent_context::{ContextError, PromptLayer, PromptLayerKind, assemble_prompt_bundle};
use serde_json::json;

#[test]
fn prompt_layers_expose_public_kind_contract() {
    assert_eq!(
        serde_json::to_string(&PromptLayerKind::System).unwrap(),
        "\"system\""
    );
}

fn layer(
    id: &str,
    kind: PromptLayerKind,
    priority: i32,
    content: &str,
    tokens: u32,
) -> PromptLayer {
    PromptLayer {
        id: id.to_string(),
        kind,
        priority,
        content: content.to_string(),
        token_estimate: tokens,
    }
}

#[test]
fn prompt_layers_sort_by_priority_and_preserve_ties() {
    let bundle = assemble_prompt_bundle(vec![
        layer("project", PromptLayerKind::Project, 10, "project rules", 3),
        layer("system", PromptLayerKind::System, 100, "system rules", 5),
        layer("runtime", PromptLayerKind::Runtime, 10, "runtime state", 7),
        layer("user", PromptLayerKind::User, 0, "user asks", 2),
    ])
    .expect("assemble bundle");

    let layer_ids: Vec<_> = bundle
        .layers
        .iter()
        .map(|layer| layer.id.as_str())
        .collect();
    let messages: Vec<_> = bundle
        .messages
        .iter()
        .map(|message| (message.role.as_str(), message.content.as_str()))
        .collect();

    assert_eq!(layer_ids, vec!["system", "project", "runtime", "user"]);
    assert_eq!(
        messages,
        vec![
            ("system", "system rules"),
            ("system", "project rules"),
            ("system", "runtime state"),
            ("user", "user asks")
        ]
    );
    assert_eq!(bundle.estimated_tokens, 17);
}

#[test]
fn prompt_layers_reject_empty_id() {
    let error = assemble_prompt_bundle(vec![layer(
        " ",
        PromptLayerKind::System,
        1,
        "system rules",
        1,
    )])
    .expect_err("empty id rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("id")
    ));
}

#[test]
fn prompt_layers_reject_empty_content() {
    let error = assemble_prompt_bundle(vec![layer("system", PromptLayerKind::System, 1, " ", 1)])
        .expect_err("empty content rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("content")
    ));
}

#[test]
fn prompt_layers_serialize_stable_shape() {
    let value = serde_json::to_value(layer(
        "system",
        PromptLayerKind::System,
        100,
        "system rules",
        5,
    ))
    .expect("serialize prompt layer");

    assert_eq!(
        value,
        json!({
            "id": "system",
            "kind": "system",
            "priority": 100,
            "content": "system rules",
            "token_estimate": 5
        })
    );
}
