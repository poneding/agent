use ruse_core::{
    ContextCandidate, ContextCandidateOmission, ContextCandidateOmissionReason,
    ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery,
    ContextRetrievalReport, ContextSelectionReason, ProjectContextLimits,
    ProjectContextRetrievalReport as CoreRetrievalReport, ProjectContextSummary,
    ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
};
use ruse_ui_bridge::UiProjectContextDetail;
use ruse_ui_bridge::{UiJsonFrame, UiProjectContextRetrievalSnapshot, encode_json_frame};
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

fn report() -> CoreRetrievalReport {
    let summary = summary();
    let retrieval = ContextRetrievalReport::new(
        ContextRetrievalQuery::new(
            "rust lib env sk-test-secret",
            2,
            1,
            vec!["missing.rs".to_string()],
        )
        .expect("valid query"),
        vec![
            ContextCandidate::new(
                ".env",
                ProjectFileKind::SecretLike,
                None,
                32,
                None,
                1,
                ProjectContextVisibility::MetadataOnly,
                35,
                vec![
                    ContextSelectionReason::PathTokenMatch,
                    ContextSelectionReason::FileNameMatch,
                    ContextSelectionReason::KindMatch,
                ],
            )
            .expect("valid selected candidate"),
        ],
        vec![ContextCandidateOmission::new(
            ContextCandidate::new(
                "src/lib.rs",
                ProjectFileKind::Source,
                Some("rust".to_string()),
                120,
                Some(7),
                12,
                ProjectContextVisibility::SummaryOnly,
                37,
                vec![
                    ContextSelectionReason::PathTokenMatch,
                    ContextSelectionReason::FileNameMatch,
                    ContextSelectionReason::LanguageMatch,
                ],
            )
            .expect("valid omitted candidate"),
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
    CoreRetrievalReport { summary, retrieval }
}

#[test]
fn project_context_retrieval_snapshot_serializes_stable_safe_shape() {
    let snapshot = UiProjectContextRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "task": "rust lib env [redacted]",
            "max_candidates": 2,
            "budget_tokens": 1,
            "used_tokens": 1,
            "selected_count": 1,
            "omitted_count": 1,
            "diagnostic_count": 1,
            "selected": [
                {
                    "path": ".env",
                    "kind": "secret_like",
                    "language": null,
                    "size_bytes": 32,
                    "line_count": null,
                    "token_estimate": 1,
                    "visibility": "metadata_only",
                    "score": 35,
                    "reasons": ["path_token_match", "file_name_match", "kind_match"]
                }
            ],
            "omitted": [
                {
                    "path": "src/lib.rs",
                    "kind": "source",
                    "language": "rust",
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 12,
                    "visibility": "summary_only",
                    "score": 37,
                    "reasons": ["path_token_match", "file_name_match", "language_match"],
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
fn project_context_retrieval_snapshot_json_frame_is_single_line() {
    let snapshot = UiProjectContextRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let line =
        encode_json_frame(&UiJsonFrame::project_context_retrieval(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(
        frame,
        UiJsonFrame::ProjectContextDetail(UiProjectContextDetail::Retrieval(_))
    ));
}

#[test]
fn project_context_retrieval_snapshot_hides_secret_like_content() {
    let snapshot = UiProjectContextRetrievalSnapshot::from_report(&report()).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}
