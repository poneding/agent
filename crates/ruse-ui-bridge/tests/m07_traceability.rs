use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, EventSeq, McpPolicyMappingRecorded,
    McpToolsDiscovered, RuntimeEvent, RuntimeEventPayload, SchemaVersion,
    SkillPromptContributionRecorded, Timestamp, TurnId, WorkspaceId,
};
use ruse_ui_bridge::{UiEventPayload, normalize_runtime_event};
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn runtime_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: id::<EventId>("00000000-0000-0000-0000-000000000005"),
        seq: Some(EventSeq::new(7).expect("valid sequence")),
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: Some(id::<TurnId>("00000000-0000-0000-0000-000000000003")),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        causation_id: None,
        payload,
    }
}

#[test]
fn m07_traceability_normalizes_mcp_tools_discovered() {
    let event = runtime_event(RuntimeEventPayload::McpToolsDiscovered(
        McpToolsDiscovered {
            server_id: "filesystem".to_string(),
            tool_count: 1,
            local_tool_names: vec!["mcp.filesystem.read_file".to_string()],
        },
    ));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ExternalTrace {
            source_kind: "mcp_server".to_string(),
            source_id: "filesystem".to_string(),
            summary: "discovered 1 MCP tools: mcp.filesystem.read_file".to_string(),
        }
    );
}

#[test]
fn m07_traceability_normalizes_skill_prompt_without_body() {
    let event = runtime_event(RuntimeEventPayload::SkillPromptContributionRecorded(
        SkillPromptContributionRecorded {
            skill_id: "code-review".to_string(),
            layer_id: "skill:code-review".to_string(),
            priority: 600,
            token_estimate: 4,
            source_path: "/repo/.codex/skills/code-review/SKILL.md".to_string(),
        },
    ));

    let ui_event = normalize_runtime_event(&event);
    let serialized = serde_json::to_string(&ui_event).expect("serialize UI event");

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ExternalTrace {
            source_kind: "skill".to_string(),
            source_id: "code-review".to_string(),
            summary: "skill prompt layer skill:code-review priority=600 tokens=4 source=/repo/.codex/skills/code-review/SKILL.md".to_string(),
        }
    );
    assert!(!serialized.contains("Check diffs carefully."));
}

#[test]
fn m07_traceability_normalizes_mcp_policy_mapping() {
    let event = runtime_event(RuntimeEventPayload::McpPolicyMappingRecorded(
        McpPolicyMappingRecorded {
            server_id: "filesystem".to_string(),
            remote_tool_name: "read_file".to_string(),
            local_tool_name: "mcp.filesystem.read_file".to_string(),
            action: "execute".to_string(),
            resource_kind: "tool".to_string(),
            resource_pattern: "mcp:filesystem:read_file".to_string(),
        },
    ));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ExternalTrace {
            source_kind: "mcp_server".to_string(),
            source_id: "filesystem".to_string(),
            summary: "mapped MCP tool read_file -> mcp.filesystem.read_file action=execute scope=tool:mcp:filesystem:read_file".to_string(),
        }
    );
}
