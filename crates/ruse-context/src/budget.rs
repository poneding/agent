use serde::{Deserialize, Serialize};

use crate::{ContextError, PromptLayer};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextBudget {
    pub max_tokens: u32,
    pub reserved_response_tokens: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetClass {
    Required,
    Optional,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetItem {
    pub layer: PromptLayer,
    pub class: BudgetClass,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetDecision {
    Included,
    Omitted { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetPlan {
    pub included: Vec<PromptLayer>,
    pub omitted: Vec<(PromptLayer, String)>,
    pub used_tokens: u32,
    pub available_tokens: u32,
}

pub fn plan_context_budget(
    budget: ContextBudget,
    items: Vec<BudgetItem>,
) -> Result<BudgetPlan, ContextError> {
    let available_tokens = available_tokens(budget)?;
    let mut included = Vec::new();
    let mut omitted = Vec::new();
    let mut used_tokens = 0;

    for item in items {
        let next_used = used_tokens + item.layer.token_estimate;
        if next_used <= available_tokens {
            used_tokens = next_used;
            included.push(item.layer);
            continue;
        }

        match item.class {
            BudgetClass::Required => {
                return Err(ContextError::BudgetExceeded(format!(
                    "required layer {} needs {} tokens with {} tokens available",
                    item.layer.id, item.layer.token_estimate, available_tokens
                )));
            }
            BudgetClass::Optional => {
                omitted.push((item.layer, "budget_exceeded".to_string()));
            }
        }
    }

    Ok(BudgetPlan {
        included,
        omitted,
        used_tokens,
        available_tokens,
    })
}

fn available_tokens(budget: ContextBudget) -> Result<u32, ContextError> {
    if budget.reserved_response_tokens >= budget.max_tokens {
        return Err(ContextError::InvalidBudget(
            "reserved response tokens must be less than max tokens".to_string(),
        ));
    }
    Ok(budget.max_tokens - budget.reserved_response_tokens)
}
