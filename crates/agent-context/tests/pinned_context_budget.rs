use agent_context::{
    ContextBudget, ContextCandidate, ContextCandidateOmissionReason, ContextError,
    ContextHybridCandidate, ContextHybridCandidateOmission, ContextHybridRetrievalReport,
    ContextRetrievalQuery, ContextSelectionReason, ContextSemanticRetrievalStatus,
    ContextSemanticRetrievalSummary, PinnedContextOmissionReason, PinnedContextPriority,
    PinnedContextSource, PinnedProjectContextPin, PinnedProjectContextRequest,
    PinnedProjectContextRetrievalLayer, ProjectContextLimits, ProjectContextSummary,
    ProjectContextVisibility, ProjectFileKind, ProjectFileSummary, PromptLayerKind,
    PromptLayerSensitivity, PromptLayerVisibility, plan_pinned_project_context_budget,
    retrieve_relevant_context,
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
                "README.md",
                ProjectFileKind::Documentation,
                Some("markdown"),
                12,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                "docs/design.md",
                ProjectFileKind::Documentation,
                Some("markdown"),
                60,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                "src/lib.rs",
                ProjectFileKind::Source,
                Some("rust"),
                40,
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

fn budget(max_tokens: u32, reserved_response_tokens: u32) -> ContextBudget {
    ContextBudget {
        max_tokens,
        reserved_response_tokens,
    }
}

#[test]
fn pinned_context_budget_serializes_stable_safe_shape() {
    let request = PinnedProjectContextRequest::new(
        vec![
            PinnedProjectContextPin::required("README.md", "design anchor").expect("pin"),
            PinnedProjectContextPin::optional(".env", "env metadata only").expect("pin"),
        ],
        budget(100, 20),
        false,
    )
    .expect("request");
    let report = plan_pinned_project_context_budget(&summary(), request, None).expect("plan");
    let value = serde_json::to_value(&report).expect("serialize");

    assert_eq!(
        value,
        json!({
            "request": {
                "pins": [
                    {
                        "path": "README.md",
                        "priority": "required",
                        "reason": "design anchor"
                    },
                    {
                        "path": ".env",
                        "priority": "optional",
                        "reason": "env metadata only"
                    }
                ],
                "budget": {
                    "max_tokens": 100,
                    "reserved_response_tokens": 20
                },
                "include_retrieval": false
            },
            "budget": {
                "max_tokens": 100,
                "reserved_response_tokens": 20
            },
            "available_tokens": 80,
            "used_tokens": 13,
            "included": [
                {
                    "path": "README.md",
                    "kind": "documentation",
                    "language": "markdown",
                    "visibility": "summary_only",
                    "token_estimate": 12,
                    "priority": "required",
                    "source": "project_file",
                    "reason": "design anchor"
                },
                {
                    "path": ".env",
                    "kind": "secret_like",
                    "language": null,
                    "visibility": "metadata_only",
                    "token_estimate": 1,
                    "priority": "optional",
                    "source": "project_file",
                    "reason": "env metadata only"
                }
            ],
            "omitted": [],
            "retrieval_included": false,
            "retrieval_omitted_reason": null,
            "diagnostics": []
        })
    );
}

#[test]
fn pinned_context_validates_paths_reasons_duplicates_and_budget() {
    assert!(PinnedProjectContextPin::required(" ", "reason").is_err());
    assert!(PinnedProjectContextPin::optional("README.md", " ").is_err());
    assert!(
        PinnedProjectContextRequest::new(
            vec![
                PinnedProjectContextPin::required("./README.md", "first").expect("pin"),
                PinnedProjectContextPin::optional("README.md", "duplicate").expect("pin"),
            ],
            budget(100, 20),
            false,
        )
        .is_err()
    );
    assert!(
        PinnedProjectContextRequest::new(
            vec![PinnedProjectContextPin::required("README.md", "reason").expect("pin")],
            budget(10, 10),
            false,
        )
        .is_err()
    );
}

#[test]
fn required_pins_fail_when_unknown_or_over_budget() {
    let unknown = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required("missing.rs", "must read").expect("pin")],
        budget(100, 20),
        false,
    )
    .expect("request");
    let error = plan_pinned_project_context_budget(&summary(), unknown, None)
        .expect_err("unknown required pin fails");
    assert!(matches!(
        error,
        ContextError::InvalidPin(message) if message.contains("missing.rs")
    ));

    let overflow = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required("README.md", "must fit").expect("pin")],
        budget(20, 10),
        false,
    )
    .expect("request");
    let error = plan_pinned_project_context_budget(&summary(), overflow, None)
        .expect_err("required overflow fails");
    assert!(matches!(
        error,
        ContextError::BudgetExceeded(message) if message.contains("README.md")
    ));
}

