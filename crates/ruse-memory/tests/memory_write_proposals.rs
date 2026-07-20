use ruse_memory::{
    InMemoryMemoryProposalStore, MemoryError, MemoryProposalDecision, MemoryProposalStatus,
    MemoryProposalStore, MemoryScope, MemorySensitivity, MemoryWriteProposal,
};
use ruse_types::{ConversationId, EventId, WorkspaceId};

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000002404"
        .parse::<EventId>()
        .expect("event id")
}

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000002414"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn project_scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: "00000000-0000-0000-0000-000000002424"
            .parse::<WorkspaceId>()
            .expect("workspace id"),
    }
}

fn proposal(
    proposal_id: &str,
    proposed_record_id: &str,
    scope: MemoryScope,
    sensitivity: MemorySensitivity,
) -> MemoryWriteProposal {
    MemoryWriteProposal::new(
        proposal_id,
        proposed_record_id,
        scope,
        "prefer deterministic memory write reviews",
        "deterministic memory write review preference",
        vec![event_id()],
        90,
        sensitivity,
    )
    .expect("valid proposal")
}

#[test]
fn memory_write_proposals_validate_required_fields_and_redact_errors() {
    let empty_id = MemoryWriteProposal::new(
        " ",
        "record",
        conversation_scope(),
        "content",
        "summary",
        vec![event_id()],
        80,
        MemorySensitivity::Internal,
    )
    .expect_err("empty proposal id rejected");
    assert!(matches!(
        empty_id,
        MemoryError::InvalidProposal(message) if message.contains("proposal id")
    ));

    let empty_content = MemoryWriteProposal::new(
        "proposal",
        "record",
        conversation_scope(),
        " ",
        "summary",
        vec![event_id()],
        80,
        MemorySensitivity::Internal,
    )
    .expect_err("empty content rejected");
    assert!(matches!(
        empty_content,
        MemoryError::InvalidProposal(message) if message.contains("content")
    ));

    let empty_summary = MemoryWriteProposal::new(
        "proposal",
        "record",
        conversation_scope(),
        "content",
        " ",
        vec![event_id()],
        80,
        MemorySensitivity::Internal,
    )
    .expect_err("empty summary rejected");
    assert!(matches!(
        empty_summary,
        MemoryError::InvalidProposal(message) if message.contains("summary")
    ));

    let invalid_confidence = MemoryWriteProposal::new(
        "proposal",
        "record",
        conversation_scope(),
        "content",
        "summary",
        vec![event_id()],
        0,
        MemorySensitivity::Internal,
    )
    .expect_err("invalid confidence rejected");
    assert!(matches!(
        invalid_confidence,
        MemoryError::InvalidProposal(message) if message.contains("confidence")
    ));

    let mut store = InMemoryMemoryProposalStore::default();
    store
        .add_proposal(proposal(
            "sk-secret-proposal",
            "record-one",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect("first proposal");
    let duplicate = store
        .add_proposal(proposal(
            "sk-secret-proposal",
            "record-two",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect_err("duplicate proposal rejected");
    let duplicate_message = duplicate.to_string();
    assert!(duplicate_message.contains("[redacted]"));
    assert!(!duplicate_message.contains("sk-secret-proposal"));

    let duplicate_record = store
        .add_proposal(proposal(
            "proposal-two",
            "record-one",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect_err("duplicate proposed record rejected");
    assert!(matches!(
        duplicate_record,
        MemoryError::InvalidProposal(message) if message.contains("proposed record id")
    ));
}

#[test]
fn memory_write_proposals_approve_commit_and_repeat_commit_idempotently() {
    let mut store = InMemoryMemoryProposalStore::default();
    store
        .add_proposal(proposal(
            "proposal-a",
            "record-a",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect("add proposal");

    let approved = store
        .resolve("proposal-a", MemoryProposalDecision::Approve)
        .expect("approve proposal");
    assert_eq!(approved.status, MemoryProposalStatus::Approved);

    let record = store.commit("proposal-a").expect("commit proposal");
    assert_eq!(record.id, "record-a");
    assert_eq!(record.text, "prefer deterministic memory write reviews");
    assert_eq!(record.source.event_id, Some(event_id()));
    assert_eq!(
        record.source.description,
        "deterministic memory write review preference"
    );
    assert_eq!(record.confidence, 90);
    assert_eq!(record.sensitivity, MemorySensitivity::Internal);

    let repeated = store
        .commit("proposal-a")
        .expect("repeat commit is idempotent");
    assert_eq!(repeated, record);
    assert_eq!(
        store
            .inspect_proposal("proposal-a")
            .expect("proposal exists")
            .status,
        MemoryProposalStatus::Committed
    );
}

#[test]
fn memory_write_proposals_reject_invalid_terminal_transitions() {
    let mut store = InMemoryMemoryProposalStore::default();
    store
        .add_proposal(proposal(
            "proposal-reject",
            "record-reject",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect("add rejected proposal");
    store
        .add_proposal(proposal(
            "proposal-expire",
            "record-expire",
            conversation_scope(),
            MemorySensitivity::Internal,
        ))
        .expect("add expired proposal");

    let rejected = store
        .resolve(
            "proposal-reject",
            MemoryProposalDecision::Reject {
                reason: "not accurate".to_string(),
            },
        )
        .expect("reject proposal");
    assert_eq!(rejected.status, MemoryProposalStatus::Rejected);

    let expired = store
        .resolve(
            "proposal-expire",
            MemoryProposalDecision::Expire {
                reason: "stale suggestion".to_string(),
            },
        )
        .expect("expire proposal");
    assert_eq!(expired.status, MemoryProposalStatus::Expired);

    let commit_rejected = store
        .commit("proposal-reject")
        .expect_err("rejected proposal cannot commit");
    assert!(matches!(
        commit_rejected,
        MemoryError::InvalidProposal(message) if message.contains("approved")
    ));

    let reapprove = store
        .resolve("proposal-reject", MemoryProposalDecision::Approve)
        .expect_err("terminal proposal cannot resolve again");
    assert!(matches!(
        reapprove,
        MemoryError::InvalidProposal(message) if message.contains("pending")
    ));
}

#[test]
fn memory_write_proposals_list_deterministically_and_report_auto_approval_eligibility() {
    let mut store = InMemoryMemoryProposalStore::default();
    let public_project = proposal(
        "proposal-b",
        "record-b",
        project_scope(),
        MemorySensitivity::Public,
    );
    let sensitive_conversation = proposal(
        "proposal-a",
        "record-a",
        conversation_scope(),
        MemorySensitivity::Sensitive,
    );

    assert!(public_project.auto_approval_eligibility().eligible);
    assert!(!public_project.requires_explicit_approval());
    assert!(!sensitive_conversation.auto_approval_eligibility().eligible);
    assert!(sensitive_conversation.requires_explicit_approval());

    store.add_proposal(public_project).expect("public project");
    store
        .add_proposal(sensitive_conversation)
        .expect("sensitive conversation");

    let ids = store
        .list_proposals()
        .into_iter()
        .map(|proposal| proposal.proposal_id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["proposal-a", "proposal-b"]);
}

#[test]
fn memory_write_proposals_safe_summary_excludes_content_and_redacts_secret_like_text() {
    let proposal = MemoryWriteProposal::new(
        "proposal-safe",
        "record-safe",
        project_scope(),
        "raw sk-secret token content must stay private",
        "summary mentions sk-secret token",
        vec![event_id()],
        90,
        MemorySensitivity::Public,
    )
    .expect("proposal");

    let summary = proposal.safe_summary();
    assert_eq!(summary.proposal_id, "proposal-safe");
    assert_eq!(summary.summary, "summary mentions [redacted] [redacted]");
    assert!(summary.content_redacted);
    assert!(!summary.auto_approval.eligible);

    let serialized = serde_json::to_string(&summary).expect("serialize summary");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("raw sk-secret token content"));
    assert!(!serialized.contains("sk-secret"));
}
