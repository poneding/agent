use agent_context::{
    ContextCandidateOmissionReason, ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery,
    ContextSelectionReason, ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig,
    ContextSemanticRetrievalMode, ContextSemanticRetrievalStatus, ProjectContextLimits,
    ProjectContextSummary, ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
    PromptLayerKind, PromptLayerSensitivity, PromptLayerVisibility,
    build_project_semantic_documents, retrieve_hybrid_context,
};
use agent_memory::{
    EmbeddingBatch, EmbeddingInput, EmbeddingMetadata, EmbeddingProvider, EmbeddingRequest,
    EmbeddingVector, MemoryError, MemorySensitivity,
};
use serde_json::json;

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 16_384, 1_000_000).expect("valid limits")
}

fn file(
    path: &str,
    kind: ProjectFileKind,
    language: Option<&str>,
    token_estimate: u32,
    visibility: ProjectContextVisibility,
) -> ProjectFileSummary {
    ProjectFileSummary::new(
        path,
        kind,
        language.map(ToString::to_string),
        120,
        Some(7),
        token_estimate,
        Some(format!("fnv1a64:{path}")),
        visibility,
    )
    .expect("valid file summary")
}

fn summary() -> ProjectContextSummary {
    ProjectContextSummary::new(
        ".",
        vec![
            file(
                "src/command.rs",
                ProjectFileKind::Source,
                Some("rust"),
                20,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                "src/http.rs",
                ProjectFileKind::Source,
                Some("rust"),
                20,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                ".env",
                ProjectFileKind::SecretLike,
                None,
                1,
                ProjectContextVisibility::MetadataOnly,
            ),
        ],
        vec![],
        vec![],
        vec![],
        limits(),
    )
    .expect("valid summary")
}

#[derive(Clone, Debug)]
struct FixtureProvider;

#[async_trait::async_trait]
impl EmbeddingProvider for FixtureProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingBatch, MemoryError> {
        let vectors = request
            .inputs
            .into_iter()
            .map(|input| {
                let values =
                    if input.record_id == "query" || input.record_id.contains("src/http.rs") {
                        vec![1.0, 0.0]
                    } else {
                        vec![0.0, 1.0]
                    };
                vector(input, values)
            })
            .collect::<Result<Vec<_>, _>>()?;
        EmbeddingBatch::new(vectors)
    }
}

fn vector(input: EmbeddingInput, values: Vec<f32>) -> Result<EmbeddingVector, MemoryError> {
    EmbeddingVector::new(
        input.record_id,
        values,
        EmbeddingMetadata::new(
            "fixture",
            "fixture-embedding",
            2,
            "fixture-hash",
            MemorySensitivity::Internal,
        )?,
    )
}

fn semantic_config() -> ContextSemanticRetrievalConfig {
    ContextSemanticRetrievalConfig::new(
        ContextSemanticRetrievalMode::Provider {
            provider_id: "fixture".to_string(),
            model_id: "fixture-embedding".to_string(),
            dimensions: 2,
        },
        0.1,
        0.9,
        0.0,
        ContextSemanticFallbackPolicy::LexicalOnly,
    )
    .expect("valid semantic config")
}

#[test]
fn semantic_config_and_documents_serialize_safe_shape() {
    let config = semantic_config();
    let config_value = serde_json::to_value(config).expect("serialize config");
    assert_eq!(
        config_value,
        json!({
            "mode": {
                "provider": {
                    "provider_id": "fixture",
                    "model_id": "fixture-embedding",
                    "dimensions": 2
                }
            },
            "lexical_weight": 0.1,
            "semantic_weight": 0.9,
            "minimum_score": 0.0,
            "fallback_policy": "lexical_only"
        })
    );

    let documents = build_project_semantic_documents(&summary()).expect("documents");
    let value = serde_json::to_value(&documents).expect("serialize documents");

    assert_eq!(value[0]["path"], json!(".env"));
    assert_eq!(value[0]["embeddable"], json!(false));
    assert_eq!(value[0]["visibility"], json!("metadata_only"));
    assert_eq!(value[1]["path"], json!("src/command.rs"));
    assert_eq!(value[1]["embeddable"], json!(true));
    let serialized = serde_json::to_string(&documents).expect("serialize");
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("sk-test-secret"));
}

