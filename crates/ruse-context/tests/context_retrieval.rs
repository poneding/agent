use ruse_context::{
    ContextCandidateOmissionReason, ContextRetrievalQuery, ContextSelectionReason,
    ProjectContextLimits, ProjectContextSummary, ProjectContextVisibility, ProjectDirectorySummary,
    ProjectFileKind, ProjectFileSummary, PromptLayerKind, PromptLayerSensitivity,
    PromptLayerVisibility, retrieve_relevant_context,
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
                "crates/ruse-cli/src/command.rs",
                ProjectFileKind::Source,
                Some("rust"),
                80,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                "crates/ruse-cli/tests/project_context_commands.rs",
                ProjectFileKind::Test,
                Some("rust"),
                24,
                ProjectContextVisibility::SummaryOnly,
            ),
            file(
                "crates/ruse-ui-bridge/src/frame.rs",
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
            file(
                "README.md",
                ProjectFileKind::Documentation,
                Some("markdown"),
                10,
                ProjectContextVisibility::SummaryOnly,
            ),
        ],
        vec![
            ProjectDirectorySummary::new("crates/ruse-cli/src", 1).expect("valid directory"),
            ProjectDirectorySummary::new("crates/ruse-cli/tests", 1).expect("valid directory"),
            ProjectDirectorySummary::new("crates/ruse-ui-bridge/src", 1).expect("valid directory"),
        ],
        vec![],
        vec![],
        limits(),
    )
    .expect("valid summary")
}

#[test]
fn retrieval_contract_serializes_stable_safe_shape() {
    let query = ContextRetrievalQuery::new(
        "update rust CLI context command",
        3,
        160,
        vec!["README.md".to_string()],
    )
    .expect("valid query");
    let report = retrieve_relevant_context(&summary(), query).expect("retrieval report");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(
        value["query"],
        json!({
            "task": "update rust CLI context command",
            "max_candidates": 3,
            "budget_tokens": 160,
            "pinned_paths": ["README.md"]
        })
    );
    assert_eq!(value["selected"][0]["path"], json!("README.md"));
    assert_eq!(value["selected"][0]["reasons"][0], json!("pinned_path"));
    assert_eq!(value["selected"][0]["visibility"], json!("summary_only"));
    assert!(value["selected"][0]["score"].as_u64().expect("score") >= 100);
    assert_eq!(value["used_tokens"], json!(114));
}

#[test]
fn retrieval_scores_and_orders_candidates_deterministically() {
    let query = ContextRetrievalQuery::new("rust cli command context", 3, 200, Vec::new())
        .expect("valid query");
    let report = retrieve_relevant_context(&summary(), query).expect("retrieval report");

    let paths: Vec<_> = report
        .selected
        .iter()
        .map(|candidate| candidate.path.as_str())
        .collect();

    assert_eq!(
        paths,
        vec![
            "crates/ruse-cli/src/command.rs",
            "crates/ruse-cli/tests/project_context_commands.rs",
            "crates/ruse-ui-bridge/src/frame.rs",
        ]
    );
    assert!(report.selected[0].score >= report.selected[1].score);
    assert!(
        report.selected[0]
            .reasons
            .contains(&ContextSelectionReason::PathTokenMatch)
    );
    assert!(
        report.selected[0]
            .reasons
            .contains(&ContextSelectionReason::FileNameMatch)
    );
    assert!(
        report.selected[0]
            .reasons
            .contains(&ContextSelectionReason::DirectoryMatch)
    );
    assert!(
        report.selected[0]
            .reasons
            .contains(&ContextSelectionReason::LanguageMatch)
    );
}

#[test]
fn retrieval_handles_pins_unknown_pins_and_budget_omissions() {
    let query = ContextRetrievalQuery::new(
        "rust cli command context",
        4,
        60,
        vec![
            "crates/ruse-ui-bridge/src/frame.rs".to_string(),
            "missing.rs".to_string(),
        ],
    )
    .expect("valid query");
    let report = retrieve_relevant_context(&summary(), query).expect("retrieval report");

    assert_eq!(
        report
            .selected
            .first()
            .map(|candidate| candidate.path.as_str()),
        Some("crates/ruse-ui-bridge/src/frame.rs")
    );
    assert!(
        report.selected[0]
            .reasons
            .contains(&ContextSelectionReason::PinnedPath)
    );
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "context_retrieval.pin.unknown"
            && diagnostic.path.as_deref() == Some("missing.rs")
    }));
    assert!(report.omitted.iter().any(|omission| {
        omission.reason == ContextCandidateOmissionReason::BudgetExceeded
            && omission.candidate.path == "crates/ruse-cli/src/command.rs"
    }));
}

#[test]
fn retrieval_rejects_invalid_queries() {
    assert!(ContextRetrievalQuery::new(" ", 1, 10, Vec::new()).is_err());
    assert!(ContextRetrievalQuery::new("task", 0, 10, Vec::new()).is_err());
    assert!(ContextRetrievalQuery::new("task", 1, 0, Vec::new()).is_err());
    assert!(
        ContextRetrievalQuery::new(
            "task",
            1,
            10,
            vec!["src/lib.rs".into(), " src/lib.rs ".into()]
        )
        .is_err()
    );
    assert!(ContextRetrievalQuery::new("task", 1, 10, vec![" ".into()]).is_err());
}

#[test]
fn retrieval_prompt_layer_is_summary_only_and_metadata_safe() {
    let query =
        ContextRetrievalQuery::new("env secret context", 2, 40, Vec::new()).expect("valid query");
    let report = retrieve_relevant_context(&summary(), query).expect("retrieval report");
    let layer = report
        .to_prompt_layer("project-context-retrieval", 60)
        .expect("prompt layer");

    assert_eq!(layer.kind, PromptLayerKind::Project);
    assert_eq!(layer.visibility, PromptLayerVisibility::SummaryOnly);
    assert_eq!(layer.sensitivity, PromptLayerSensitivity::Workspace);
    assert!(layer.content.contains(".env"));
    assert!(layer.content.contains("metadata_only"));
    assert!(!layer.content.contains("api_key"));
    assert!(!layer.content.contains("sk-test-secret"));
}
