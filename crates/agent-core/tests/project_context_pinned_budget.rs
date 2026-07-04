use std::fs;
use std::sync::Arc;

use agent_core::{
    ContextBudget, ContextRetrievalQuery, ContextSemanticFallbackPolicy,
    ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode, ContextSemanticRetrievalStatus,
    PinnedContextOmissionReason, PinnedProjectContextPin, PinnedProjectContextRequest,
    ProjectContextHybridPinnedRetrievalInput, ProjectContextLimits,
    ProjectContextPinnedRetrievalInput, ProjectContextRetrievalError,
    retrieve_project_context_hybrid_pinned, retrieve_project_context_pinned,
};
use agent_memory::{
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
fn project_context_pinned_retrieval_scans_retrieves_and_plans_budget() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_pinned(ProjectContextPinnedRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        pinned_request: PinnedProjectContextRequest::new(
            vec![
                PinnedProjectContextPin::required("README.md", "entry point").expect("pin"),
                PinnedProjectContextPin::optional("src/command.rs", "implementation").expect("pin"),
            ],
            budget(400, 50),
            true,
        )
        .expect("pins"),
    })
    .expect("pinned retrieval");

    assert_eq!(report.summary.workspace_root_display, ".");
    assert_eq!(report.pinned.included[0].path, "README.md");
    assert_eq!(report.pinned.included[1].path, "src/command.rs");
    assert!(report.pinned.retrieval_included);
    assert_eq!(report.retrieval.selected[0].path, "src/command.rs");
    assert!(
        !serde_json::to_string(&report)
            .expect("serialize")
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn project_context_pinned_retrieval_fails_unknown_required_pin() {
    let temp = fixture_workspace();
    let error = retrieve_project_context_pinned(ProjectContextPinnedRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
        pinned_request: PinnedProjectContextRequest::new(
            vec![PinnedProjectContextPin::required("missing.rs", "must include").expect("pin")],
            budget(400, 50),
            true,
        )
        .expect("pins"),
    })
    .expect_err("unknown required pin");

    assert!(matches!(
        error,
        ProjectContextRetrievalError::Retrieval { .. }
    ));
    assert_eq!(error.code(), "project_context.retrieval.failed");
}

#[test]
fn project_context_pinned_retrieval_omits_optional_pins_safely() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_pinned(ProjectContextPinnedRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("readme docs", 2, 100, Vec::new()).expect("query"),
        pinned_request: PinnedProjectContextRequest::new(
            vec![
                PinnedProjectContextPin::required("README.md", "small required").expect("pin"),
                PinnedProjectContextPin::optional("missing.rs", "missing optional").expect("pin"),
                PinnedProjectContextPin::optional("docs/design.md", "large optional").expect("pin"),
            ],
            budget(30, 10),
            false,
        )
        .expect("pins"),
    })
    .expect("pinned retrieval");

    assert_eq!(report.pinned.included[0].path, "README.md");
    assert!(report.pinned.omitted.iter().any(|omission| {
        omission.path == "missing.rs" && omission.reason == PinnedContextOmissionReason::UnknownPath
    }));
    assert!(report.pinned.omitted.iter().any(|omission| {
        omission.path == "docs/design.md"
            && omission.reason == PinnedContextOmissionReason::BudgetExceeded
    }));
    let serialized = serde_json::to_string(&report).expect("serialize");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}

#[tokio::test]
async fn project_context_hybrid_pinned_retrieval_preserves_semantic_evidence() {
    let temp = fixture_workspace();
    let report = retrieve_project_context_hybrid_pinned(ProjectContextHybridPinnedRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("command palette", 2, 100, Vec::new()).expect("query"),
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
    .expect("hybrid pinned retrieval");

    assert_eq!(
        report.hybrid.semantic.status,
        ContextSemanticRetrievalStatus::Completed
    );
    assert_eq!(report.pinned.included[0].path, "README.md");
    assert!(report.pinned.retrieval_included);
    assert_eq!(report.hybrid.selected[0].candidate.path, "src/http.rs");
}
