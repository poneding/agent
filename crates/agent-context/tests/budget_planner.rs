use agent_context::{
    BudgetClass, BudgetDecision, BudgetItem, ContextBudget, ContextError, PromptLayer,
    PromptLayerKind, plan_context_budget,
};
use serde_json::json;

fn layer(id: &str, tokens: u32) -> PromptLayer {
    PromptLayer {
        id: id.to_string(),
        kind: PromptLayerKind::Project,
        priority: 10,
        content: format!("{id} content"),
        token_estimate: tokens,
    }
}

fn item(id: &str, tokens: u32, class: BudgetClass) -> BudgetItem {
    BudgetItem {
        layer: layer(id, tokens),
        class,
    }
}

#[test]
fn budget_planner_includes_required_and_fitting_optional_layers() {
    let plan = plan_context_budget(
        ContextBudget {
            max_tokens: 100,
            reserved_response_tokens: 20,
        },
        vec![
            item("required", 40, BudgetClass::Required),
            item("optional-fit", 30, BudgetClass::Optional),
            item("optional-overflow", 50, BudgetClass::Optional),
        ],
    )
    .expect("plan budget");

    let included: Vec<_> = plan
        .included
        .iter()
        .map(|layer| layer.id.as_str())
        .collect();
    let omitted: Vec<_> = plan
        .omitted
        .iter()
        .map(|(layer, reason)| (layer.id.as_str(), reason.as_str()))
        .collect();

    assert_eq!(plan.available_tokens, 80);
    assert_eq!(plan.used_tokens, 70);
    assert_eq!(included, vec!["required", "optional-fit"]);
    assert_eq!(omitted, vec![("optional-overflow", "budget_exceeded")]);
}

#[test]
fn budget_planner_rejects_required_overflow() {
    let error = plan_context_budget(
        ContextBudget {
            max_tokens: 100,
            reserved_response_tokens: 20,
        },
        vec![item("required-overflow", 90, BudgetClass::Required)],
    )
    .expect_err("required overflow rejected");

    assert!(matches!(
        error,
        ContextError::BudgetExceeded(message) if message.contains("required-overflow")
    ));
}

#[test]
fn budget_planner_omits_optional_layers_with_reasons() {
    let plan = plan_context_budget(
        ContextBudget {
            max_tokens: 100,
            reserved_response_tokens: 20,
        },
        vec![
            item("required", 40, BudgetClass::Required),
            item("optional-overflow", 50, BudgetClass::Optional),
            item("optional-fit", 30, BudgetClass::Optional),
        ],
    )
    .expect("plan budget");

    let included: Vec<_> = plan
        .included
        .iter()
        .map(|layer| layer.id.as_str())
        .collect();
    let omitted: Vec<_> = plan
        .omitted
        .iter()
        .map(|(layer, reason)| (layer.id.as_str(), reason.as_str()))
        .collect();

    assert_eq!(included, vec!["required", "optional-fit"]);
    assert_eq!(omitted, vec![("optional-overflow", "budget_exceeded")]);
}

#[test]
fn context_budget_serializes_stable_shape() {
    let value = serde_json::to_value(ContextBudget {
        max_tokens: 100,
        reserved_response_tokens: 20,
    })
    .expect("serialize budget");
    let decision = serde_json::to_value(BudgetDecision::Omitted {
        reason: "budget_exceeded".to_string(),
    })
    .expect("serialize decision");

    assert_eq!(
        value,
        json!({
            "max_tokens": 100,
            "reserved_response_tokens": 20
        })
    );
    assert_eq!(
        decision,
        json!({
            "omitted": {
                "reason": "budget_exceeded"
            }
        })
    );
}
