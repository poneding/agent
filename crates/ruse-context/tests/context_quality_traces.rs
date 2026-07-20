use ruse_context::{
    ContextBudget, ContextCandidate, ContextCandidateOmissionReason, ContextHybridCandidate,
    ContextHybridCandidateOmission, ContextHybridRetrievalReport, ContextQualityBudgetPressure,
    ContextQualityRetrievalKind, ContextQualitySignalKind, ContextRetrievalQuery,
    ContextSelectionReason, ContextSemanticRetrievalStatus, ContextSemanticRetrievalSummary,
    PinnedContextOmissionReason, PinnedContextPriority, PinnedProjectContextPin,
    PinnedProjectContextRequest, PinnedProjectContextRetrievalLayer, ProjectContextLimits,
    ProjectContextSummary, ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
    build_context_quality_trace_from_hybrid, build_context_quality_trace_from_pinned_hybrid,
    build_context_quality_trace_from_pinned_retrieval, build_context_quality_trace_from_retrieval,
    plan_pinned_project_context_budget, retrieve_relevant_context,
};
use serde_json::json;

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 16_384, 1_000_000).expect("valid limits")
}

fn budget(max_tokens: u32, reserved_response_tokens: u32) -> ContextBudget {
    ContextBudget {
        max_tokens,
        reserved_response_tokens,
    }
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
                "src/command.rs",
                ProjectFileKind::Source,
                Some("rust"),
                40,
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

fn lexical_report() -> ruse_context::ContextRetrievalReport {
    retrieve_relevant_context(
        &summary(),
        ContextRetrievalQuery::new(
            "command sk-test-secret context",
            3,
            120,
            vec!["missing.rs".to_string()],
        )
        .expect("query"),
    )
    .expect("retrieval")
}

fn hybrid_report() -> ContextHybridRetrievalReport {
    let selected = ContextCandidate::new(
        "src/command.rs",
        ProjectFileKind::Source,
        Some("rust".to_string()),
        120,
        Some(7),
        40,
        ProjectContextVisibility::SummaryOnly,
        700,
        vec![ContextSelectionReason::SemanticMatch],
    )
    .expect("candidate");
    let omitted = ContextCandidate::new(
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
    .expect("candidate");

    ContextHybridRetrievalReport::new(
        ContextRetrievalQuery::new("command sk-test-secret context", 1, 50, Vec::new())
            .expect("query"),
        ContextSemanticRetrievalSummary {
            status: ContextSemanticRetrievalStatus::Completed,
            provider_id: Some("fixture".to_string()),
            model_id: Some("fixture-embedding".to_string()),
            dimensions: Some(2),
            document_count: 4,
            embedded_document_count: 3,
        },
        vec![ContextHybridCandidate {
            candidate: selected,
            lexical_score: 0.25,
            semantic_score: Some(1.0),
            combined_score: 0.85,
        }],
        vec![ContextHybridCandidateOmission::new(
            ContextHybridCandidate {
                candidate: omitted,
                lexical_score: 0.2,
                semantic_score: None,
                combined_score: 0.2,
            },
            ContextCandidateOmissionReason::BudgetExceeded,
        )],
        vec![],
        40,
    )
}

#[test]
fn quality_trace_serializes_stable_safe_lexical_shape() {
    let trace =
        build_context_quality_trace_from_retrieval(".", &lexical_report()).expect("quality trace");
    let value = serde_json::to_value(&trace).expect("serialize trace");

    assert_eq!(trace.retrieval_kind, ContextQualityRetrievalKind::Lexical);
    assert_eq!(trace.budget_pressure, ContextQualityBudgetPressure::Low);
    assert_eq!(
        value,
        json!({
            "task": "command [redacted] context",
            "workspace_root_display": ".",
            "retrieval_kind": "lexical",
            "selected_count": 2,
            "omitted_count": 0,
            "pinned_included_count": 0,
            "pinned_omitted_count": 0,
            "used_tokens": 41,
            "available_tokens": 120,
            "budget_pressure": "low",
            "signals": [
                {
                    "path": ".env",
                    "kind": "selected_candidate",
                    "weight": 5,
                    "message": "selected candidate score=5 tokens=1"
                },
                {
                    "path": "src/command.rs",
                    "kind": "selected_candidate",
                    "weight": 30,
                    "message": "selected candidate score=30 tokens=40"
                }
            ],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "context_retrieval.pin.unknown",
                    "message": "pinned path was not found in project summary",
                    "path": "missing.rs"
                }
            ]
        })
    );
    assert!(
        !serde_json::to_string(&trace)
            .expect("serialize")
            .contains("sk-test-secret")
    );
}

