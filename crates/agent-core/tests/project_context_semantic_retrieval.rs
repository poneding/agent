use std::fs;
use std::sync::Arc;

use agent_core::{
    ContextRetrievalQuery, ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig,
    ContextSemanticRetrievalMode, ContextSemanticRetrievalStatus,
    ProjectContextHybridRetrievalInput, ProjectContextLimits, ProjectContextRetrievalError,
    ProjectContextVisibility, ProjectFileKind, retrieve_project_context_hybrid,
};
use agent_memory::{
    EmbeddingBatch, EmbeddingInput, EmbeddingMetadata, EmbeddingProvider, EmbeddingRequest,
    EmbeddingVector, MemoryError, MemorySensitivity,
};

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 2_048, 20_000).expect("valid limits")
}

fn fixture_workspace() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::write(
        temp.path().join("src/command.rs"),
        "pub fn command_palette() {}\n",
    )
    .expect("command");
    fs::write(temp.path().join("src/http.rs"), "pub fn request() {}\n").expect("http");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    temp
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

#[tokio::test]
async fn project_context_hybrid_retrieval_scans_and_applies_semantic_ranking() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_hybrid(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config: semantic_config(),
        embedding_provider: Some(Arc::new(FixtureProvider)),
    })
    .await
    .expect("hybrid retrieval");

    assert_eq!(report.summary.workspace_root_display, ".");
    assert_eq!(
        report.hybrid.semantic.status,
        ContextSemanticRetrievalStatus::Completed
    );
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/http.rs");
    assert_eq!(report.hybrid.selected[0].semantic_score, Some(1.0));
    assert!(
        !serde_json::to_string(&report)
            .expect("serialize")
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[tokio::test]
async fn project_context_hybrid_disabled_mode_preserves_lexical_default() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_hybrid(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config: ContextSemanticRetrievalConfig::disabled(),
        embedding_provider: None,
    })
    .await
    .expect("hybrid retrieval");

    assert_eq!(
        report.hybrid.semantic.status,
        ContextSemanticRetrievalStatus::Disabled
    );
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/command.rs");
    assert_eq!(report.hybrid.selected[0].semantic_score, None);
}

#[tokio::test]
async fn project_context_hybrid_retrieval_falls_back_or_fails_closed() {
    let temp = fixture_workspace();
    let fallback = ContextSemanticRetrievalConfig::new(
        ContextSemanticRetrievalMode::Provider {
            provider_id: "wrong".to_string(),
            model_id: "fixture-embedding".to_string(),
            dimensions: 2,
        },
        0.1,
        0.9,
        0.0,
        ContextSemanticFallbackPolicy::LexicalOnly,
    )
    .expect("fallback config");
    let provider = Arc::new(
        agent_memory::DeterministicEmbeddingProvider::new("fixture", "fixture-embedding", 2)
            .expect("provider"),
    );

    let report = retrieve_project_context_hybrid(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config: fallback,
        embedding_provider: Some(provider.clone()),
    })
    .await
    .expect("fallback report");
    assert_eq!(
        report.hybrid.semantic.status,
        ContextSemanticRetrievalStatus::FallbackLexical
    );
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/command.rs");

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
    let error = retrieve_project_context_hybrid(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config: fail_closed,
        embedding_provider: Some(provider),
    })
    .await
    .expect_err("fail closed");

    assert!(matches!(
        error,
        ProjectContextRetrievalError::Retrieval { .. }
    ));
    assert_eq!(error.code(), "project_context.retrieval.failed");
}

#[tokio::test]
async fn project_context_hybrid_retrieval_keeps_secret_like_metadata_safe() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_hybrid(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("env secret metadata", 3, 100, Vec::new())
            .expect("query"),
        semantic_config: semantic_config(),
        embedding_provider: Some(Arc::new(FixtureProvider)),
    })
    .await
    .expect("hybrid retrieval");

    let env = report
        .hybrid
        .selected
        .iter()
        .find(|candidate| candidate.candidate.path == ".env")
        .expect("env metadata selected lexically");
    assert_eq!(env.candidate.kind, ProjectFileKind::SecretLike);
    assert_eq!(
        env.candidate.visibility,
        ProjectContextVisibility::MetadataOnly
    );

    let serialized = serde_json::to_string(&report).expect("serialize");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}
