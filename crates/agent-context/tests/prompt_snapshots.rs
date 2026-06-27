use agent_context::{
    PromptBudget, PromptLayer, PromptLayerId, PromptLayerKind, PromptLayerPolicy,
    PromptLayerSensitivity, PromptLayerSource, PromptLayerVisibility, PromptProfile,
    PromptProfileId, PromptRuntime, PromptSnapshot,
};

fn layer(id: &str, content: &str) -> PromptLayer {
    PromptLayer::new(
        id,
        PromptLayerKind::System,
        100,
        content,
        5,
        PromptLayerSource::BuiltIn,
        PromptLayerVisibility::Hidden,
        PromptLayerSensitivity::SecretLike,
        PromptLayerPolicy::Required,
        1,
    )
    .expect("valid layer")
}

fn profile() -> PromptProfile {
    PromptProfile::new(
        PromptProfileId::new("default").expect("valid profile id"),
        1,
        "Default Agent",
        vec![PromptLayerId::new("system").expect("valid layer id")],
        PromptBudget::new(4096, 1024).expect("valid budget"),
    )
    .expect("valid profile")
}

#[test]
fn prompt_snapshot_hash_is_stable_for_same_bundle() {
    let request = agent_context::PromptAssemblyRequest {
        profile: profile(),
        layers: vec![layer("system", "SECRET_SYSTEM_PROMPT")],
        overlays: vec![],
    };
    let first = PromptRuntime
        .assemble(request.clone())
        .expect("assemble first");
    let second = PromptRuntime.assemble(request).expect("assemble second");

    let first_snapshot = PromptSnapshot::from_assembly(&first.bundle, &first.report);
    let second_snapshot = PromptSnapshot::from_assembly(&second.bundle, &second.report);

    assert_eq!(first_snapshot.bundle_hash, second_snapshot.bundle_hash);
    assert_eq!(first_snapshot.layer_hashes, second_snapshot.layer_hashes);
    assert!(first_snapshot.bundle_hash.starts_with("fnv1a64:"));
}

#[test]
fn prompt_snapshot_serialization_excludes_raw_content() {
    let outcome = PromptRuntime
        .assemble(agent_context::PromptAssemblyRequest {
            profile: profile(),
            layers: vec![layer("system", "SECRET_SYSTEM_PROMPT")],
            overlays: vec![],
        })
        .expect("assemble prompt");

    let snapshot = PromptSnapshot::from_assembly(&outcome.bundle, &outcome.report);
    let json = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!json.contains("SECRET_SYSTEM_PROMPT"));
    assert!(json.contains("bundle_hash"));
    assert!(json.contains("content_hash"));
}
