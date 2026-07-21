use ruse_core::{AuditEventCategory, AuditExportError, AuditExportOptions, export_audit_bundle_at};
use ruse_types::{
    ActorKind, ActorRef, ConversationId, CorrelationId, EventSeq, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed, TurnId, WorkspaceId,
};
use serde_json::json;

fn runtime_event(seq: u64, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: Some(EventSeq::new(seq).expect("event seq")),
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef {
            kind: ActorKind::Agent,
            id: "agent-main".to_string(),
            display_name: Some("Agent".to_string()),
        },
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

fn default_options() -> AuditExportOptions {
    AuditExportOptions {
        include_diagnostics: true,
        max_payload_bytes: 512,
    }
}

#[test]
fn audit_export_serializes_stable_bundle_shape() {
    let event = runtime_event(
        7,
        RuntimeEventPayload::PermissionDecisionRecorded(ruse_types::PermissionDecisionRecorded {
            request_id: "request-1".to_string(),
            decision: "allow".to_string(),
            reason: "read-only file inspection".to_string(),
            action: None,
            resource_kind: None,
            resource: None,
        }),
    );
    let generated_at = Timestamp::now_utc();

    let bundle = export_audit_bundle_at(
        std::slice::from_ref(&event),
        default_options(),
        generated_at,
    )
    .expect("export bundle");

    assert_eq!(
        serde_json::to_value(bundle).expect("serialize bundle"),
        json!({
            "schema_version": 1,
            "generated_at": serde_json::to_value(generated_at).expect("timestamp"),
            "records": [{
                "event_id": serde_json::to_value(event.id).expect("event id"),
                "sequence": 7,
                "occurred_at": serde_json::to_value(event.occurred_at).expect("occurred at"),
                "actor": serde_json::to_value(&event.actor).expect("actor"),
                "correlation_id": serde_json::to_value(event.correlation_id).expect("correlation"),
                "category": "permission",
                "action": "permission.decision",
                "resource": "request-1",
                "decision": "allow",
                "summary": "permission allow for request-1: read-only file inspection",
                "diagnostics": []
            }],
            "summary": {
                "total_events": 1,
                "audit_records": 1,
                "tool_events": 0,
                "permission_events": 1,
                "lifecycle_events": 0,
                "model_events": 0,
                "external_events": 0,
                "diagnostic_events": 0,
                "other_events": 0
            }
        })
    );
}

#[test]
fn audit_export_maps_permission_and_tool_events() {
    let permission = runtime_event(
        1,
        RuntimeEventPayload::PermissionDecisionRecorded(ruse_types::PermissionDecisionRecorded {
            request_id: "request-1".to_string(),
            decision: "deny".to_string(),
            reason: "outside workspace".to_string(),
            action: None,
            resource_kind: None,
            resource: None,
        }),
    );
    let tool = runtime_event(
        2,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-1".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({ "path": "README.md" }),
        }),
    );

    let bundle =
        export_audit_bundle_at(&[permission, tool], default_options(), Timestamp::now_utc())
            .expect("export bundle");

    assert_eq!(bundle.records[0].category, AuditEventCategory::Permission);
    assert_eq!(bundle.records[0].action, "permission.decision");
    assert_eq!(bundle.records[0].decision.as_deref(), Some("deny"));
    assert_eq!(bundle.records[1].category, AuditEventCategory::Tool);
    assert_eq!(bundle.records[1].action, "tool.proposed");
    assert_eq!(
        bundle.records[1].resource.as_deref(),
        Some("workspace.read_file")
    );
    assert_eq!(bundle.summary.permission_events, 1);
    assert_eq!(bundle.summary.tool_events, 1);
}

#[test]
fn audit_export_redacts_secret_like_payload_fields() {
    let event = runtime_event(
        1,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-1".to_string(),
            tool_name: "web.fetch".to_string(),
            input_json: json!({
                "url": "https://example.test",
                "api_token": "sk-live-secret",
                "headers": {
                    "x-api-key": "super-secret-key"
                }
            }),
        }),
    );

    let bundle = export_audit_bundle_at(&[event], default_options(), Timestamp::now_utc())
        .expect("export bundle");

    let summary = &bundle.records[0].summary;
    assert!(summary.contains("[redacted]"));
    assert!(!summary.contains("sk-live-secret"));
    assert!(!summary.contains("super-secret-key"));
}

#[test]
fn audit_export_allows_empty_event_list() {
    let bundle =
        export_audit_bundle_at(&[], default_options(), Timestamp::now_utc()).expect("empty export");

    assert!(bundle.records.is_empty());
    assert_eq!(bundle.summary.total_events, 0);
    assert_eq!(bundle.summary.audit_records, 0);
}

#[test]
fn audit_export_rejects_invalid_options() {
    let error = export_audit_bundle_at(
        &[],
        AuditExportOptions {
            include_diagnostics: true,
            max_payload_bytes: 0,
        },
        Timestamp::now_utc(),
    )
    .expect_err("zero payload budget rejected");

    assert!(matches!(error, AuditExportError::InvalidMaxPayloadBytes));
}
