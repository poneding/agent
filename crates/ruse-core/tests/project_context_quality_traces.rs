use std::fs;
use std::sync::Arc;

use ruse_core::{
    ContextBudget, ContextQualityRetrievalKind, ContextRetrievalQuery,
    ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode,
    PinnedProjectContextPin, PinnedProjectContextRequest, ProjectContextHybridPinnedRetrievalInput,
    ProjectContextHybridRetrievalInput, ProjectContextLimits, ProjectContextPinnedRetrievalInput,
    ProjectContextRetrievalInput, retrieve_project_context_hybrid_pinned_quality,
    retrieve_project_context_hybrid_quality, retrieve_project_context_pinned_quality,
    retrieve_project_context_quality,
};
use ruse_memory::{
    EmbeddingBatch, EmbeddingInput, EmbeddingMetadata, EmbeddingProvider, EmbeddingRequest,
    EmbeddingVector, MemoryError, MemorySensitivity,
};

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 2_048, 20_000).expect("valid limits")
}

fn budget(max_tokens: u32, reserved_response_tokens: u32) -> ContextBudget {
    ContextBudget {
        max_tokens,
        reserved_response_tokens,
    }
}

fn fixture_workspace() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::create_dir_all(temp.path().join("docs")).expect("docs dir");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(
        temp.path().join("src/command.rs"),
        "pub fn command_palette() {}\n",
    )
    .expect("command");
    fs::write(temp.path().join("src/http.rs"), "pub fn request() {}\n").expect("http");
    fs::write(
        temp.path().join("docs/design.md"),
        format!("# Design\n{}\n", "large design note ".repeat(40)),
    )
    .expect("design");
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

#[test]
fn project_context_quality_trace_wraps_lexical_retrieval() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_quality(ProjectContextRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
    })
    .expect("quality retrieval");

    assert_eq!(report.summary.workspace_root_display, ".");
    assert_eq!(
        report.quality.retrieval_kind,
        ContextQualityRetrievalKind::Lexical
    );
    assert_eq!(
        report.quality.selected_count,
        report.retrieval.selected.len() as u32
    );
    assert!(
        report
            .quality
            .signals
            .iter()
            .any(|signal| signal.path.as_deref() == Some("src/command.rs"))
    );
}

#[tokio::test]
async fn project_context_quality_trace_wraps_hybrid_retrieval() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_hybrid_quality(ProjectContextHybridRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        semantic_config: semantic_config(),
        embedding_provider: Some(Arc::new(FixtureProvider)),
    })
    .await
    .expect("hybrid quality retrieval");

    assert_eq!(
        report.quality.retrieval_kind,
        ContextQualityRetrievalKind::Hybrid
    );
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/http.rs");
    assert!(
        serde_json::to_string(&report.quality)
            .expect("serialize")
            .contains("semantic_completed")
    );
}

#[test]
fn project_context_quality_trace_wraps_pinned_retrieval() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_pinned_quality(ProjectContextPinnedRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        pinned_request: PinnedProjectContextRequest::new(
            vec![
                PinnedProjectContextPin::required("README.md", "entry api_key").expect("pin"),
                PinnedProjectContextPin::optional("docs/design.md", "large optional").expect("pin"),
            ],
            budget(40, 10),
            true,
        )
        .expect("pins"),
    })
    .expect("pinned quality retrieval");

    assert_eq!(
        report.quality.retrieval_kind,
        ContextQualityRetrievalKind::PinnedLexical
    );
    assert_eq!(report.quality.pinned_included_count, 1);
    assert_eq!(report.quality.pinned_omitted_count, 1);
    let serialized = serde_json::to_string(&report).expect("serialize");
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains(temp.path().to_str().expect("utf8 path")));
}

#[tokio::test]
async fn project_context_quality_trace_wraps_pinned_hybrid_retrieval() {
    let temp = fixture_workspace();
    let report =
        retrieve_project_context_hybrid_pinned_quality(ProjectContextHybridPinnedRetrievalInput {
            workspace_root: temp.path().to_path_buf(),
            limits: limits(),
            query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new())
                .expect("query"),
            semantic_config: semantic_config(),
            embedding_provider: Some(Arc::new(FixtureProvider)),
            pinned_request: PinnedProjectContextRequest::new(
                vec![PinnedProjectContextPin::required("README.md", "entry point").expect("pin")],
                budget(400, 50),
                true,
            )
            .expect("pins"),
        })
        .await
        .expect("pinned hybrid quality retrieval");

    assert_eq!(
        report.quality.retrieval_kind,
        ContextQualityRetrievalKind::PinnedHybrid
    );
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/http.rs");
    assert!(report.quality.pinned_included_count > 0);
}
