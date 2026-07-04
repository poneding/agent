use agent_context::{
    ContextError, PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity,
    PromptLayerSource, PromptLayerVisibility, assemble_prompt_bundle,
};
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
    PromptLayer::new(
        id,
        kind,
        priority,
        content,
        tokens,
        PromptLayerSource::BuiltIn,
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Public,
        PromptLayerPolicy::Optional,
        1,
    )
    .expect("valid prompt layer")
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
    let error = PromptLayer::new(
        " ",
        PromptLayerKind::System,
        1,
        "system rules",
        1,
        PromptLayerSource::BuiltIn,
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Public,
        PromptLayerPolicy::Optional,
        1,
    )
    .expect_err("empty id rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("id")
    ));
}

#[test]
fn prompt_layers_reject_empty_content() {
    let error = PromptLayer::new(
        "system",
        PromptLayerKind::System,
        1,
        " ",
        1,
        PromptLayerSource::BuiltIn,
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Public,
        PromptLayerPolicy::Optional,
        1,
    )
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
            "token_estimate": 5,
            "source": {
                "type": "built_in"
            },
            "visibility": "full",
            "sensitivity": "public",
            "policy": "optional",
            "version": 1
        })
    );
}

#[test]
fn prompt_layers_serialize_extended_metadata_shape() {
    let layer = PromptLayer::new(
        "project",
        PromptLayerKind::Project,
        20,
        "project rules",
        8,
        PromptLayerSource::ProjectFile {
            path: "AGENTS.md".to_string(),
            content_hash: "sha256:project".to_string(),
        },
        PromptLayerVisibility::SummaryOnly,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Required,
        3,
    )
    .expect("valid prompt layer");

    let value = serde_json::to_value(layer).expect("serialize prompt layer");

    assert_eq!(
        value,
        json!({
            "id": "project",
            "kind": "project",
            "priority": 20,
            "content": "project rules",
            "token_estimate": 8,
            "source": {
                "type": "project_file",
                "data": {
                    "path": "AGENTS.md",
                    "content_hash": "sha256:project"
                }
            },
            "visibility": "summary_only",
            "sensitivity": "workspace",
            "policy": "required",
            "version": 3
        })
    );
}

#[test]
fn prompt_layer_source_uses_stable_snake_case_shape() {
    let value = serde_json::to_value(PromptLayerSource::RuntimeConfig {
        source: "user-config".to_string(),
    })
    .expect("serialize source");

    assert_eq!(
        value,
        json!({
            "type": "runtime_config",
            "data": {
                "source": "user-config"
            }
        })
    );
}

#[test]
fn prompt_layers_reject_empty_metadata_fields() {
    let error = PromptLayer::new(
        "project",
        PromptLayerKind::Project,
        20,
        "project rules",
        8,
        PromptLayerSource::ProjectFile {
            path: " ".to_string(),
            content_hash: "sha256:project".to_string(),
        },
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Required,
        1,
    )
    .expect_err("empty metadata rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("project file path")
    ));
}
