use ruse_context::{
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, assemble_prompt_bundle, memory_prompt_layers,
};
use ruse_memory::{
    MemoryRecord, MemoryRetrievalCandidate, MemoryRetrievalReason, MemoryScope,
    MemorySelectedCandidate, MemorySensitivity, MemorySource,
};
use ruse_types::ConversationId;

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000002323"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn selected(
    id: &str,
    text: &str,
    sensitivity: MemorySensitivity,
    rank: usize,
    token_estimate: u32,
) -> MemorySelectedCandidate {
    MemorySelectedCandidate {
        candidate: MemoryRetrievalCandidate {
            record: MemoryRecord {
                id: id.to_string(),
                scope: conversation_scope(),
                text: text.to_string(),
                source: MemorySource {
                    event_id: None,
                    description: "test memory".to_string(),
                },
                confidence: 80,
                sensitivity,
            },
            lexical_score: 0.5,
            semantic_score: Some(0.5),
            combined_score: 1.0,
            matched_terms: vec!["memory".to_string()],
        },
        rank,
        token_estimate,
        reasons: vec![MemoryRetrievalReason::Score],
    }
}

#[test]
fn memory_prompt_layers_convert_selected_memories_into_prompt_layers() {
    let layers = memory_prompt_layers(&[
        selected(
            "public-memory",
            "public project preference",
            MemorySensitivity::Public,
            1,
            3,
        ),
        selected(
            "internal-memory",
            "internal workspace convention",
            MemorySensitivity::Internal,
            2,
            4,
        ),
    ])
    .expect("memory prompt layers");

    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].id.as_str(), "memory:public-memory");
    assert_eq!(layers[0].kind, PromptLayerKind::Memory);
    assert_eq!(layers[0].content, "public project preference");
    assert_eq!(layers[0].token_estimate, 3);
    assert_eq!(
        layers[0].source,
        PromptLayerSource::Memory {
            memory_id: "public-memory".to_string()
        }
    );
    assert_eq!(layers[0].visibility, PromptLayerVisibility::Full);
    assert_eq!(layers[0].sensitivity, PromptLayerSensitivity::Public);
    assert_eq!(layers[0].policy, PromptLayerPolicy::Optional);

    assert_eq!(layers[1].id.as_str(), "memory:internal-memory");
    assert_eq!(layers[1].visibility, PromptLayerVisibility::SummaryOnly);
    assert_eq!(layers[1].sensitivity, PromptLayerSensitivity::Workspace);
}

#[test]
fn memory_prompt_layers_mark_sensitive_memory_hidden_and_private() {
    let layers = memory_prompt_layers(&[selected(
        "sensitive-memory",
        "private user preference with raw detail",
        MemorySensitivity::Sensitive,
        1,
        6,
    )])
    .expect("memory prompt layers");

    assert_eq!(layers[0].content, "private user preference with raw detail");
    assert_eq!(layers[0].visibility, PromptLayerVisibility::Hidden);
    assert_eq!(layers[0].sensitivity, PromptLayerSensitivity::UserPrivate);
    assert_eq!(
        layers[0].source,
        PromptLayerSource::Memory {
            memory_id: "sensitive-memory".to_string()
        }
    );
}

#[test]
fn memory_prompt_layers_reject_empty_selected_memory_content() {
    let error = memory_prompt_layers(&[selected(
        "empty-memory",
        " ",
        MemorySensitivity::Public,
        1,
        1,
    )])
    .expect_err("empty memory content rejected");

    assert!(error.to_string().contains("content"));
}

#[test]
fn memory_prompt_layers_assemble_with_user_layers_in_priority_order() {
    let mut layers = memory_prompt_layers(&[selected(
        "project-style",
        "Prefer focused memory review tests.",
        MemorySensitivity::Public,
        1,
        5,
    )])
    .expect("memory prompt layers");
    layers.push(
        PromptLayer::new(
            "user",
            PromptLayerKind::User,
            0,
            "Implement the next memory slice.",
            6,
            PromptLayerSource::BuiltIn,
            PromptLayerVisibility::Full,
            PromptLayerSensitivity::Public,
            PromptLayerPolicy::Required,
            1,
        )
        .expect("user layer"),
    );

    let bundle = assemble_prompt_bundle(layers).expect("prompt bundle");
    let messages = bundle
        .messages
        .iter()
        .map(|message| (message.role.as_str(), message.content.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            ("system", "Prefer focused memory review tests."),
            ("user", "Implement the next memory slice.")
        ]
    );
    assert_eq!(bundle.estimated_tokens, 11);
}
