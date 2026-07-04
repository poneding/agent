use agent_core::{
    ContextCandidate, ContextCandidateOmissionReason, ContextHybridCandidate,
    ContextHybridCandidateOmission, ContextHybridRetrievalReport, ContextRetrievalDiagnostic,
    ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery, ContextSelectionReason,
    ContextSemanticRetrievalStatus, ContextSemanticRetrievalSummary,
    ProjectContextHybridRetrievalReport as CoreHybridRetrievalReport, ProjectContextLimits,
    ProjectContextSummary, ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
};
use agent_ui_bridge::{UiJsonFrame, UiProjectContextHybridRetrievalSnapshot, encode_json_frame};
use serde_json::json;

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 16_384, 1_000_000).expect("valid limits")
}

fn summary() -> ProjectContextSummary {
    ProjectContextSummary::new(
        ".",
        vec![
            ProjectFileSummary::new(
                "src/lib.rs",
                ProjectFileKind::Source,
                Some("rust".to_string()),
                120,
                Some(7),
                12,
                Some("fnv1a64:lib".to_string()),
                ProjectContextVisibility::SummaryOnly,
            )
            .expect("valid source"),
            ProjectFileSummary::new(
                ".env",
                ProjectFileKind::SecretLike,
                None,
                32,
                None,
                1,
                None,
                ProjectContextVisibility::MetadataOnly,
            )
            .expect("valid secret metadata"),
        ],
        vec![],
        vec![],
        vec![],
        limits(),
    )
    .expect("valid summary")
}

fn candidate(
    path: &str,
    kind: ProjectFileKind,
    visibility: ProjectContextVisibility,
    score: u32,
    reasons: Vec<ContextSelectionReason>,
) -> ContextCandidate {
    ContextCandidate::new(
        path,
        kind,
        if kind == ProjectFileKind::Source {
            Some("rust".to_string())
        } else {
            None
        },
        120,
        Some(7),
        1,
        visibility,
        score,
        reasons,
    )
    .expect("valid candidate")
}

fn report() -> CoreHybridRetrievalReport {
    let summary = summary();
    let hybrid = ContextHybridRetrievalReport::new(
        ContextRetrievalQuery::new(
            "rust lib sk-test-secret",
            2,
            1,
            vec!["missing.rs".to_string()],
        )
        .expect("valid query"),
        ContextSemanticRetrievalSummary {
            status: ContextSemanticRetrievalStatus::Completed,
            provider_id: Some("fixture".to_string()),
            model_id: Some("fixture-embedding".to_string()),
            dimensions: Some(2),
            document_count: 2,
            embedded_document_count: 1,
        },
        vec![ContextHybridCandidate {
            candidate: candidate(
                ".env",
                ProjectFileKind::SecretLike,
                ProjectContextVisibility::MetadataOnly,
                875,
                vec![
                    ContextSelectionReason::PathTokenMatch,
                    ContextSelectionReason::FileNameMatch,
                    ContextSelectionReason::SemanticMatch,
                ],
            ),
            lexical_score: 0.5,
            semantic_score: Some(1.0),
            combined_score: 0.875,
        }],
        vec![ContextHybridCandidateOmission::new(
            ContextHybridCandidate {
                candidate: candidate(
                    "src/lib.rs",
                    ProjectFileKind::Source,
                    ProjectContextVisibility::SummaryOnly,
                    650,
                    vec![
                        ContextSelectionReason::PathTokenMatch,
                        ContextSelectionReason::LanguageMatch,
                    ],
                ),
                lexical_score: 1.0,
                semantic_score: Some(0.25),
                combined_score: 0.65,
            },
            ContextCandidateOmissionReason::BudgetExceeded,
        )],
        vec![
            ContextRetrievalDiagnostic::new(
                ContextRetrievalDiagnosticSeverity::Warning,
                "context_retrieval.pin.unknown",
                "pinned path was not found in project summary",
                Some("missing.rs".to_string()),
            )
            .expect("valid diagnostic"),
        ],
        1,
    );

    CoreHybridRetrievalReport { summary, hybrid }
}

#[test]
fn project_context_hybrid_snapshot_serializes_stable_safe_shape() {
    let snapshot =
        UiProjectContextHybridRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "task": "rust lib [redacted]",
            "max_candidates": 2,
            "budget_tokens": 1,
            "used_tokens": 1,
            "selected_count": 1,
            "omitted_count": 1,
            "diagnostic_count": 1,
            "semantic": {
                "status": "completed",
                "provider_id": "fixture",
                "model_id": "fixture-embedding",
                "dimensions": 2,
                "document_count": 2,
                "embedded_document_count": 1
            },
            "selected": [
                {
                    "path": ".env",
                    "kind": "secret_like",
                    "language": null,
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 1,
                    "visibility": "metadata_only",
                    "score": 875,
                    "lexical_score": 0.5,
                    "semantic_score": 1.0,
                    "combined_score": 0.875,
                    "reasons": ["path_token_match", "file_name_match", "semantic_match"]
                }
            ],
            "omitted": [
                {
                    "path": "src/lib.rs",
                    "kind": "source",
                    "language": "rust",
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 1,
                    "visibility": "summary_only",
                    "score": 650,
                    "lexical_score": 1.0,
                    "semantic_score": 0.25,
                    "combined_score": 0.65,
                    "reasons": ["path_token_match", "language_match"],
                    "reason": "budget_exceeded"
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
}

#[test]
fn project_context_hybrid_snapshot_json_frame_is_single_line() {
    let snapshot =
        UiProjectContextHybridRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let line = encode_json_frame(&UiJsonFrame::ProjectContextHybridRetrievalSnapshot(
        snapshot,
    ))
    .expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(
        frame,
        UiJsonFrame::ProjectContextHybridRetrievalSnapshot(_)
    ));
}

#[test]
fn project_context_hybrid_snapshot_hides_secret_content_and_vectors() {
    let snapshot =
        UiProjectContextHybridRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("values"));
    assert!(!serialized.contains("text_hash"));
}
