use ruse_context::{
    PromptLayer, PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, assemble_prompt_bundle,
};
use ruse_memory::{
    InMemoryMemoryStore, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource, MemoryStore,
};
use ruse_types::WorkspaceId;

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
        match kind {
            PromptLayerKind::Memory => PromptLayerSource::Memory {
                memory_id: id.to_string(),
            },
            _ => PromptLayerSource::BuiltIn,
        },
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::Workspace,
        PromptLayerPolicy::Optional,
        1,
    )
    .expect("valid prompt layer")
}

#[test]
fn m05_context_memory_project_memory_feeds_prompt_bundle() {
    let workspace_id = WorkspaceId::new();
    let scope = MemoryScope::Project { workspace_id };
    let mut store = InMemoryMemoryStore::default();

    store
        .add(MemoryRecord {
            id: "project-architecture".to_string(),
            scope: scope.clone(),
            text: "Workspace uses event-sourced runtime state.".to_string(),
            source: MemorySource {
                event_id: None,
                description: "project note".to_string(),
            },
            confidence: 90,
            sensitivity: MemorySensitivity::Internal,
        })
        .expect("add project memory");

    let memory_record = store
        .list(&scope)
        .expect("list project memory")
        .into_iter()
        .next()
        .expect("project memory exists");

    let bundle = assemble_prompt_bundle(vec![
        layer(
            "user",
            PromptLayerKind::User,
            0,
            "Explain the runtime shape.",
            5,
        ),
        layer(
            &format!("memory:{}", memory_record.id),
            PromptLayerKind::Memory,
            20,
            &memory_record.text,
            7,
        ),
    ])
    .expect("assemble prompt bundle");

    let messages: Vec<_> = bundle
        .messages
        .iter()
        .map(|message| (message.role.as_str(), message.content.as_str()))
        .collect();

    assert_eq!(
        messages,
        vec![
            ("system", "Workspace uses event-sourced runtime state."),
            ("user", "Explain the runtime shape.")
        ]
    );
    assert_eq!(bundle.estimated_tokens, 12);
}
