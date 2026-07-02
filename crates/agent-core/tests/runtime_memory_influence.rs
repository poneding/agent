use agent_core::{
    ContextAssembler, RuntimeMemoryConfig, RuntimeMemoryInput, RuntimeMemoryProposalInput,
    RuntimeMemoryProposalPolicy, assemble_runtime_memory,
};
use agent_memory::{
    MemoryRecord, MemoryRetrievalCandidate, MemoryScope, MemorySensitivity, MemorySource,
};
use agent_types::{EventId, WorkspaceId};

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000006403"
        .parse::<WorkspaceId>()
        .expect("workspace id")
}

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000006433"
        .parse::<EventId>()
        .expect("event id")
}

fn scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: workspace_id(),
    }
}

fn record(id: &str, text: &str, sensitivity: MemorySensitivity) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope: scope(),
        text: text.to_string(),
        source: MemorySource {
            event_id: Some(event_id()),
            description: "runtime memory fixture".to_string(),
        },
        confidence: 90,
        sensitivity,
    }
}

fn candidate(
    id: &str,
    text: &str,
    sensitivity: MemorySensitivity,
    combined_score: f32,
) -> MemoryRetrievalCandidate {
    MemoryRetrievalCandidate {
        record: record(id, text, sensitivity),
        lexical_score: combined_score,
        semantic_score: None,
        combined_score,
        matched_terms: vec!["memory".to_string(), "sk-secret".to_string()],
    }
}

fn proposal_input() -> RuntimeMemoryProposalInput {
    RuntimeMemoryProposalInput {
        proposal_id: "proposal-runtime".to_string(),
        proposed_record_id: "runtime-memory".to_string(),
        scope: scope(),
        content: "raw sk-secret token content must never serialize".to_string(),
        summary: "remember runtime memory review preference".to_string(),
        source_event_ids: vec![event_id()],
        confidence: 88,
        sensitivity: MemorySensitivity::Sensitive,
    }
}

#[test]
fn runtime_memory_disabled_returns_no_layers_and_safe_diagnostic() {
    let assembly = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: vec![candidate(
            "sensitive-memory",
            "private sk-secret memory",
            MemorySensitivity::Sensitive,
            1.0,
        )],
        config: RuntimeMemoryConfig {
            enabled: false,
            ..RuntimeMemoryConfig::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: vec![proposal_input()],
    })
    .expect("disabled memory assembly");

    assert!(assembly.layers.is_empty());
    assert!(!assembly.evidence.enabled);
    assert_eq!(assembly.evidence.layer_count, 0);
    assert_eq!(assembly.evidence.diagnostics[0].code, "memory_disabled");

    let serialized = serde_json::to_string(&assembly.evidence).expect("serialize evidence");
    assert!(!serialized.contains("private sk-secret memory"));
    assert!(!serialized.contains("raw sk-secret token"));
}

#[test]
fn runtime_memory_selected_layers_affect_context_assembly() {
    let assembly = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: vec![candidate(
            "public-memory",
            "Prefer focused memory review tests.",
            MemorySensitivity::Public,
            1.0,
        )],
        config: RuntimeMemoryConfig::default(),
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: Vec::new(),
    })
    .expect("memory assembly");

    assert_eq!(assembly.evidence.influence.selected_count, 1);
    assert_eq!(assembly.evidence.layer_count, 1);

    let messages =
        ContextAssembler::with_layers(assembly.layers).assemble("Implement the runtime loop.");

    assert_eq!(
        messages,
        vec![
            "Prefer focused memory review tests.".to_string(),
            "Implement the runtime loop.".to_string()
        ]
    );
}

#[test]
fn runtime_memory_influence_redacts_sensitive_previews() {
    let assembly = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: vec![
            candidate(
                "public-memory",
                "public memory preference",
                MemorySensitivity::Public,
                1.0,
            ),
            candidate(
                "sensitive-memory",
                "private sk-secret token preference",
                MemorySensitivity::Sensitive,
                0.5,
            ),
        ],
        config: RuntimeMemoryConfig {
            max_selected: 1,
            ..RuntimeMemoryConfig::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: Vec::new(),
    })
    .expect("memory assembly");

    assert_eq!(assembly.evidence.influence.selected_count, 1);
    assert_eq!(assembly.evidence.influence.rejected_count, 1);
    let retrieval = assembly
        .evidence
        .snapshot
        .retrieval
        .as_ref()
        .expect("retrieval preview");
    assert_eq!(
        retrieval.candidates[1].preview.as_deref(),
        Some("[redacted]")
    );

    let serialized = serde_json::to_string(&assembly.evidence).expect("serialize evidence");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("private sk-secret token"));
    assert!(!serialized.contains("sk-secret"));
}

#[test]
fn runtime_memory_creates_reviewable_proposal_summaries_without_content() {
    let assembly = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: Vec::new(),
        config: RuntimeMemoryConfig::default(),
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: vec![proposal_input()],
    })
    .expect("memory assembly");

    assert_eq!(assembly.evidence.proposals.created_count, 1);
    assert_eq!(assembly.evidence.proposals.proposals.len(), 1);
    assert_eq!(
        assembly.evidence.proposals.proposals[0].proposal_id,
        "proposal-runtime"
    );
    assert!(assembly.evidence.proposals.proposals[0].content_redacted);
    assert!(assembly.evidence.proposals.proposals[0].requires_explicit_approval);

    let serialized = serde_json::to_string(&assembly.evidence).expect("serialize evidence");
    assert!(serialized.contains("\"status\":\"pending\""));
    assert!(!serialized.contains("raw sk-secret token content"));
    assert!(!serialized.contains("sk-secret"));
}