#[test]
fn optional_pins_omit_when_unknown_or_over_budget() {
    let request = PinnedProjectContextRequest::new(
        vec![
            PinnedProjectContextPin::required("README.md", "small required").expect("pin"),
            PinnedProjectContextPin::optional("missing.rs", "best effort missing").expect("pin"),
            PinnedProjectContextPin::optional("docs/design.md", "best effort large").expect("pin"),
        ],
        budget(40, 20),
        false,
    )
    .expect("request");
    let report = plan_pinned_project_context_budget(&summary(), request, None).expect("plan");

    assert_eq!(
        report
            .included
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>(),
        vec!["README.md"]
    );
    assert!(report.omitted.iter().any(|omission| {
        omission.path == "missing.rs"
            && omission.priority == PinnedContextPriority::Optional
            && omission.reason == PinnedContextOmissionReason::UnknownPath
    }));
    assert!(report.omitted.iter().any(|omission| {
        omission.path == "docs/design.md"
            && omission.reason == PinnedContextOmissionReason::BudgetExceeded
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "context_pins.optional.unknown"
            && diagnostic.path.as_deref() == Some("missing.rs")
    }));
}

#[test]
fn retrieval_layers_are_optional_after_pins() {
    let retrieval = retrieve_relevant_context(
        &summary(),
        ContextRetrievalQuery::new("rust source library", 2, 100, Vec::new()).expect("query"),
    )
    .expect("retrieval");
    let retrieval_layer =
        PinnedProjectContextRetrievalLayer::lexical(&retrieval, "project-context-retrieval", 40)
            .expect("retrieval layer");
    let request = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required("README.md", "required first").expect("pin")],
        budget(400, 50),
        true,
    )
    .expect("request");
    let report = plan_pinned_project_context_budget(&summary(), request, Some(retrieval_layer))
        .expect("plan");

    assert_eq!(report.included[0].path, "README.md");
    assert!(report.retrieval_included);
    assert_eq!(report.retrieval_omitted_reason, None);

    let retrieval = retrieve_relevant_context(
        &summary(),
        ContextRetrievalQuery::new("rust source library", 2, 100, Vec::new()).expect("query"),
    )
    .expect("retrieval");
    let retrieval_layer =
        PinnedProjectContextRetrievalLayer::lexical(&retrieval, "project-context-retrieval", 40)
            .expect("retrieval layer");
    let small = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required("README.md", "required first").expect("pin")],
        budget(30, 10),
        true,
    )
    .expect("request");
    let report =
        plan_pinned_project_context_budget(&summary(), small, Some(retrieval_layer)).expect("plan");

    assert!(!report.retrieval_included);
    assert_eq!(
        report.retrieval_omitted_reason.as_deref(),
        Some("budget_exceeded")
    );
}

#[test]
fn hybrid_retrieval_layers_and_prompt_layer_stay_metadata_safe() {
    let candidate = ContextCandidate::new(
        "src/lib.rs",
        ProjectFileKind::Source,
        Some("rust".to_string()),
        120,
        Some(7),
        40,
        ProjectContextVisibility::SummaryOnly,
        900,
        vec![ContextSelectionReason::SemanticMatch],
    )
    .expect("candidate");
    let hybrid = ContextHybridRetrievalReport::new(
        ContextRetrievalQuery::new("secret sk-test-token", 1, 100, Vec::new()).expect("query"),
        ContextSemanticRetrievalSummary {
            status: ContextSemanticRetrievalStatus::Completed,
            provider_id: Some("fixture".to_string()),
            model_id: Some("fixture-embedding".to_string()),
            dimensions: Some(2),
            document_count: 4,
            embedded_document_count: 3,
        },
        vec![ContextHybridCandidate {
            candidate,
            lexical_score: 0.1,
            semantic_score: Some(1.0),
            combined_score: 0.91,
        }],
        vec![ContextHybridCandidateOmission::new(
            ContextHybridCandidate {
                candidate: ContextCandidate::new(
                    "docs/design.md",
                    ProjectFileKind::Documentation,
                    Some("markdown".to_string()),
                    120,
                    Some(7),
                    60,
                    ProjectContextVisibility::SummaryOnly,
                    200,
                    vec![ContextSelectionReason::PathTokenMatch],
                )
                .expect("candidate"),
                lexical_score: 0.2,
                semantic_score: None,
                combined_score: 0.2,
            },
            ContextCandidateOmissionReason::BudgetExceeded,
        )],
        vec![],
        40,
    );
    let retrieval_layer =
        PinnedProjectContextRetrievalLayer::hybrid(&hybrid, "project-context-hybrid", 40)
            .expect("hybrid layer");
    let request = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required(".env", "api_key sk-test-token").expect("pin")],
        budget(400, 50),
        true,
    )
    .expect("request");
    let report = plan_pinned_project_context_budget(&summary(), request, Some(retrieval_layer))
        .expect("plan");
    let layer = report
        .to_prompt_layer("pinned-project-context", 70)
        .expect("prompt layer");

    assert_eq!(report.included[0].source, PinnedContextSource::ProjectFile);
    assert_eq!(
        report.included[0].visibility,
        ProjectContextVisibility::MetadataOnly
    );
    assert!(report.retrieval_included);
    assert_eq!(layer.kind, PromptLayerKind::Project);
    assert_eq!(layer.visibility, PromptLayerVisibility::SummaryOnly);
    assert_eq!(layer.sensitivity, PromptLayerSensitivity::Workspace);
    assert!(layer.content.contains(".env"));
    assert!(layer.content.contains("metadata_only"));
    assert!(!layer.content.contains("api_key"));
    assert!(!layer.content.contains("sk-test-token"));
    assert!(!layer.content.contains("values"));
    assert!(!layer.content.contains("text_hash"));
}
