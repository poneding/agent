use ruse_team::{
    ArtifactRef, BlackboardEntry, BlackboardEntryKind, BlackboardStore, InMemoryBlackboardStore,
    ReviewVote, TeamError,
};
use ruse_types::{AgentId, EventId, TaskId, TeamId, Timestamp};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn team_id() -> TeamId {
    "00000000-0000-0000-0000-000000000001"
        .parse()
        .expect("valid team id")
}

fn other_team_id() -> TeamId {
    "00000000-0000-0000-0000-000000000002"
        .parse()
        .expect("valid team id")
}

fn agent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000003"
        .parse()
        .expect("valid agent id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000004"
        .parse()
        .expect("valid task id")
}

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000000005"
        .parse()
        .expect("valid event id")
}

fn artifact() -> ArtifactRef {
    ArtifactRef::new("diff", "crates/ruse-team/src/blackboard.rs", Some("hash-1"))
        .expect("valid artifact")
}

fn entry(summary: &str, created_at: Timestamp) -> BlackboardEntry {
    BlackboardEntry::new(
        team_id(),
        agent_id(),
        task_id(),
        BlackboardEntryKind::Review,
        summary,
        vec![artifact()],
        vec![event_id()],
        Some(ReviewVote::RequestChanges),
        created_at,
    )
    .expect("valid blackboard entry")
}

#[test]
fn team_blackboard_serializes_entry_shape() {
    let entry = entry("Reviewer found missing validation", ts(1));

    let value = serde_json::to_value(entry).expect("serialize blackboard entry");

    assert_eq!(value["team_id"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(value["agent_id"], "00000000-0000-0000-0000-000000000003");
    assert_eq!(value["task_id"], "00000000-0000-0000-0000-000000000004");
    assert_eq!(value["kind"], "review");
    assert_eq!(value["summary"], "Reviewer found missing validation");
    assert_eq!(value["artifacts"][0]["label"], "diff");
    assert_eq!(
        value["artifacts"][0]["path"],
        "crates/ruse-team/src/blackboard.rs"
    );
    assert_eq!(value["artifacts"][0]["content_hash"], "hash-1");
    assert_eq!(
        value["trace_event_ids"][0],
        "00000000-0000-0000-0000-000000000005"
    );
    assert_eq!(value["review_vote"], "request_changes");
    assert_eq!(value["created_at"], "2026-06-25T00:00:01Z");
}

#[test]
fn team_blackboard_appends_and_lists_entries_in_order() {
    let mut store = InMemoryBlackboardStore::default();
    let later = entry("later", ts(2));
    let earlier = entry("earlier", ts(1));
    let other_team = BlackboardEntry::new(
        other_team_id(),
        agent_id(),
        task_id(),
        BlackboardEntryKind::Finding,
        "other team",
        Vec::new(),
        Vec::new(),
        None,
        ts(0),
    )
    .unwrap();

    store.append(later).unwrap();
    store.append(other_team).unwrap();
    store.append(earlier).unwrap();

    let listed = store.list_for_team(team_id());

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].summary, "earlier");
    assert_eq!(listed[1].summary, "later");
}

#[test]
fn team_blackboard_rejects_empty_summary() {
    let mut store = InMemoryBlackboardStore::default();
    let mut invalid = entry("valid before mutation", ts(1));
    invalid.summary = " ".to_string();

    let err = store.append(invalid).expect_err("empty summary is invalid");

    assert!(matches!(err, TeamError::InvalidBlackboard(_)));
}

#[test]
fn team_blackboard_rejects_unsafe_artifact_path() {
    let empty_label = ArtifactRef::new(" ", "diffs/review.patch", None);
    let parent_traversal = ArtifactRef::new("diff", "../secret", None);

    assert!(matches!(empty_label, Err(TeamError::InvalidBlackboard(_))));
    assert!(matches!(
        parent_traversal,
        Err(TeamError::InvalidBlackboard(_))
    ));
}

#[test]
fn team_blackboard_preserves_review_vote_values() {
    let values = serde_json::json!([
        ReviewVote::Approve,
        ReviewVote::RequestChanges,
        ReviewVote::Block
    ]);

    assert_eq!(
        values,
        serde_json::json!(["approve", "request_changes", "block"])
    );
}
