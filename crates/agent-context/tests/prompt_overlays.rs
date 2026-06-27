use agent_context::{
    ContextError, PromptAssemblyRequest, PromptBudget, PromptLayer, PromptLayerId, PromptLayerKind,
    PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource, PromptLayerVisibility,
    PromptOverlay, PromptOverlayAction, PromptProfile, PromptProfileId, PromptRuntime,
};

fn budget(max_tokens: u32) -> PromptBudget {
    PromptBudget::new(max_tokens, 20).expect("valid prompt budget")
}

fn profile(required: &[&str], max_tokens: u32) -> PromptProfile {
    PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        1,
        "Default Agent",
        required
            .iter()
            .map(|id| PromptLayerId::new(*id).expect("valid required id"))
            .collect(),
        budget(max_tokens),
    )
    .expect("valid profile")
}

fn layer(
    id: &str,
    priority: i32,
    content: &str,
    tokens: u32,
    visibility: PromptLayerVisibility,
    policy: PromptLayerPolicy,
) -> PromptLayer {
    PromptLayer::new(
        id,
        PromptLayerKind::System,
        priority,
        content,
        tokens,
        PromptLayerSource::BuiltIn,
        visibility,
        PromptLayerSensitivity::Public,
        policy,
        1,
    )
    .expect("valid prompt layer")
}

fn request(
    profile: PromptProfile,
    layers: Vec<PromptLayer>,
    overlays: Vec<PromptOverlay>,
) -> PromptAssemblyRequest {
    PromptAssemblyRequest {
        profile,
        layers,
        overlays,
    }
}

#[test]
fn prompt_runtime_requires_profile_layers() {
    let error = PromptRuntime
        .assemble(request(profile(&["system"], 200), vec![], vec![]))
        .expect_err("missing required layer rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("missing required layer")
    ));
}

#[test]
fn prompt_runtime_appends_overlay_layers_by_priority() {
    let overlay_layer = layer(
        "session",
        90,
        "session rules",
        5,
        PromptLayerVisibility::Full,
        PromptLayerPolicy::Optional,
    );
    let outcome = PromptRuntime
        .assemble(request(
            profile(&["system"], 200),
            vec![layer(
                "system",
                100,
                "system rules",
                10,
                PromptLayerVisibility::Hidden,
                PromptLayerPolicy::Required,
            )],
            vec![PromptOverlay {
                id: "session-overlay".to_string(),
                priority: 50,
                action: PromptOverlayAction::Append,
                layer: Some(overlay_layer),
            }],
        ))
        .expect("assemble prompt");

    let layer_ids: Vec<_> = outcome
        .bundle
        .layers
        .iter()
        .map(|layer| layer.id.as_str())
        .collect();
    assert_eq!(layer_ids, vec!["system", "session"]);
    assert_eq!(
        outcome.report.included_layer_ids,
        vec![
            PromptLayerId::new("system").expect("valid id"),
            PromptLayerId::new("session").expect("valid id")
        ]
    );
}

#[test]
fn prompt_runtime_replaces_optional_layer_by_id() {
    let outcome = PromptRuntime
        .assemble(request(
            profile(&["system"], 200),
            vec![
                layer(
                    "system",
                    100,
                    "system rules",
                    10,
                    PromptLayerVisibility::Hidden,
                    PromptLayerPolicy::Required,
                ),
                layer(
                    "project",
                    40,
                    "old project rules",
                    5,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
            ],
            vec![PromptOverlay {
                id: "replace-project".to_string(),
                priority: 70,
                action: PromptOverlayAction::Replace {
                    target_layer_id: PromptLayerId::new("project").expect("valid id"),
                },
                layer: Some(layer(
                    "project",
                    80,
                    "new project rules",
                    6,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                )),
            }],
        ))
        .expect("assemble prompt");

    let project = outcome
        .bundle
        .layers
        .iter()
        .find(|layer| layer.id.as_str() == "project")
        .expect("project layer exists");

    assert_eq!(project.content, "new project rules");
    assert_eq!(project.priority, 80);
}

#[test]
fn prompt_runtime_rejects_required_hidden_layer_disable() {
    let error = PromptRuntime
        .assemble(request(
            profile(&["system"], 200),
            vec![layer(
                "system",
                100,
                "system rules",
                10,
                PromptLayerVisibility::Hidden,
                PromptLayerPolicy::Required,
            )],
            vec![PromptOverlay {
                id: "disable-system".to_string(),
                priority: 100,
                action: PromptOverlayAction::Disable {
                    target_layer_id: PromptLayerId::new("system").expect("valid id"),
                },
                layer: None,
            }],
        ))
        .expect_err("required hidden layer disable rejected");

    assert!(matches!(
        error,
        ContextError::InvalidLayer(message) if message.contains("cannot disable required hidden layer")
    ));
}

#[test]
fn prompt_runtime_records_duplicate_optional_layer_diagnostic() {
    let outcome = PromptRuntime
        .assemble(request(
            profile(&["system"], 200),
            vec![
                layer(
                    "system",
                    100,
                    "system rules",
                    10,
                    PromptLayerVisibility::Hidden,
                    PromptLayerPolicy::Required,
                ),
                layer(
                    "project",
                    50,
                    "higher priority project rules",
                    5,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
                layer(
                    "project",
                    10,
                    "lower priority project rules",
                    5,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
            ],
            vec![],
        ))
        .expect("assemble prompt");

    assert_eq!(
        outcome
            .bundle
            .layers
            .iter()
            .filter(|layer| layer.id.as_str() == "project")
            .count(),
        1
    );
    assert!(
        outcome
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "duplicate_optional_layer")
    );
}

#[test]
fn prompt_runtime_omits_optional_layers_when_budget_is_exceeded() {
    let outcome = PromptRuntime
        .assemble(request(
            profile(&["system"], 35),
            vec![
                layer(
                    "system",
                    100,
                    "system rules",
                    10,
                    PromptLayerVisibility::Hidden,
                    PromptLayerPolicy::Required,
                ),
                layer(
                    "project",
                    50,
                    "project rules",
                    10,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
                layer(
                    "memory",
                    40,
                    "memory rules",
                    10,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
            ],
            vec![],
        ))
        .expect("assemble prompt");

    let included: Vec<_> = outcome
        .report
        .included_layer_ids
        .iter()
        .map(PromptLayerId::as_str)
        .collect();
    let omitted: Vec<_> = outcome
        .report
        .omitted_layer_ids
        .iter()
        .map(PromptLayerId::as_str)
        .collect();

    assert_eq!(included, vec!["system"]);
    assert_eq!(omitted, vec!["project", "memory"]);
    assert!(
        outcome
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "optional_layer_omitted")
    );
}

#[test]
fn prompt_diagnostics_do_not_include_layer_content() {
    let outcome = PromptRuntime
        .assemble(request(
            profile(&["system"], 200),
            vec![
                layer(
                    "system",
                    100,
                    "system rules",
                    10,
                    PromptLayerVisibility::Hidden,
                    PromptLayerPolicy::Required,
                ),
                layer(
                    "project",
                    50,
                    "SECRET_PROJECT_CONTENT",
                    5,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
                layer(
                    "project",
                    10,
                    "SECRET_DUPLICATE_CONTENT",
                    5,
                    PromptLayerVisibility::Full,
                    PromptLayerPolicy::Optional,
                ),
            ],
            vec![],
        ))
        .expect("assemble prompt");

    let diagnostic_text = outcome
        .report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!diagnostic_text.contains("SECRET_PROJECT_CONTENT"));
    assert!(!diagnostic_text.contains("SECRET_DUPLICATE_CONTENT"));
}
