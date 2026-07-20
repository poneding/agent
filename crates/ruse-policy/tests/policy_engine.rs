use ruse_policy::{
    CapabilityAction, PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
    StaticPolicyEngine,
};
use ruse_types::{ActorRef, CorrelationId};
use serde_json::json;

fn request(action: CapabilityAction, kind: ResourceKind, pattern: &str) -> PermissionRequest {
    PermissionRequest::new(
        ActorRef::system(),
        action,
        ResourceScope::new(kind, pattern).expect("valid scope"),
        "test policy decision",
        CorrelationId::new(),
    )
    .expect("valid request")
}

#[test]
fn static_policy_defaults_to_ask() {
    let engine = StaticPolicyEngine::default();
    let request = request(CapabilityAction::Read, ResourceKind::File, "README.md");

    let decision = engine.decide(&request).expect("policy decision");

    assert_eq!(
        decision,
        PolicyDecision::Ask {
            reason: "no matching policy rule".to_string(),
        }
    );
}

#[test]
fn static_policy_allows_matching_rule_and_issues_lease() {
    let scope = ResourceScope::new(ResourceKind::File, "README.md").expect("valid scope");
    let engine = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        scope.clone(),
        PolicyDecision::Allow {
            reason: "readme is safe".to_string(),
        },
    );
    let request = PermissionRequest::new(
        ActorRef::system(),
        CapabilityAction::Read,
        scope,
        "read docs",
        CorrelationId::new(),
    )
    .expect("valid request");

    let decision = engine.decide(&request).expect("policy decision");
    let lease = engine
        .issue_lease(&request, decision.clone())
        .expect("issue lease")
        .expect("allow creates lease");

    assert_eq!(
        decision,
        PolicyDecision::Allow {
            reason: "readme is safe".to_string(),
        }
    );
    assert_eq!(lease.request_id, request.id);
    assert_eq!(lease.decision, decision);
}

#[test]
fn static_policy_denies_matching_rule_without_lease() {
    let scope = ResourceScope::new(ResourceKind::Command, "rm").expect("valid scope");
    let engine = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        scope.clone(),
        PolicyDecision::Deny {
            reason: "destructive command".to_string(),
        },
    );
    let request = PermissionRequest::new(
        ActorRef::system(),
        CapabilityAction::Execute,
        scope,
        "run command",
        CorrelationId::new(),
    )
    .expect("valid request");

    let decision = engine.decide(&request).expect("policy decision");
    let lease = engine
        .issue_lease(&request, decision.clone())
        .expect("lease check");

    assert_eq!(
        decision,
        PolicyDecision::Deny {
            reason: "destructive command".to_string(),
        }
    );
    assert!(lease.is_none());
}

#[test]
fn permission_request_and_decision_serialize_stable_shape() {
    let request = PermissionRequest {
        id: "request-1".to_string(),
        actor: ActorRef::system(),
        action: CapabilityAction::Read,
        scope: ResourceScope {
            kind: ResourceKind::File,
            pattern: "README.md".to_string(),
        },
        reason: "read docs".to_string(),
        correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
    };
    let decision = PolicyDecision::Allow {
        reason: "readme is safe".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&request).expect("serialize request"),
        json!({
            "id": "request-1",
            "actor": {
                "kind": "system",
                "id": "system",
                "display_name": "system"
            },
            "action": "read",
            "scope": {
                "kind": "file",
                "pattern": "README.md"
            },
            "reason": "read docs",
            "correlation_id": "00000000-0000-0000-0000-000000000000"
        })
    );
    assert_eq!(
        serde_json::to_value(&decision).expect("serialize decision"),
        json!({
            "allow": {
                "reason": "readme is safe"
            }
        })
    );
}
