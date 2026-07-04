use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, McpToolsDiscovered, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, SkillPromptContributionRecorded, Timestamp, TurnId,
    WorkspaceId,
};

fn runtime_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

#[test]
fn m07_mcp_skill_traceability_serializes_runtime_event_envelope() {
    let events = vec![
        runtime_event(RuntimeEventPayload::McpToolsDiscovered(
            McpToolsDiscovered {
                server_id: "filesystem".to_string(),
                tool_count: 1,
                local_tool_names: vec!["mcp.filesystem.read_file".to_string()],
            },
        )),
        runtime_event(RuntimeEventPayload::SkillPromptContributionRecorded(
            SkillPromptContributionRecorded {
                skill_id: "code-review".to_string(),
                layer_id: "skill:code-review".to_string(),
                priority: 600,
                token_estimate: 4,
                source_path: "/repo/.codex/skills/code-review/SKILL.md".to_string(),
            },
        )),
    ];

    let serialized = serde_json::to_string(&events).expect("serialize trace events");

    assert!(serialized.contains("mcp_tools_discovered"));
    assert!(serialized.contains("skill_prompt_contribution_recorded"));
    assert!(serialized.contains("mcp.filesystem.read_file"));
    assert!(!serialized.contains("Check diffs carefully."));
}
