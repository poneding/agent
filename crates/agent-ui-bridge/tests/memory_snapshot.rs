use agent_core::{
    RuntimeMemoryConfig, RuntimeMemoryInput, RuntimeMemoryProposalInput,
    RuntimeMemoryProposalPolicy, assemble_runtime_memory,
};
use agent_memory::{
    MemoryPrivacyDiagnostic, MemoryPrivacyDiagnosticSeverity, MemoryProductSnapshot, MemoryRecord,
    MemoryRetrievalCandidate, MemoryScope, MemorySensitivity, MemorySource, MemoryWriteProposal,
};
use agent_types::{ConversationId, EventId, WorkspaceId};
use agent_ui_bridge::{UiJsonFrame, UiMemorySnapshot, encode_json_frame};

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000006441"
        .parse()
        .expect("event id")
}

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000006442"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn project_scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: "00000000-0000-0000-0000-000000006443"
            .parse::<WorkspaceId>()
            .expect("workspace id"),
    }
}

fn record(id: &str, text: &str, sensitivity: MemorySensitivity, confidence: u8) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope: project_scope(),
        text: text.to_string(),
        source: MemorySource {
            event_id: Some(event_id()),
            description: "memory bridge fixture".to_string(),
        },
        confidence,
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
        record: record(id, text, sensitivity, 91),
        lexical_score: combined_score - 0.1,
        semantic_score: Some(0.4),
        combined_score,
        matched_terms: vec!["memory".to_string(), "sk-secret".to_string()],
    }
}

#[test]
fn memory_snapshot_frame_has_stable_json_tag_and_redacted_runtime_evidence() {
    let evidence = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: vec![
            candidate(
                "selected-public",
                "prefer deterministic memory UI surfaces",
                MemorySensitivity::Public,
                0.95,
            ),
            candidate(
                "rejected-sensitive",
                "private sk-secret password should never render",
                MemorySensitivity::Sensitive,
                0.60,
            ),
        ],
        config: RuntimeMemoryConfig {
            max_selected: 1,
            ..Default::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: vec![RuntimeMemoryProposalInput {
            proposal_id: "proposal-review".to_string(),
            proposed_record_id: "record-review".to_string(),
            scope: conversation_scope(),
            content: "remember raw sk-secret token only through review".to_string(),
            summary: "remember review preference for sk-secret token values".to_string(),
            source_event_ids: vec![event_id()],
            confidence: 88,
            sensitivity: MemorySensitivity::Sensitive,
        }],
    })
    .expect("runtime memory")
    .evidence;

    let frame = UiJsonFrame::memory_snapshot(
        UiMemorySnapshot::from_runtime_evidence(&evidence).expect("ui memory snapshot"),
    );
    let encoded = encode_json_frame(&frame).expect("encode frame");
    let value: serde_json::Value = serde_json::from_str(&encoded).expect("json frame");

    assert_eq!(value["type"], "memory_snapshot");
    assert_eq!(value["data"]["enabled"], true);
    assert_eq!(value["data"]["layer_count"], 1);
    assert_eq!(value["data"]["record_count"], 2);
    assert_eq!(value["data"]["proposal_count"], 1);
    assert_eq!(value["data"]["selected_count"], 1);
    assert_eq!(value["data"]["rejected_count"], 1);
    assert!(
        value["data"]["records"]
            .as_array()
            .expect("records array")
            .iter()
            .any(|record| record["text_preview"] == serde_json::json!("[redacted]"))
    );
    assert_eq!(value["data"]["proposals"][0]["content_redacted"], true);
    assert!(
        value["data"]["retrieval"]["candidates"]
            .as_array()
            .expect("retrieval candidates")
            .iter()
            .any(|candidate| candidate["preview"] == serde_json::json!("[redacted]"))
    );
    assert!(
        value["data"]["influence"]["entries"]
            .as_array()
            .expect("influence entries")
            .iter()
            .any(|entry| entry["preview"] == serde_json::json!("[redacted]"))
    );

    assert!(encoded.contains("[redacted]"));
    assert!(!encoded.contains("sk-secret"));
    assert!(!encoded.contains("password should never render"));
    assert!(!encoded.contains("remember raw"));

    let decoded: UiJsonFrame = serde_json::from_str(&encoded).expect("decode frame");
    assert!(matches!(
        decoded,
        UiJsonFrame::memory_snapshot(snapshot)
            if snapshot.enabled
                && snapshot.records.len() == 2
                && snapshot.proposals.len() == 1
                && snapshot.retrieval.is_some()
                && snapshot.influence.is_some()
    ));
}

#[test]
fn memory_snapshot_frame_handles_empty_and_disabled_states() {
    let disabled = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory preference".to_string(),
        candidates: Vec::new(),
        config: RuntimeMemoryConfig {
            enabled: false,
            ..Default::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: Vec::new(),
    })
    .expect("disabled runtime memory")
    .evidence;

    let snapshot = UiMemorySnapshot::from_runtime_evidence(&disabled).expect("ui snapshot");
    assert!(!snapshot.enabled);
    assert_eq!(snapshot.record_count, 0);
    assert_eq!(snapshot.proposal_count, 0);
    assert_eq!(snapshot.diagnostics[0].code, "memory_disabled");

    let product = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[],
        &[MemoryWriteProposal::new(
            "proposal-safe",
            "record-safe",
            project_scope(),
            "remember stable UI review preferences",
            "remember stable UI review preferences",
            vec![event_id()],
            82,
            MemorySensitivity::Public,
        )
        .expect("valid proposal")],
        None,
        None,
        Default::default(),
        &[],
        vec![MemoryPrivacyDiagnostic::new(
            "memory_degraded",
            MemoryPrivacyDiagnosticSeverity::Warning,
            "memory index unavailable; showing stored summaries only",
        )],
    );
    let snapshot = UiMemorySnapshot::from_product_snapshot(&product).expect("product snapshot");
    assert!(snapshot.enabled);
    assert_eq!(snapshot.proposal_count, 1);
    assert_eq!(snapshot.diagnostics[0].severity, "warning");
}