#[test]
fn semantic_config_rejects_invalid_values() {
    assert!(
        ContextSemanticRetrievalConfig::new(
            ContextSemanticRetrievalMode::Provider {
                provider_id: " ".to_string(),
                model_id: "fixture".to_string(),
                dimensions: 2,
            },
            1.0,
            1.0,
            0.0,
            ContextSemanticFallbackPolicy::LexicalOnly,
        )
        .is_err()
    );
    assert!(
        ContextSemanticRetrievalConfig::new(
            ContextSemanticRetrievalMode::Provider {
                provider_id: "fixture".to_string(),
                model_id: "fixture".to_string(),
                dimensions: 0,
            },
            1.0,
            1.0,
            0.0,
            ContextSemanticFallbackPolicy::LexicalOnly,
        )
        .is_err()
    );
    assert!(
        ContextSemanticRetrievalConfig::new(
            ContextSemanticRetrievalMode::Disabled,
            0.0,
            0.0,
            0.0,
            ContextSemanticFallbackPolicy::LexicalOnly,
        )
        .is_err()
    );
    assert!(
        ContextSemanticRetrievalConfig::new(
            ContextSemanticRetrievalMode::Disabled,
            1.0,
            0.0,
            -0.1,
            ContextSemanticFallbackPolicy::LexicalOnly,
        )
        .is_err()
    );
    assert!(
        ContextSemanticRetrievalConfig::new(
            ContextSemanticRetrievalMode::Disabled,
            1.0,
            0.0,
            0.0,
            ContextSemanticFallbackPolicy::FailClosed,
        )
        .is_err()
    );
}

#[tokio::test]
async fn fixture_semantic_scores_can_change_hybrid_ranking() {
    let report = retrieve_hybrid_context(
        &summary(),
        ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config(),
        Some(&FixtureProvider),
    )
    .await
    .expect("hybrid report");

    assert_eq!(
        report.semantic.status,
        ContextSemanticRetrievalStatus::Completed
    );
    assert_eq!(report.selected[0].candidate.path, "src/http.rs");
    assert_eq!(report.selected[0].semantic_score, Some(1.0));
    assert_eq!(report.selected[0].lexical_score, 0.0);
    assert!(
        report.selected[0]
            .candidate
            .reasons
            .contains(&ContextSelectionReason::SemanticMatch)
    );
    assert_eq!(report.selected[1].candidate.path, "src/command.rs");
}

#[tokio::test]
async fn semantic_failure_falls_back_to_lexical_or_fails_closed() {
    let query = ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query");
    let mut fallback_config = semantic_config();
    fallback_config.mode = ContextSemanticRetrievalMode::Provider {
        provider_id: "wrong".to_string(),
        model_id: "fixture-embedding".to_string(),
        dimensions: 2,
    };
    let provider =
        agent_memory::DeterministicEmbeddingProvider::new("fixture", "fixture-embedding", 2)
            .expect("provider");

    let report =
        retrieve_hybrid_context(&summary(), query.clone(), fallback_config, Some(&provider))
            .await
            .expect("fallback report");
    assert_eq!(
        report.semantic.status,
        ContextSemanticRetrievalStatus::FallbackLexical
    );
    assert_eq!(report.selected[0].candidate.path, "src/command.rs");
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == ContextRetrievalDiagnosticSeverity::Warning
            && diagnostic.code == "context_retrieval.semantic.fallback"
    }));

    let fail_closed = ContextSemanticRetrievalConfig::new(
        ContextSemanticRetrievalMode::Provider {
            provider_id: "wrong".to_string(),
            model_id: "fixture-embedding".to_string(),
            dimensions: 2,
        },
        0.1,
        0.9,
        0.0,
        ContextSemanticFallbackPolicy::FailClosed,
    )
    .expect("fail-closed config");
    assert!(
        retrieve_hybrid_context(&summary(), query, fail_closed, Some(&provider))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn disabled_semantic_retrieval_returns_lexical_only_prompt_layer() {
    let report = retrieve_hybrid_context(
        &summary(),
        ContextRetrievalQuery::new("command palette sk-test-secret", 2, 100, Vec::new())
            .expect("query"),
        ContextSemanticRetrievalConfig::disabled(),
        None,
    )
    .await
    .expect("lexical-only report");

    assert_eq!(
        report.semantic.status,
        ContextSemanticRetrievalStatus::Disabled
    );
    assert_eq!(report.selected[0].candidate.path, "src/command.rs");
    assert_eq!(report.selected[0].semantic_score, None);
    let layer = report
        .to_prompt_layer("project-context-hybrid", 60)
        .expect("prompt layer");
    assert_eq!(layer.kind, PromptLayerKind::Project);
    assert_eq!(layer.visibility, PromptLayerVisibility::SummaryOnly);
    assert_eq!(layer.sensitivity, PromptLayerSensitivity::Workspace);
    assert!(layer.content.contains("semantic=disabled"));
    assert!(!layer.content.contains("sk-test-secret"));
    assert!(!layer.content.contains("api_key"));
}

#[tokio::test]
async fn budget_omits_hybrid_candidates_with_stable_reason() {
    let report = retrieve_hybrid_context(
        &summary(),
        ContextRetrievalQuery::new("command palette", 3, 20, Vec::new()).expect("query"),
        semantic_config(),
        Some(&FixtureProvider),
    )
    .await
    .expect("hybrid report");

    assert_eq!(report.used_tokens, 20);
    assert!(report.omitted.iter().any(|omission| {
        omission.reason == ContextCandidateOmissionReason::BudgetExceeded
            && omission.candidate.candidate.path == "src/command.rs"
    }));
}
