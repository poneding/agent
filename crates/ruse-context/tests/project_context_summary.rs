use ruse_context::{
    ProjectContextDiagnostic, ProjectContextDiagnosticSeverity, ProjectContextLimits,
    ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary, PromptLayerKind,
    PromptLayerSensitivity, PromptLayerVisibility,
};
use serde_json::json;

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 16_384, 1_000_000).expect("valid limits")
}

fn file(path: &str, kind: ProjectFileKind) -> ProjectFileSummary {
    ProjectFileSummary::new(
        path,
        kind,
        Some("rust".to_string()),
        120,
        Some(7),
        12,
        Some(format!("fnv1a64:{path}")),
        ProjectContextVisibility::SummaryOnly,
    )
    .expect("valid file summary")
}

#[test]
fn project_context_summary_serializes_stable_sorted_shape() {
    let summary = ProjectContextSummary::new(
        ".",
        vec![
            file("src/lib.rs", ProjectFileKind::Source),
            file("README.md", ProjectFileKind::Documentation),
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
    .expect("valid summary");

    let value = serde_json::to_value(summary).expect("serialize summary");

    assert_eq!(
        value,
        json!({
            "workspace_root_display": ".",
            "files": [
                {
                    "path": "README.md",
                    "kind": "documentation",
                    "language": "rust",
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 12,
                    "content_hash": "fnv1a64:README.md",
                    "visibility": "summary_only"
                },
                {
                    "path": "src/lib.rs",
                    "kind": "source",
                    "language": "rust",
                    "size_bytes": 120,
                    "line_count": 7,
                    "token_estimate": 12,
                    "content_hash": "fnv1a64:src/lib.rs",
                    "visibility": "summary_only"
                }
            ],
            "directories": [
                {
                    "path": "src",
                    "file_count": 1
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
            ],
            "limits": {
                "max_files": 100,
                "max_depth": 8,
                "max_file_bytes": 16384,
                "max_total_bytes": 1000000
            }
        })
    );
}

#[test]
fn project_context_summary_rejects_invalid_contracts() {
    assert!(ProjectContextLimits::new(0, 8, 16_384, 1_000_000).is_err());
    assert!(ProjectContextSummary::new(" ", vec![], vec![], vec![], vec![], limits()).is_err());
    assert!(
        ProjectFileSummary::new(
            " ",
            ProjectFileKind::Source,
            None,
            1,
            Some(1),
            1,
            None,
            ProjectContextVisibility::Full,
        )
        .is_err()
    );
    assert!(
        ProjectContextSummary::new(
            ".",
            vec![
                file("src/lib.rs", ProjectFileKind::Source),
                file("src/lib.rs", ProjectFileKind::Source),
            ],
            vec![],
            vec![],
            vec![],
            limits(),
        )
        .is_err()
    );
}

#[test]
fn project_context_prompt_layer_is_bounded_and_redacted() {
    let secret = ProjectFileSummary::new(
        ".env",
        ProjectFileKind::SecretLike,
        None,
        32,
        None,
        1,
        None,
        ProjectContextVisibility::MetadataOnly,
    )
    .expect("valid secret-like summary");
    let summary = ProjectContextSummary::new(
        ".",
        vec![file("src/lib.rs", ProjectFileKind::Source), secret],
        vec![],
        vec![],
        vec![],
        limits(),
    )
    .expect("valid summary");

    let layer = summary
        .to_prompt_layer("project-context", 50)
        .expect("project prompt layer");

    assert_eq!(layer.kind, PromptLayerKind::Project);
    assert_eq!(layer.visibility, PromptLayerVisibility::SummaryOnly);
    assert_eq!(layer.sensitivity, PromptLayerSensitivity::Workspace);
    assert!(layer.content.contains("src/lib.rs"));
    assert!(layer.content.contains(".env"));
    assert!(!layer.content.contains("sk-test-secret"));
    assert!(!layer.content.contains("api_key"));
}
