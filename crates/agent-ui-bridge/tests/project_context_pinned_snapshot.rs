use agent_ui_bridge::UiProjectContextDetail;
use agent_core::{
    ContextBudget, ContextCandidate, ContextRetrievalDiagnostic,
    ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery, ContextRetrievalReport,
    ContextSelectionReason, PinnedContextItem, PinnedContextOmission, PinnedContextOmissionReason,
    PinnedContextPriority, PinnedContextSource, PinnedProjectContextBudgetReport,
    PinnedProjectContextPin, PinnedProjectContextRequest, ProjectContextLimits,
    ProjectContextPinnedRetrievalReport as CorePinnedRetrievalReport, ProjectContextSummary,
    ProjectContextVisibility, ProjectFileKind, ProjectFileSummary,
};
use agent_ui_bridge::{UiJsonFrame, UiProjectContextPinnedRetrievalSnapshot, encode_json_frame};
use serde_json::json;

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 16_384, 1_000_000).expect("valid limits")
}

fn summary() -> ProjectContextSummary {
    ProjectContextSummary::new(
        ".",
        vec![
            ProjectFileSummary::new(
                "README.md",
                ProjectFileKind::Documentation,
                Some("markdown".to_string()),
                120,
                Some(7),
                12,
                Some("fnv1a64:readme".to_string()),
                ProjectContextVisibility::SummaryOnly,
            )
            .expect("valid readme"),
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

fn retrieval() -> ContextRetrievalReport {
    ContextRetrievalReport::new(
        ContextRetrievalQuery::new("readme sk-test-secret", 2, 80, Vec::new()).expect("query"),
        vec![
            ContextCandidate::new(
                "README.md",
                ProjectFileKind::Documentation,
                Some("markdown".to_string()),
                120,
                Some(7),
                12,
                ProjectContextVisibility::SummaryOnly,
                42,
                vec![ContextSelectionReason::PathTokenMatch],
            )
            .expect("candidate"),
        ],
        vec![],
        vec![],
        12,
    )
}

fn pinned() -> PinnedProjectContextBudgetReport {
    PinnedProjectContextBudgetReport {
        request: PinnedProjectContextRequest::new(
            vec![PinnedProjectContextPin::required("README.md", "entry api_key").expect("pin")],
            ContextBudget {
                max_tokens: 100,
                reserved_response_tokens: 20,
            },
            true,
        )
        .expect("request"),
        budget: ContextBudget {
            max_tokens: 100,
            reserved_response_tokens: 20,
        },
        available_tokens: 80,
        used_tokens: 12,
        included: vec![PinnedContextItem {
            path: "README.md".to_string(),
            kind: ProjectFileKind::Documentation,
            language: Some("markdown".to_string()),
            visibility: ProjectContextVisibility::SummaryOnly,
            token_estimate: 12,
            priority: PinnedContextPriority::Required,
            source: PinnedContextSource::ProjectFile,
            reason: "entry [redacted]".to_string(),
        }],
        omitted: vec![PinnedContextOmission {
            path: "missing.rs".to_string(),
            priority: PinnedContextPriority::Optional,
            reason: PinnedContextOmissionReason::UnknownPath,
            message: "optional pinned context path was not found".to_string(),
        }],
        retrieval_included: true,
        retrieval_omitted_reason: None,
        diagnostics: vec![
            ContextRetrievalDiagnostic::new(
                ContextRetrievalDiagnosticSeverity::Warning,
                "context_pins.optional.unknown",
                "optional pinned context path was not found",
                Some("missing.rs".to_string()),
            )
            .expect("diagnostic"),
        ],
    }
}

fn report() -> CorePinnedRetrievalReport {
    CorePinnedRetrievalReport {
        summary: summary(),
        retrieval: retrieval(),
        pinned: pinned(),
    }
}

#[test]
fn project_context_pinned_snapshot_serializes_stable_safe_shape() {
    let snapshot = UiProjectContextPinnedRetrievalSnapshot::from_retrieval_report(&report())
        .expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "task": "readme [redacted]",
            "retrieval_kind": "lexical",
            "max_candidates": 2,
            "retrieval_budget_tokens": 80,
            "context_budget_tokens": 100,
            "reserved_response_tokens": 20,
            "available_tokens": 80,
            "used_tokens": 12,
            "included_count": 1,
            "omitted_count": 1,
            "diagnostic_count": 1,
            "retrieval_included": true,
            "retrieval_omitted_reason": null,
            "included": [
                {
                    "path": "README.md",
                    "kind": "documentation",
                    "language": "markdown",
                    "visibility": "summary_only",
                    "token_estimate": 12,
                    "priority": "required",
                    "source": "project_file",
                    "reason": "entry [redacted]"
                }
            ],
            "omitted": [
                {
                    "path": "missing.rs",
                    "priority": "optional",
                    "reason": "unknown_path",
                    "message": "optional pinned context path was not found"
                }
            ],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "context_pins.optional.unknown",
                    "message": "optional pinned context path was not found",
                    "path": "missing.rs"
                }
            ]
        })
    );
}

#[test]
fn project_context_pinned_snapshot_json_frame_is_single_line() {
    let snapshot = UiProjectContextPinnedRetrievalSnapshot::from_retrieval_report(&report())
        .expect("snapshot");
    let line = encode_json_frame(&UiJsonFrame::project_context_pinned(
        snapshot,
    ))
    .expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(
        frame,
        UiJsonFrame::ProjectContextDetail(UiProjectContextDetail::Pinned(_))
    ));
}

#[test]
fn project_context_pinned_snapshot_hides_secret_content() {
    let snapshot = UiProjectContextPinnedRetrievalSnapshot::from_retrieval_report(&report())
        .expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("values"));
    assert!(!serialized.contains("text_hash"));
}
