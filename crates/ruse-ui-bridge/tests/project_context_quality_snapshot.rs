use ruse_ui_bridge::UiProjectContextDetail;
use ruse_core::{
    ContextQualityBudgetPressure, ContextQualityDiagnostic, ContextQualityDiagnosticSeverity,
    ContextQualityRetrievalKind, ContextQualitySignal, ContextQualitySignalKind,
    ContextQualityTrace,
};
use ruse_ui_bridge::{UiJsonFrame, UiProjectContextQualityTraceSnapshot, encode_json_frame};
use serde_json::json;

fn trace() -> ContextQualityTrace {
    ContextQualityTrace {
        task: "command sk-test-secret palette".to_string(),
        workspace_root_display: ".".to_string(),
        retrieval_kind: ContextQualityRetrievalKind::PinnedHybrid,
        selected_count: 2,
        omitted_count: 1,
        pinned_included_count: 1,
        pinned_omitted_count: 1,
        used_tokens: 80,
        available_tokens: 100,
        budget_pressure: ContextQualityBudgetPressure::High,
        signals: vec![
            ContextQualitySignal {
                path: Some("README.md".to_string()),
                kind: ContextQualitySignalKind::IncludedRequiredPin,
                weight: 12,
                message: "included reason api_key".to_string(),
            },
            ContextQualitySignal {
                path: None,
                kind: ContextQualitySignalKind::SemanticCompleted,
                weight: 3,
                message: "semantic completed without vectors".to_string(),
            },
        ],
        diagnostics: vec![ContextQualityDiagnostic {
            severity: ContextQualityDiagnosticSeverity::Warning,
            code: "context_quality.secret".to_string(),
            message: "diagnostic sk-test-secret".to_string(),
            path: Some(".env".to_string()),
        }],
    }
}

#[test]
fn project_context_quality_snapshot_serializes_stable_safe_shape() {
    let snapshot = UiProjectContextQualityTraceSnapshot::from_trace(&trace()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "task": "command [redacted] palette",
            "retrieval_kind": "pinned_hybrid",
            "selected_count": 2,
            "omitted_count": 1,
            "pinned_included_count": 1,
            "pinned_omitted_count": 1,
            "used_tokens": 80,
            "available_tokens": 100,
            "budget_pressure": "high",
            "signal_count": 2,
            "diagnostic_count": 1,
            "signals": [
                {
                    "path": "README.md",
                    "kind": "included_required_pin",
                    "weight": 12,
                    "message": "included reason [redacted]"
                },
                {
                    "path": null,
                    "kind": "semantic_completed",
                    "weight": 3,
                    "message": "semantic completed without vectors"
                }
            ],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "[redacted]",
                    "message": "diagnostic [redacted]",
                    "path": ".env"
                }
            ]
        })
    );
}

#[test]
fn project_context_quality_snapshot_json_frame_is_single_line() {
    let snapshot = UiProjectContextQualityTraceSnapshot::from_trace(&trace()).expect("snapshot");
    let line = encode_json_frame(&UiJsonFrame::project_context_quality(snapshot))
        .expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(
        frame,
        UiJsonFrame::ProjectContextDetail(UiProjectContextDetail::Quality(_))
    ));
}

#[test]
fn project_context_quality_snapshot_hides_secret_content() {
    let snapshot = UiProjectContextQualityTraceSnapshot::from_trace(&trace()).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("values"));
}