#[test]
fn quality_trace_records_hybrid_semantic_and_omission_signals() {
    let trace =
        build_context_quality_trace_from_hybrid(".", &hybrid_report()).expect("quality trace");

    assert_eq!(trace.retrieval_kind, ContextQualityRetrievalKind::Hybrid);
    assert_eq!(trace.selected_count, 1);
    assert_eq!(trace.omitted_count, 1);
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::SemanticCompleted && signal.path.is_none()
    }));
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::OmittedCandidate
            && signal.path.as_deref() == Some("docs/design.md")
    }));
    let serialized = serde_json::to_string(&trace).expect("serialize");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("embedding"));
    assert!(!serialized.contains("values"));
}

#[test]
fn quality_trace_records_pinned_budget_evidence() {
    let retrieval = lexical_report();
    let retrieval_layer =
        PinnedProjectContextRetrievalLayer::lexical(&retrieval, "project-context-retrieval", 40)
            .expect("retrieval layer");
    let request = PinnedProjectContextRequest::new(
        vec![
            PinnedProjectContextPin::required("README.md", "entry api_key").expect("pin"),
            PinnedProjectContextPin::optional("missing.rs", "missing optional").expect("pin"),
            PinnedProjectContextPin::optional("docs/design.md", "large optional").expect("pin"),
        ],
        budget(70, 20),
        true,
    )
    .expect("request");
    let pinned = plan_pinned_project_context_budget(&summary(), request, Some(retrieval_layer))
        .expect("pinned budget");

    assert!(pinned.omitted.iter().any(|omission| {
        omission.path == "missing.rs"
            && omission.priority == PinnedContextPriority::Optional
            && omission.reason == PinnedContextOmissionReason::UnknownPath
    }));
    let trace = build_context_quality_trace_from_pinned_retrieval(".", &retrieval, &pinned)
        .expect("quality trace");

    assert_eq!(
        trace.retrieval_kind,
        ContextQualityRetrievalKind::PinnedLexical
    );
    assert_eq!(trace.pinned_included_count, 1);
    assert_eq!(trace.pinned_omitted_count, 2);
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::IncludedRequiredPin
            && signal.path.as_deref() == Some("README.md")
    }));
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::OmittedOptionalPin
            && signal.path.as_deref() == Some("docs/design.md")
    }));
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::RetrievalLayerOmitted && signal.path.is_none()
    }));
    let serialized = serde_json::to_string(&trace).expect("serialize");
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("sk-test-secret"));
}

#[test]
fn quality_trace_records_pinned_hybrid_kind() {
    let hybrid = hybrid_report();
    let retrieval_layer =
        PinnedProjectContextRetrievalLayer::hybrid(&hybrid, "project-context-hybrid", 40)
            .expect("hybrid layer");
    let request = PinnedProjectContextRequest::new(
        vec![PinnedProjectContextPin::required("README.md", "entry").expect("pin")],
        budget(400, 50),
        true,
    )
    .expect("request");
    let pinned = plan_pinned_project_context_budget(&summary(), request, Some(retrieval_layer))
        .expect("pinned budget");
    let trace = build_context_quality_trace_from_pinned_hybrid(".", &hybrid, &pinned)
        .expect("quality trace");

    assert_eq!(
        trace.retrieval_kind,
        ContextQualityRetrievalKind::PinnedHybrid
    );
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::SemanticCompleted && signal.path.is_none()
    }));
    assert!(trace.signals.iter().any(|signal| {
        signal.kind == ContextQualitySignalKind::RetrievalLayerIncluded && signal.path.is_none()
    }));
}
