use agent_core::{
    ProjectContextDiagnostic, ProjectContextDiagnosticSeverity, ProjectContextLimits,
    ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary,
};
use agent_ui_bridge::{UiJsonFrame, UiProjectContextSnapshot, encode_json_frame};
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
            .expect("valid file"),
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
        vec![ProjectDirectorySummary::new("src", 1).expect("valid directory")],
        vec![ProjectContextOmission::new("target", "ignored_path").expect("valid omission")],
        vec![
            ProjectContextDiagnostic::new(
                ProjectContextDiagnosticSeverity::Warning,
                "project_context.limit.max_files",
                "scan limit reached",
                None,
            )
            .expect("valid diagnostic"),
        ],
        limits(),
    )
    .expect("valid summary")
}

#[test]
fn project_context_snapshot_serializes_stable_safe_shape() {
    let snapshot = UiProjectContextSnapshot::from_summary(&summary()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "file_count": 2,
            "directory_count": 1,
            "omitted_count": 1,
            "diagnostic_count": 1,
            "files": [
                {
                    "path": ".env",
                    "kind": "secret_like",
                    "language": null,
                    "size_bytes": 32,
                    "line_count": null,
                    "token_estimate": 1,
                    "visibility": "metadata_only"
                },
                {
                    "path": "src/lib.rs",
                    "kind": "source",
                    "language": "rust",
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 12,
                    "visibility": "summary_only"
                }
            ],
            "omitted": [
                {
                    "path": "target",
                    "reason": "ignored_path"
                }
            ],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "project_context.limit.max_files",
                    "message": "scan limit reached",
                    "path": null
                }
            ]
        })
    );
}

#[test]
fn project_context_snapshot_json_frame_is_single_line() {
    let snapshot = UiProjectContextSnapshot::from_summary(&summary()).expect("snapshot");
    let line = encode_json_frame(&UiJsonFrame::ProjectContextSnapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::ProjectContextSnapshot(_)));
}

#[test]
fn project_context_snapshot_hides_secret_like_content() {
    let snapshot = UiProjectContextSnapshot::from_summary(&summary()).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}
